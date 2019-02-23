#![deny(clippy::all)]
#![allow(clippy::unused_unit)]
#![feature(await_macro, futures_api, async_await, integer_atomics, gen_future)]

#[macro_use]
extern crate rental;

#[macro_use]
pub mod scope;
mod drop_guard;
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
    futures::{
        executor::ThreadPool,
        future::{AbortHandle, Abortable},
        pending,
    },
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

    pub fn start(self) {
        let mut exec = self.exec.clone();

        let (exit_handle, exit_registration) = AbortHandle::new_pair();
        let main_loop = Abortable::new(
            async {
                let mut compose = self;
                // make sure we can be woken back up
                std::future::get_task_waker(|lw| compose.set_waker(lw.clone().into()));
                // make sure we can be exited
                compose.set_top_level_exit(exit_handle);

                loop {
                    trace!("composing surface");
                    compose.surface(scope!(), 1920, 1080);

                    // unless we stash our own waker above, we'll never get woken again, be careful
                    pending!();
                }
            },
            exit_registration,
        );

        info!("running top-level composition loop");
        let _ = exec.run(main_loop);
    }

    fn spawner(&self) -> Spawner {
        Spawner(self.exec.clone())
    }
}

impl Default for Composer {
    fn default() -> Self {
        Self::new()
    }
}

#[salsa::query_group(ComponentStorage)]
pub trait Components: Runtime {
    #[salsa::input]
    fn waker(&self) -> Waker;
    #[salsa::input]
    fn top_level_exit(&self) -> AbortHandle;

    // TODO replace this salsa annotation with passing a scope directly
    #[salsa::dependencies]
    fn surface(&self, parent: ScopeId, width: u32, height: u32) -> ();
}

pub trait Runtime: SalsaBowl {
    fn scope(&self, scope: ScopeId) -> Scope;
}

impl Runtime for Composer {
    fn scope(&self, id: ScopeId) -> Scope {
        let mut port = None;

        self.states.alter(id, |prev: Option<Scope>| {
            let current = prev.unwrap_or_else(|| {
                Scope::new(id, self.spawner(), self.waker(), self.top_level_exit())
            });
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
