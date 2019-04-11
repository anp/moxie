#![deny(clippy::all)]
#![allow(clippy::unused_unit)]
#![feature(
    await_macro,
    futures_api,
    async_await,
    integer_atomics,
    gen_future,
    trait_alias,
    weak_ptr_eq
)]

#[macro_use]
extern crate rental;

#[macro_use]
mod caps;
mod channel;
mod compose;
mod state;

pub use {
    crate::{
        caps::{CallsiteId, Moniker, ScopeId},
        channel::{channel, Sender},
        compose::{Component, Compose, Scope, Witness},
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
        tokio_trace::{debug, error, field, info, span, trace, warn, Level},
    };
}

use {
    crate::our_prelude::*,
    futures::{
        executor::ThreadPool,
        future::{AbortHandle, Abortable},
        pending,
    },
    std::panic::{catch_unwind, AssertUnwindSafe},
};

pub struct Runtime;

impl Runtime {
    pub async fn go(spawner: ThreadPool, root: impl Component) {
        let (top_level_exit, exit_registration) = AbortHandle::new_pair();

        // make sure we can be woken back up and exited
        let mut waker = None;
        std::future::get_task_waker(|lw| waker = Some(lw.clone()));
        let waker = waker.unwrap();

        let root_scope = Scope::root(spawner, waker, top_level_exit);

        // this returns an error on abort, which is the only time we expect it to return at all
        // so we'll just ignore the return value
        let _main_compose_loop = await!(Abortable::new(
            async move {
                loop {
                    let root_scope = AssertUnwindSafe(&root_scope);
                    let root = AssertUnwindSafe(&root);
                    if let Err(e) = catch_unwind(move || {
                        let root_scope = root_scope.clone();
                        let root = root.clone();
                        trace!("composing");
                        mox! { root_scope <- root };
                    }) {
                        error!("error composing: {:?}", e);
                        // TODO soft restart (reset state, recordings, etc)
                    }
                    pending!();
                }
            },
            exit_registration
        ));
    }
}
