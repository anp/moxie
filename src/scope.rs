use {
    crate::{
        caps::{CallsiteId, ScopeId},
        our_prelude::*,
        record::{Recorder, Records},
        state::*,
        Component, ComponentScope, ComponentSpawn, Witness,
    },
    futures::future::AbortHandle,
    parking_lot::Mutex,
    std::{
        any::Any,
        collections::HashMap,
        fmt::{Debug, Formatter, Result as FmtResult},
        hash::{Hash, Hasher},
        panic::{AssertUnwindSafe, UnwindSafe},
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, Weak,
        },
        task::Waker,
    },
};

/// Provides a component with access to the persistent state store and futures executor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope(pub(crate) Arc<InnerScope>);

impl ComponentScope for Scope {
    fn compose_child<C: Component>(&self, id: ScopeId, props: C) {
        span!(
            tokio_trace::Level::TRACE,
            "compose_child",
            id = field::debug(&id),
            // name = field::display(&C::type_name()),
        )
        .enter(|| {
            let child = self.child(id);
            {
                let child = child.clone();
                child.prepare_to_compose();
                props.run(child);
            }
            child.finish_composition();
        })
    }

    fn compose_child_with_witness<C, W>(&self, child_id: ScopeId, props: C, witness: W) -> W
    where
        C: Component,
        W: Witness,
    {
        let child_scope = self.child(child_id);
        child_scope.install_witness(witness);
        self.compose_child(child_id, props);
        child_scope.remove_witness().unwrap()
    }

    fn state<S: 'static + Any + UnwindSafe>(
        &self,
        callsite: CallsiteId,
        f: impl FnOnce() -> S,
    ) -> Guard<S> {
        self.0.states.get_or_init(self.weak(), callsite, f)
    }

    fn task<F>(&self, _callsite: CallsiteId, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.0
            .spawner
            .lock()
            .spawn_local(
                Box::new(AssertUnwindSafe(fut).catch_unwind().map(|r| {
                    if let Err(e) = r {
                        error!({ error = field::debug(&e) }, "user code panicked");
                    }
                }))
                .into(),
            )
            .unwrap();
    }

    fn record<N>(&self, node: N)
    where
        N: 'static,
    {
        self.0.records.record(node, self.weak());
    }

    fn install_witness<W>(&self, witness: W)
    where
        W: Witness,
    {
        self.0.records.install_witness(witness);
    }

    fn remove_witness<W>(&self) -> Option<W>
    where
        W: Witness,
    {
        self.0.records.remove_witness()
    }
}

impl Scope {
    pub fn id(&self) -> ScopeId {
        self.0.id
    }

    pub(crate) fn root<Spawner>(spawner: Spawner, waker: Waker, exit: AbortHandle) -> Self
    where
        Spawner: ComponentSpawn + 'static,
    {
        Self(Arc::new(InnerScope {
            id: ScopeId::root(),
            revision: Arc::new(AtomicU64::new(0)),
            spawner: Mutex::new(Box::new(spawner)),
            parent: None,
            states: Default::default(),
            children: Default::default(),
            bind_order: Default::default(),
            records: Default::default(),
            exit,
            waker,
        }))
    }

    pub fn child(&self, id: ScopeId) -> Self {
        let inner = &self.0;

        inner
            .children
            .lock()
            .entry(id)
            .or_insert_with(|| {
                let parent = Some(self.weak());
                self.0.bind_order.lock().push(id);

                Self(Arc::new(InnerScope {
                    id,
                    revision: Arc::new(AtomicU64::new(0)),
                    exit: inner.exit.clone(),
                    waker: inner.waker.clone(),
                    spawner: Mutex::new(inner.spawner.lock().child()),
                    records: self.0.records.child(),
                    parent,
                    bind_order: Default::default(),
                    children: Default::default(),
                    states: Default::default(),
                }))
            })
            .clone()
    }

    fn weak(&self) -> WeakScope {
        WeakScope {
            inner: Arc::downgrade(&self.0),
        }
    }

    fn tick(&self) {
        let mut to_tick = Some(self.to_owned());
        while let Some(ref mut inner) = &mut to_tick {
            inner.0.tick();
            to_tick = inner.0.parent.as_ref().and_then(WeakScope::upgrade);
        }
    }

    pub fn top_level_exit_handle(&self) -> AbortHandle {
        self.0.exit.clone()
    }

    fn prepare_to_compose(&self) {
        self.0.bind_order.lock().clear();
        self.0
            .records
            .for_each_storage(Records::flush_before_composition);
    }

    fn finish_composition(&self) {
        // TODO garbage collect state, children, and tasks
        self.0
            .records
            .for_each_storage(|records| records.show_witnesses_after_composition(self.clone()));
    }
}

pub(crate) struct InnerScope {
    pub id: ScopeId,
    pub revision: Arc<AtomicU64>,
    pub parent: Option<WeakScope>,
    pub states: States,
    pub children: Mutex<HashMap<ScopeId, Scope>>,
    pub bind_order: Mutex<Vec<ScopeId>>,
    pub records: Recorder,
    pub spawner: Mutex<Box<dyn ComponentSpawn>>,
    pub waker: Waker,
    pub exit: AbortHandle,
}

impl InnerScope {
    pub(crate) fn tick(&self) {
        self.revision.fetch_add(1, Ordering::SeqCst);
        self.waker.clone().wake();
    }
}

impl Debug for InnerScope {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_tuple("InnerScope")
            .field(&self.id)
            .field(&self.revision)
            .finish()
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

#[derive(Clone, Debug)]
pub(crate) struct WeakScope {
    pub inner: Weak<InnerScope>,
}

impl WeakScope {
    pub fn upgrade(&self) -> Option<Scope> {
        self.inner.upgrade().map(Scope)
    }

    pub fn tick(&self) {
        if let Some(inner) = self.upgrade() {
            inner.tick();
        }
    }
}
