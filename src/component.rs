use {
    crate::{prelude::*, state::*},
    futures::future::AbortHandle,
    parking_lot::{Mutex, MutexGuard},
    std::{
        any::Any,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
    },
};

pub trait Compose {
    fn state<S: 'static + Any>(&self, callsite: CallsiteId, f: impl FnOnce() -> S) -> Guard<S>;
    fn task<F>(&self, _callsite: CallsiteId, fut: F)
    where
        F: Future<Output = ()> + Send + 'static;
}

// pub trait Compose {
//     // FIXME offer a thread-local state so this can be Send again?
//     fn state<S: 'static + Any>(&self, callsite: CallsiteId, f: impl FnOnce() -> S) -> Guard<S>;
//     fn task<F>(&self, _callsite: CallsiteId, fut: F)
//     where
//         F: Future<Output = ()> + Send + 'static;
//     // TODO define `try_task` method too, for potentially fallible tasks?
//     // what should the behavior on error be then? emitting some error event?
//     // could reset the component state, treat it like a redraw of this subtree?
//     // maybe have some `Fallible` component?

// }

/// Provides a component with access to the persistent state store and futures executor.
///
/// Because `salsa` does not yet support generic queries, we need a concrete type that can be
/// passed as an argument and tracked within the incremental computation system.
#[derive(Clone, Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub revision: Arc<Revision>,
    states: States,
    spawner: Arc<Mutex<crate::Spawner>>,
    waker: Waker,
    exit: AbortHandle,
}

impl Scope {
    pub(crate) fn new(
        id: ScopeId,
        spawner: crate::Spawner,
        waker: Waker,
        exit: AbortHandle,
    ) -> Self {
        Self {
            id,
            revision: Arc::new(Revision::current()),
            exit,
            waker,
            spawner: Arc::new(Mutex::new(spawner)),
            states: Default::default(),
        }
    }

    pub(crate) fn waker(&self) -> Waker {
        self.waker.clone()
    }

    pub(crate) fn top_level_exit_handle(&self) -> AbortHandle {
        self.exit.clone()
    }
}

impl Compose for Scope {
    #[inline]
    fn state<S: 'static + Any>(&self, callsite: CallsiteId, f: impl FnOnce() -> S) -> Guard<S> {
        let mut cell = None;

        self.states.alter(callsite, |prev| {
            if let Some(p) = prev {
                cell = Some(p);
            } else {
                let initialized = f();
                cell = Some(Arc::new(StateCell::new(
                    Arc::downgrade(&self.revision),
                    initialized,
                )));
            }
            cell.clone()
        });

        let cell = cell.unwrap();

        Guard {
            cell: cell.downgrade(),
            rented: crate::state::RentedGuard::new(cell, |cell| {
                MutexGuard::map((*cell).0.contents.lock(), |any| any.downcast_mut().unwrap())
            }),
        }
    }

    fn task<F>(&self, _callsite: CallsiteId, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // TODO tie the span of this task's execution to the scope
        // TODO catch panics and abort runtime?
        use futures::task::Spawn;
        (*self.spawner.lock())
            .spawn_obj(Box::new(fut).into())
            .unwrap();
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.states, &other.states)
    }
}

impl Eq for Scope {}

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
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Moniker(usize);

impl Moniker {
    #[doc(hidden)]
    #[inline]
    pub fn new(scope: ScopeId, callsite: &'static str) -> Self {
        Moniker(fxhash::hash(&(scope, callsite)))
    }
}

macro_rules! moniker {
    ($parent:expr) => {
        $crate::prelude::Moniker::new($parent, concat!(file!(), "@", line!(), ":", column!()))
    };
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopeId(Moniker);

impl ScopeId {
    pub fn new(callsite: Moniker) -> Self {
        Self(callsite)
    }

    pub(crate) fn root() -> Self {
        Self(Moniker(fxhash::hash(&0)))
    }
}

macro_rules! scope {
    () => {
        $crate::prelude::ScopeId::root()
    };
    ($parent:expr) => {
        $crate::prelude::ScopeId::new(moniker!($parent))
    };
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallsiteId {
    site: Moniker,
    scope: ScopeId,
}

impl CallsiteId {
    pub fn new(scope: ScopeId, site: Moniker) -> Self {
        Self { scope, site }
    }
}

macro_rules! callsite {
    ($parent:expr) => {
        $crate::prelude::CallsiteId::new($parent, moniker!($parent))
    };
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(u64);

static CURRENT_REVISION: AtomicU64 = AtomicU64::new(0);

impl Revision {
    // /// Get the current revision, or "tick". Every event recieved advances the revision by 1, so
    // /// not all revisions will cause a recomposition to be executed, and so an even smaller number
    // /// will cause a new frame to be generated.
    pub fn current() -> Self {
        Self(CURRENT_REVISION.load(Ordering::Relaxed))
    }

    /// Get the next revision, advancing the global revision counter by 1.
    ///
    /// Note: this is private because user-defined code should be able to percieve the passage of
    /// time, but only the event system should be able to drive it forward.
    pub(crate) fn next() -> Self {
        Self(CURRENT_REVISION.fetch_add(1, Ordering::Relaxed))
    }
}
