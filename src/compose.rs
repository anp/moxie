use {
    crate::{
        caps::{CallsiteId, ScopeId},
        our_prelude::*,
        state::*,
    },
    futures::{executor::ThreadPool, future::AbortHandle, task::Spawn},
    parking_lot::Mutex,
    std::{
        any::Any,
        hash::{Hash, Hasher},
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
        task::Waker,
    },
};

pub trait Compose {
    fn state<S: 'static + Any>(&self, callsite: CallsiteId, f: impl FnOnce() -> S) -> Guard<S>;
    fn task<F>(&self, _callsite: CallsiteId, fut: F)
    where
        F: Future<Output = ()> + Send + 'static;
}

#[derive(Default)]
pub struct Scopes {
    inner: chashmap::CHashMap<ScopeId, Scope>,
}

impl Scopes {
    #[doc(hidden)]
    pub fn get(&self, id: ScopeId, tasker: &impl crate::Runtime) -> Scope {
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
    // parent_revision: Weak<AtomicU64>,
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
        self.spawner.lock().spawn_obj(Box::new(fut).into()).unwrap();
    }
}

impl Hash for Scope {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
        self.revision.load(Ordering::SeqCst).hash(hasher);
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

unsafe impl Send for Scope {}
