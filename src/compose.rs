use {
    crate::{our_prelude::*, state::*, CallsiteId, ScopeId},
    futures::{executor::ThreadPool, future::AbortHandle, task::Spawn},
    parking_lot::Mutex,
    std::{
        any::Any,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
        task::Waker,
    },
};

pub trait Compose {
    // FIXME offer a thread-local state so this can be Send again?
    fn state<S: 'static + Any>(&self, callsite: CallsiteId, f: impl FnOnce() -> S) -> Guard<S>;
    fn task<F>(&self, _callsite: CallsiteId, fut: F)
    where
        F: Future<Output = ()> + Send + 'static;
    // TODO define `try_task` method too, for potentially fallible tasks?
    // what should the behavior on error be then? emitting some error event?
    // could reset the component state, treat it like a redraw of this subtree?
    // maybe have some `Fallible` component?
}

#[derive(Default)]
pub struct Scopes {
    inner: chashmap::CHashMap<ScopeId, Scope>,
}

impl Scopes {
    pub(crate) fn get(&self, id: ScopeId, tasker: &impl crate::Runtime) -> Scope {
        let mut port = None;

        self.inner.alter(id, |prev: Option<Scope>| {
            let current = prev.unwrap_or_else(|| {
                Scope::new(
                    id,
                    tasker.spawner(),
                    tasker.waker(),
                    tasker.top_level_exit(),
                )
            });
            port = Some(current.clone());
            Some(current)
        });

        port.unwrap()
    }
}

/// Provides a component with access to the persistent state store and futures executor.
///
/// Because `salsa` does not yet support generic queries, we need a concrete type that can be
/// passed as an argument and tracked within the incremental computation system.
#[derive(Clone, Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub revision: Arc<AtomicU64>,
    states: States,

    spawner: Arc<Mutex<ThreadPool>>,
    waker: Waker,
    exit: AbortHandle,
}

impl Scope {
    pub(crate) fn new(id: ScopeId, spawner: ThreadPool, waker: Waker, exit: AbortHandle) -> Self {
        Self {
            id,
            revision: Arc::new(AtomicU64::new(0)),
            exit,
            waker,
            spawner: Arc::new(Mutex::new(spawner)),
            states: Default::default(),
        }
    }

    pub fn waker(&self) -> Waker {
        self.waker.clone()
    }

    pub fn top_level_exit_handle(&self) -> AbortHandle {
        self.exit.clone()
    }
}

impl Compose for Scope {
    #[inline]
    fn state<S: 'static + Any>(&self, callsite: CallsiteId, f: impl FnOnce() -> S) -> Guard<S> {
        self.states.get_or_init(callsite, f)
    }

    #[inline]
    fn task<F>(&self, _callsite: CallsiteId, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // TODO make sure we only have a single task for this callsite at a time
        // TODO tie the span of this task's execution to the scope
        // TODO catch panics and abort runtime?
        self.spawner.lock().spawn_obj(Box::new(fut).into()).unwrap();
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.revision.load(Ordering::SeqCst) == other.revision.load(Ordering::SeqCst)
            && self.states == other.states
    }
}

impl Eq for Scope {}

// FIXME this is rly bad yall
unsafe impl Send for Scope {}
