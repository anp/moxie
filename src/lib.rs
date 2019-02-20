#![feature(await_macro, futures_api, async_await, integer_atomics, gen_future)]

#[macro_use]
extern crate rental;

mod double_waker;
#[macro_use]
pub mod scope;
mod events;
mod surface;

// TODO split into public/private preludes
pub mod prelude {
    pub use {
        crate::{
            scope::{CallsiteId, Guard, Handle, Moniker, Scope, ScopeId},
            surface::surface,
            {Components, Composer},
        },
        futures::stream::{Stream, StreamExt},
        log::{debug, error, info, trace, warn},
        parking_lot::Mutex,
        std::{future::Future, sync::Arc, task::Waker},
    };
}

use {
    crate::{prelude::*, scope::Scope},
    chashmap::CHashMap,
    futures::{executor::ThreadPool, pending},
    salsa::Database as SalsaBowl,
};

pub fn run() {
    let compose = Composer::new();
    compose.start();
}

/// A `Composer` is the primary entry point to moxie's runtime systems. It contains the salsa
/// incremental storage, a futures executor, interners, and is passed to every composable function.
#[salsa::database(ComponentStorage)]
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

    pub fn start(mut self) {
        let mut exec = self.exec.clone();

        info!("starting threadpool");
        exec.run(
            async {
                // make sure we can be woken back up
                std::future::get_task_waker(|lw| self.set_composition_waker(lw.clone().into()));
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

#[salsa::query_group(ComponentStorage)]
pub trait Components: Runtime {
    #[salsa::input]
    fn composition_waker(&self) -> Waker;

    #[salsa::dependencies]
    fn surface(&self, parent: ScopeId) -> ();
}

pub trait Runtime: SalsaBowl {
    fn scope(&self, scope: ScopeId) -> Scope;
}

impl Runtime for Composer {
    fn scope(&self, id: ScopeId) -> Scope {
        let mut port = None;

        self.states.alter(id, |prev: Option<Scope>| {
            let current =
                prev.unwrap_or_else(|| Scope::new(id, self.spawner(), self.composition_waker()));
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
