#![deny(clippy::all)]
#![feature(await_macro, async_await, gen_future, weak_ptr_eq)]

#[macro_use]
extern crate rental;

#[macro_use]
mod caps;
mod channel;
mod compose;
mod spawn;
mod state;

pub use {
    crate::{
        caps::{CallsiteId, Moniker, ScopeId},
        channel::{channel, Sender},
        compose::{Component, Scope, Witness},
        spawn::PrioritySpawn,
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
    futures::future::{AbortHandle, Abortable},
    std::panic::{catch_unwind, AssertUnwindSafe},
};

pub struct Runtime;

impl Runtime {
    pub fn go(spawner: impl PrioritySpawn + 'static, root: impl Component + 'static) {
        let (top_level_exit, exit_registration) = AbortHandle::new_pair();
        let mut root_spawner = spawner.child();
        root_spawner
            .spawn_local(
                Box::new(
                    Abortable::new(
                        async move {
                            // make sure we can be woken back up and exited
                            let waker = {
                                let mut waker = None;
                                std::future::get_task_context(|cx| {
                                    waker = Some(cx.waker().clone())
                                });
                                waker.unwrap()
                            };

                            let root_scope = Scope::root(spawner, waker, top_level_exit);

                            loop {
                                let root_scope = AssertUnwindSafe(&root_scope);
                                let root = root.clone();
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
                .into(),
            )
            .unwrap();
    }
}
