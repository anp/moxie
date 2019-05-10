#![deny(clippy::all)]
#![feature(await_macro, async_await, gen_future, weak_ptr_eq)]

#[macro_use]
extern crate rental;

#[macro_use]
mod caps;
mod record;
mod scope;
mod state;

pub use crate::{
    caps::{CallsiteId, Moniker, ScopeId},
    scope::Scope,
    state::{Guard, Key},
};

#[doc(hidden)]
pub use typename;

pub(crate) mod our_prelude {
    pub use {
        futures::{
            future::{Aborted, FutureExt, FutureObj},
            stream::{Stream, StreamExt},
            task::Spawn,
        },
        parking_lot::Mutex,
        std::{future::Future, sync::Arc, task::Waker},
        tokio_trace::*,
    };
}

use {
    crate::our_prelude::*,
    downcast_rs::*,
    futures::{
        future::{AbortHandle, Abortable, LocalFutureObj},
        task::{LocalSpawn, SpawnError},
    },
    std::panic::{catch_unwind, AssertUnwindSafe},
};

pub trait Component {
    fn run(self, scp: Scope);
}

pub trait Witness: 'static + Downcast {
    type Node: 'static;
    fn see(&mut self, node: &Self::Node, parent: Option<&Self::Node>);
}
impl_downcast!(Witness assoc Node where Node: 'static);

pub trait Runtime: 'static + Send + Sized {
    type Spawner: ComponentSpawn;
    fn spawner(&self) -> Self::Spawner;

    fn spawn_self<C: Component + Clone + 'static>(self, root: C) {
        let mut spawner = self.spawner();
        spawner.spawn_local(self.composer(root)).unwrap();
    }

    fn composer<C: Component + Clone + 'static>(self, root: C) -> LocalFutureObj<'static, ()> {
        let (top_level_exit, exit_registration) = AbortHandle::new_pair();
        Box::new(
            Abortable::new(
                async move {
                    // make sure we can be woken back up and exited
                    let waker = {
                        let mut waker = None;
                        std::future::get_task_context(|cx| waker = Some(cx.waker().clone()));
                        waker.unwrap()
                    };

                    let root_scope = Scope::root(self.spawner(), waker, top_level_exit);

                    loop {
                        let root_scope = AssertUnwindSafe(&root_scope);
                        let root = AssertUnwindSafe(root.clone());
                        if let Err(e) = catch_unwind(move || {
                            let root_scope = root_scope.clone();
                            let root = root.clone();
                            trace!("composing");
                            run! { root_scope <- root };
                        }) {
                            error!("error composing: {:?}", e);
                            // TODO soft restart (reset state, recordings, etc)
                        }
                        futures::pending!();
                    }
                },
                exit_registration,
            )
            .map(|_| ()),
        )
        .into()
    }
}

pub trait ComponentSpawn: 'static {
    fn spawn_local(&mut self, fut: LocalFutureObj<'static, ()>) -> Result<(), SpawnError>;
    fn child(&self) -> Box<dyn ComponentSpawn>;
}

impl<Exec> ComponentSpawn for Exec
where
    Exec: 'static + Clone + LocalSpawn,
{
    fn spawn_local(&mut self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        LocalSpawn::spawn_local_obj(self, future)
    }

    fn child(&self) -> Box<dyn ComponentSpawn> {
        Box::new(self.clone())
    }
}
