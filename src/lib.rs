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
        compose::{Component, Compose, Scope, Scopes},
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

pub struct Runtime {
    scopes: Scopes,
    waker: Waker,
    spawner: ThreadPool,
    top_level_exit: AbortHandle,
}

impl Runtime {
    pub async fn go(spawner: ThreadPool, root: impl Component) {
        let (top_level_exit, exit_registration) = AbortHandle::new_pair();

        // make sure we can be woken back up and exited
        let mut waker = None;
        std::future::get_task_waker(|lw| waker = Some(lw.clone()));

        let runtime = Self {
            scopes: Default::default(),
            waker: waker.unwrap(),
            top_level_exit,
            spawner,
        };

        // this returns an error on abort, which is the only time we expect it to return at all
        let _main_compose_loop = await!(Abortable::new(
            async move {
                let root_scope = runtime.scope(caps::ScopeId::root());
                let _ensure_waker_is_set = runtime.waker.clone();
                loop {
                    root_scope.compose(root.clone());
                    // unless we stash our own waker above, we'll never get woken again, be careful
                    pending!();
                }
            },
            exit_registration
        ));
    }

    fn scope(&self, id: caps::ScopeId) -> Scope {
        self.scopes.get(id, self)
    }
}
