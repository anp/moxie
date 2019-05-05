#![deny(clippy::all)]
#![feature(await_macro, async_await, gen_future, weak_ptr_eq)]

#[macro_use]
extern crate rental;

#[macro_use]
mod caps;
mod compose;
mod state;

pub use {
    crate::{
        caps::{CallsiteId, Moniker, ScopeId},
        compose::Scope,
        state::{Guard, Handle},
    },
    mox::props,
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
    std::{
        fmt::Debug,
        panic::{catch_unwind, AssertUnwindSafe},
    },
};

pub trait Component: Clone + std::fmt::Debug + Eq + PartialEq + typename::TypeName {
    fn compose(scp: Scope, props: Self);
}

pub trait Runtime: Send + Sized + 'static {
    type Spawner: ComponentSpawn + 'static;
    fn spawner(&self) -> Self::Spawner;

    fn spawn_self<C: Component + 'static>(self, root: C) {
        let mut spawner = self.spawner();
        spawner.spawn_local(self.composer(root)).unwrap();
    }

    fn composer<C: Component + 'static>(self, root: C) -> LocalFutureObj<'static, ()> {
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
                            mox! { root_scope <- root };
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

pub trait Witness: Debug + Downcast + 'static {
    type Node: Debug + 'static;
    fn see(&mut self, scope: ScopeId, parent: ScopeId, node: &Self::Node);
}

impl_downcast!(Witness assoc Node where Node: Debug + 'static);

pub trait ComponentSpawn {
    fn spawn_local(&mut self, fut: LocalFutureObj<'static, ()>) -> Result<(), SpawnError>;
    fn child(&self) -> Box<dyn ComponentSpawn>;
}

impl<Exec> ComponentSpawn for Exec
where
    Exec: Clone + LocalSpawn + 'static,
{
    fn spawn_local(&mut self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        LocalSpawn::spawn_local_obj(self, future)
    }

    fn child(&self) -> Box<dyn ComponentSpawn> {
        Box::new(self.clone())
    }
}
