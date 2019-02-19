#![feature(await_macro, futures_api, async_await, integer_atomics)]

#[macro_use]
extern crate rental;

#[macro_use]
pub mod scope;
mod events;
mod surface;

// TODO split into public/private preludes
pub mod prelude {
    pub use {
        crate::{
            scope::{CallsiteId, Guard, Handle, Moniker, Scope, ScopeId},
            {Composer, Surface},
        },
        futures::{
            prelude::*,
            stream::{Stream, StreamExt},
        },
        log::{debug, error, info, trace, warn},
        std::sync::Arc,
    };
}

use {
    crate::{prelude::*, scope::Scope},
    chashmap::CHashMap,
    futures::{executor::ThreadPool, pending},
    salsa::Database as SalsaBowl,
};

/// A `Composer` is the primary entry point to moxie's runtime systems. It contains the salsa
/// incremental storage, a futures executor, interners, and is passed to every composable function.
#[salsa::database(ComposeStorage)]
pub struct Composer {
    runtime: salsa::Runtime<Composer>,
    states: CHashMap<ScopeId, Scope>,
    exec: ThreadPool,
}

impl Composer {
    pub fn new() -> Self {
        Self {
            runtime: salsa::Runtime::default(),
            states: CHashMap::default(),
            exec: ThreadPool::new().unwrap(),
        }
    }

    pub fn run() {
        let compose = Self::new();
        compose.start();
    }

    pub fn start(self) {
        let mut exec = self.exec.clone();

        info!("starting threadpool");
        exec.run(
            async {
                loop {
                    trace!("composing surface");
                    self.surface(scope!());
                    pending!();
                }
            },
        );
    }

    fn spawner(&self) -> Spawner {
        Spawner(self.exec.clone())
    }
}

use surface::surface;

#[salsa::query_group(ComposeStorage)]
pub trait Surface: SalsaBowl + Runtime {
    #[salsa::dependencies]
    fn surface(&self, parent: ScopeId) -> ();
}

pub trait Runtime {
    fn scope(&self, scope: ScopeId) -> Scope;
}

impl Runtime for Composer {
    fn scope(&self, id: ScopeId) -> Scope {
        let mut port = None;

        self.states.alter(id, |prev: Option<Scope>| {
            let current = prev.unwrap_or_else(|| Scope::new(id, self.spawner()));
            port = Some(current.clone());
            Some(current)
        });

        port.unwrap()
    }
}

impl SalsaBowl for Composer {
    fn salsa_runtime(&self) -> &salsa::Runtime<Composer> {
        &self.runtime
    }
}

/// A handle to the main executor to spawn additional futures.
#[derive(Clone, Debug)]
struct Spawner(futures::executor::ThreadPool);

impl futures::task::Spawn for Spawner {
    fn spawn_obj(
        &mut self,
        future: futures::future::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        self.0.spawn_obj(future)
    }
}
