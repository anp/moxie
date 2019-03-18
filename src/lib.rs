#![deny(clippy::all)]
#![allow(clippy::unused_unit)]
#![feature(await_macro, futures_api, async_await, integer_atomics, gen_future)]

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
        compose::{Component, Compose, Scope},
        state::{Guard, Handle},
    },
    futures::{
        future::FutureExt,
        stream::{Stream, StreamExt},
    },
    mox::props,
    std::future::Future,
};

pub(crate) mod our_prelude {
    pub use {
        futures::{
            future::{Aborted, FutureExt, FutureObj},
            task::Spawn,
        },
        log::{debug, error, info, trace, warn},
        parking_lot::Mutex,
        std::{future::Future, sync::Arc, task::Waker},
    };
}

use {
    crate::our_prelude::*,
    futures::{
        executor::ThreadPool,
        future::{AbortHandle, Abortable},
        pending,
    },
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
                    Component::compose(root_scope.clone(), root.clone());
                    pending!();
                }
            },
            exit_registration
        ));
    }
}
