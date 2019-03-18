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
        collections::HashMap,
        hash::{Hash, Hasher},
        panic::AssertUnwindSafe,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, Weak,
        },
        task::Waker,
    },
};

pub trait Component: Clone + std::fmt::Debug + Eq + Hash + PartialEq {
    fn compose(scp: Scope, props: Self);
}

pub trait Compose {
    fn compose_child<C: Component>(&self, id: ScopeId, props: C);
    fn state<S: 'static + Any>(&self, callsite: CallsiteId, f: impl FnOnce() -> S) -> Guard<S>;
    fn task<F>(&self, _callsite: CallsiteId, fut: F)
    where
        F: Future<Output = ()> + Send + 'static;
}

/// Provides a component with access to the persistent state store and futures executor.
#[derive(Clone, Debug)]
pub struct Scope {
    inner: Arc<InnerScope>,
}

#[derive(Clone, Debug)]
struct WeakScope {
    inner: Weak<InnerScope>,
}

impl Scope {
    pub fn id(&self) -> ScopeId {
        self.inner.id
    }

    pub(crate) fn root(spawner: ThreadPool, waker: Waker, exit: AbortHandle) -> Self {
        let new = Self {
            inner: Arc::new(InnerScope {
                id: ScopeId::root(),
                revision: Arc::new(AtomicU64::new(0)),
                exit,
                waker,
                spawner: Mutex::new(spawner),
                states: Default::default(),
                parent: None,
                children: Default::default(),
            }),
        };

        debug!("created root scope with id {:?}", new.id());

        new
    }

    fn child(&self, id: ScopeId) -> Self {
        let inner = &self.inner;

        // FIXME garbage collect?
        inner
            .children
            .lock()
            .entry(id)
            .or_insert_with(|| {
                let parent = Some(self.weak());

                Self {
                    inner: Arc::new(InnerScope {
                        id,
                        revision: Arc::new(AtomicU64::new(0)),
                        exit: inner.exit.clone(),
                        waker: inner.waker.clone(),
                        spawner: Mutex::new(inner.spawner.lock().clone()),
                        states: Default::default(),
                        parent,
                        children: Mutex::new(Default::default()),
                    }),
                }
            })
            .clone()
    }

    fn weak(&self) -> WeakScope {
        WeakScope {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn waker(&self) -> Waker {
        self.inner.waker.clone()
    }

    pub fn top_level_exit_handle(&self) -> AbortHandle {
        self.inner.exit.clone()
    }
}

impl Compose for Scope {
    #[inline]
    fn compose_child<C: Component>(&self, id: ScopeId, props: C) {
        C::compose(self.child(id), props)
    }

    #[inline]
    fn state<S: 'static + Any>(&self, callsite: CallsiteId, f: impl FnOnce() -> S) -> Guard<S> {
        self.inner.states.get_or_init(callsite, f)
    }

    #[inline]
    fn task<F>(&self, _callsite: CallsiteId, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        debug!("acquiring lock");
        self.inner
            .spawner
            .lock()
            .spawn_obj(
                Box::new(AssertUnwindSafe(fut).catch_unwind().map(|r| {
                    if let Err(e) = r {
                        error!("user code panicked: {:?}", e);
                    }
                }))
                .into(),
            )
            .unwrap();
    }
}

#[derive(Debug)]
struct InnerScope {
    pub id: ScopeId,
    pub revision: Arc<AtomicU64>,
    parent: Option<WeakScope>,
    states: States,
    children: Mutex<HashMap<ScopeId, Scope>>,

    spawner: Mutex<ThreadPool>,
    waker: Waker,
    exit: AbortHandle,
}

impl Drop for InnerScope {
    fn drop(&mut self) {
        trace!("inner scope dropping: {:?}", self);
    }
}

impl Hash for InnerScope {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
        self.revision.load(Ordering::SeqCst).hash(hasher);
    }
}

impl PartialEq for InnerScope {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.revision.load(Ordering::SeqCst) == other.revision.load(Ordering::SeqCst)
            && self.states == other.states
    }
}

impl Eq for InnerScope {}

unsafe impl Send for InnerScope {}
