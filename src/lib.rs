#![deny(clippy::all)]
#![allow(clippy::unused_unit)]
#![feature(await_macro, futures_api, async_await, integer_atomics, gen_future)]

#[macro_use]
extern crate rental;

pub mod compose;
pub mod state;

pub use {
    crate::{
        compose::{Compose, Scope},
        state::{Guard, Handle},
    },
    futures::{
        future::FutureExt,
        stream::{Stream, StreamExt},
    },
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
    crate::{compose::Scopes, our_prelude::*},
    futures::{
        executor::ThreadPool,
        future::{AbortHandle, Abortable},
        pending,
    },
};

pub trait Runtime: TaskBootstrapper + Send + 'static {
    fn scopes(&self) -> &Scopes;

    fn scope(&self, id: ScopeId) -> Scope {
        self.scopes().get(id, self)
    }
}

// TODO make this a trait method when impl trait in trait methods works
pub async fn run<'a, ThisRuntime>(
    runtime: ThisRuntime,
    spawner: ThreadPool,
    root_component: fn(&'a ThisRuntime, ScopeId),
) where
    ThisRuntime: Runtime + Unpin + 'static,
{
    let mut runtime = Box::new(runtime);
    let (exit_handle, exit_registration) = AbortHandle::new_pair();

    let main_compose_loop = async move {
        pin_utils::pin_mut!(runtime);
        // make sure we can be woken back up and exited
        let mut waker = None;
        std::future::get_task_waker(|lw| waker = Some(lw.clone()));
        runtime.set_waker(waker.unwrap().into());
        runtime.set_top_level_exit(exit_handle);
        runtime.set_spawner(spawner);

        loop {
            // root_component(&runtime, ScopeId::root());
            // unless we stash our own waker above, we'll never get woken again, be careful
            pending!();
        }
    };

    await!(Abortable::new(main_compose_loop, exit_registration).map(|r| r.unwrap_or(())))
}

#[salsa::query_group(Moxie)]
pub trait TaskBootstrapper: salsa::Database {
    #[salsa::input]
    fn waker(&self) -> Waker;
    #[salsa::input]
    fn spawner(&self) -> ThreadPool;
    #[salsa::input]
    fn top_level_exit(&self) -> AbortHandle;
}

#[salsa::database(Moxie)]
#[derive(Default)]
struct TestRuntime {
    runtime: salsa::Runtime<Self>,
    scopes: Scopes,
}

impl Runtime for TestRuntime {
    fn scopes(&self) -> &Scopes {
        &self.scopes
    }
}

impl salsa::Database for TestRuntime {
    fn salsa_runtime(&self) -> &salsa::Runtime<Self> {
        &self.runtime
    }
}

/// A `Moniker` represents the coordinates of a code location in the render hierarchy.
///
/// The struct describes a location in the program specific to:
///
/// * a line and column of code,
/// * in a particular element function,
/// * TODO: on a particular round of iteration (straight line code always has a single round),
/// * as well as the moniker which resulted in that particular function's invocation
///
/// It can be derived at any point within any element as long as the parent/invoking/enclosing
/// moniker is available. We guarantee that it's always available in render lifecycle in other ways.
///
/// `Moniker`s are the tool underlying elements, state, context, etc. because they allow us to map
/// from a "pure" function back to a state location.
// TODO: there should probably be an actual Moniker capability that encloses one, right?
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Moniker(usize);

impl Moniker {
    #[doc(hidden)]
    #[inline]
    pub fn new(scope: ScopeId, callsite: &'static str) -> Self {
        Moniker(fxhash::hash(&(scope, callsite)))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! moniker {
    ($parent:expr) => {
        $crate::Moniker::new($parent, concat!(file!(), "@", line!(), ":", column!()))
    };
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopeId(Moniker);

impl ScopeId {
    #[doc(hidden)]
    pub fn new(callsite: Moniker) -> Self {
        Self(callsite)
    }

    pub(crate) fn root() -> Self {
        Self(Moniker(fxhash::hash(&0)))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! scope {
    ($parent:expr) => {
        $crate::ScopeId::new($crate::moniker!($parent))
    };
}

#[macro_export]
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallsiteId {
    scope: ScopeId,
    site: Moniker,
}

impl CallsiteId {
    #[doc(hidden)]
    pub fn new(scope: ScopeId, site: Moniker) -> Self {
        Self { scope, site }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! callsite {
    ($parent:expr) => {
        $crate::CallsiteId::new($parent, $crate::moniker!($parent))
    };
}
