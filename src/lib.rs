#![deny(clippy::all)]
#![allow(clippy::unused_unit)]
#![feature(await_macro, futures_api, async_await, integer_atomics, gen_future)]

#[macro_use]
extern crate rental;

#[macro_use]
pub mod component;
mod drop_guard;
mod events;
pub mod state;
mod surface;

// TODO split into public/private preludes
pub mod prelude {
    pub use {
        crate::{
            component::{CallsiteId, Compose, Moniker, Scope, ScopeId},
            state::{Guard, Handle},
            surface::surface,
            {Components, Runtime},
        },
        futures::{
            future::{Aborted, FutureExt},
            stream::{Stream, StreamExt},
            task::Spawn,
        },
        log::{debug, error, info, trace, warn},
        parking_lot::Mutex,
        std::{future::Future, sync::Arc, task::Waker},
    };
}

use {
    crate::{component::Scope, prelude::*},
    chashmap::CHashMap,
    futures::{
        future::{AbortHandle, Abortable, FutureObj},
        pending,
    },
    salsa::Database as SalsaBowl,
};

/// A `Composer` is the primary entry point to moxie's runtime systems. It contains the salsa
/// incremental storage, a futures executor, interners, and is passed to every composable function.
#[salsa::database(ComponentStorage)]
pub struct Runtime {
    runtime: salsa::Runtime<Self>,
    states: CHashMap<ScopeId, Scope>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            runtime: salsa::Runtime::default(),
            states: CHashMap::default(),
        }
    }

    pub async fn run(self, task_spawner: impl Spawn + Send + 'static) {
        let (exit_handle, exit_registration) = AbortHandle::new_pair();
        let _ = await!(Abortable::new(
            async {
                let mut compose = self;
                // make sure we can be woken back up
                std::future::get_task_waker(|lw| compose.set_waker(lw.clone().into()));
                // make sure we can be exited
                compose.set_top_level_exit(exit_handle);
                compose.set_spawner(Spawner::new(task_spawner));

                loop {
                    trace!("composing surface");
                    compose.surface(scope!(), 1920, 1080);

                    // unless we stash our own waker above, we'll never get woken again, be careful
                    pending!();
                }
            },
            exit_registration,
        ));
        info!("main runtime loop has ended");
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[salsa::query_group(ComponentStorage)]
pub trait Components: ComponentRuntime {
    #[salsa::input]
    fn waker(&self) -> Waker;
    #[salsa::input]
    fn spawner(&self) -> Spawner;
    #[salsa::input]
    fn top_level_exit(&self) -> AbortHandle;

    // TODO replace this salsa annotation with passing a scope directly
    #[salsa::dependencies]
    fn surface(&self, parent: ScopeId, width: u32, height: u32) -> ();
}

pub trait ComponentRuntime: SalsaBowl {
    fn scope(&self, scope: ScopeId) -> Scope;
}

impl ComponentRuntime for Runtime {
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

impl SalsaBowl for Runtime {
    fn salsa_runtime(&self) -> &salsa::Runtime<Self> {
        &self.runtime
    }
}

#[derive(Clone)]
pub struct Spawner(Arc<Mutex<dyn Spawn + Send + 'static>>);

impl Spawner {
    fn new(s: impl Spawn + Send + 'static) -> Self {
        Spawner(Arc::new(Mutex::new(s)))
    }
}

impl std::fmt::Debug for Spawner {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "spawner")
    }
}

impl futures::task::Spawn for Spawner {
    fn spawn_obj(&mut self, fut: FutureObj<'static, ()>) -> Result<(), futures::task::SpawnError> {
        self.0.lock().spawn_obj(fut)
    }
}
