use {
    crate::{
        caps::{CallsiteId, ScopeId},
        our_prelude::*,
        state::*,
    },
    downcast_rs::*,
    futures::{executor::ThreadPool, future::AbortHandle, task::Spawn},
    parking_lot::Mutex,
    std::{
        any::{Any, TypeId},
        collections::HashMap,
        fmt::Debug,
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

    fn record<Node>(&self, node: Node)
    where
        Node: Recorded;

    /// Install a `Witness` into this scope. Each witness is responsible for consuming recorded
    /// nodes of a single type.
    fn install_witness<W>(&self, witness: W)
    where
        W: Witness + Clone + 'static;

    fn remove_witness<W>(&self) -> Option<W>
    where
        W: Witness + Clone + 'static;
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
                bind_order: Default::default(),
                records: Default::default(),
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
                self.inner.bind_order.lock().push(id);

                Self {
                    inner: Arc::new(InnerScope {
                        id,
                        revision: Arc::new(AtomicU64::new(0)),
                        exit: inner.exit.clone(),
                        waker: inner.waker.clone(),
                        spawner: Mutex::new(inner.spawner.lock().clone()),
                        states: Default::default(),
                        parent,
                        children: Default::default(),
                        bind_order: Default::default(),
                        records: Default::default(),
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

    fn prepare_to_compose(&self) {
        self.inner.bind_order.lock().clear();
        self.for_each_record_storage(Records::flush_before_composition);
    }

    fn finish_composition(&self) {
        // TODO garbage collect state, children, and tasks
        self.for_each_record_storage(|records| {
            records.show_witnesses_after_composition(self.clone())
        })
    }
}

impl Compose for Scope {
    #[inline]
    fn compose_child<C: Component>(&self, id: ScopeId, props: C) {
        let child = self.child(id);

        // TODO only run if things have changed
        {
            let child = child.clone();
            child.prepare_to_compose();
            C::compose(child, props);
        }

        child.finish_composition();
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
        trace!("acquiring lock");
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

    #[inline]
    fn record<N>(&self, node: N)
    where
        N: Recorded,
    {
        self.with_record_storage(|storage| {
            storage.records.push(node);
        });
    }

    fn install_witness<W>(&self, witness: W)
    where
        W: Witness,
    {
        self.with_record_storage(|storage: &mut RecordStorage<W::Node>| {
            storage
                .witnesses
                .insert(TypeId::of::<W>(), Box::new(witness))
        });
    }

    fn remove_witness<W>(&self) -> Option<W>
    where
        W: Clone + Witness + 'static,
    {
        self.with_record_storage(|storage: &mut RecordStorage<W::Node>| {
            storage.witnesses.remove(&TypeId::of::<W>())
        })
        .map(Downcast::into_any)
        .map(|any: Box<std::any::Any>| any.downcast().unwrap())
        .map(|boxed: Box<W>| *boxed)
    }
}

#[derive(Debug)]
struct InnerScope {
    pub id: ScopeId,
    pub revision: Arc<AtomicU64>,
    parent: Option<WeakScope>,
    states: States,
    children: Mutex<HashMap<ScopeId, Scope>>,
    bind_order: Mutex<Vec<ScopeId>>,
    records: Mutex<HashMap<TypeId, Box<dyn Records>>>,

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

/// A `Witness` is a generic implementor of a close analogy to React's reconciliation/commit phase.
/// (TODO better explanation!)
///
/// After a composition, each component has recorded a set of nodes into its scope, and those nodes
/// must be operated on in some backend-specific way to fully realize them within the UI. For
/// example, on the web a `Witness<DomElement>` might be responsible for attaching DOM nodes to
/// their parents appropriately. On a GPU-oriented backend like Webrender, a `Witness<DisplayItem>`
/// might be responsible for creating a single display list from the memoized node fragments of
/// a given composition's components.
pub trait Witness: Debug + Downcast + Send + 'static {
    type Node: Recorded;
    fn see_component(&mut self, id: ScopeId, parent: ScopeId, nodes: &[Self::Node]);
}

impl_downcast!(Witness assoc Node where Node: Recorded);

pub trait Recorded = Debug + Send + Sized + 'static;

impl Scope {
    fn with_record_storage<Node, Ret>(
        &self,
        op: impl FnOnce(&mut RecordStorage<Node>) -> Ret,
    ) -> Ret
    where
        Node: Recorded,
    {
        let mut storage_by_node = self.inner.records.lock();
        #[allow(clippy::borrowed_box)]
        let storage: &mut Box<dyn Records> = storage_by_node
            .entry(TypeId::of::<Node>())
            .or_insert_with(|| {
                let storage: RecordStorage<Node> = RecordStorage::default();
                Box::new(storage)
            });
        let storage: &mut dyn Records = &mut **storage;
        let storage: &mut std::any::Any = storage.as_any_mut();
        let storage: &mut RecordStorage<Node> = storage.downcast_mut().unwrap();

        // not panic-safe, maybe fix?
        op(storage)
    }

    fn for_each_record_storage(&self, op: impl Fn(&mut dyn Records)) {
        self.inner
            .records
            .lock()
            .values_mut()
            .map(|b| &mut **b)
            .for_each(op)
    }
}

#[derive(Debug)]
struct RecordStorage<Node>
where
    Node: Recorded,
{
    records: Vec<Node>,
    witnesses: HashMap<TypeId, Box<dyn Witness<Node = Node>>>,
}

trait Records: Debug + Downcast + Send + 'static {
    /// Clear recorded nodes from storage. Should be called immediately before composing in this
    /// scope.
    fn flush_before_composition(&mut self);

    /// Show the current component hierarchy and associated recordings to all installed witnesses.
    ///
    /// Probably needs a better name. Takes the current scope as an argument so that it can
    /// traverse to children. Vague name, poor API. We'll refactor this another time.
    fn show_witnesses_after_composition(&mut self, scope: Scope);
}
impl_downcast!(Records);

impl<Node> Records for RecordStorage<Node>
where
    Node: Recorded,
{
    fn flush_before_composition(&mut self) {
        self.records.clear();
    }

    fn show_witnesses_after_composition(&mut self, scope: Scope) {
        for witness in self.witnesses.values_mut() {
            let parent = scope
                .inner
                .parent
                .as_ref()
                .and_then(|p| p.inner.upgrade().map(|p| p.id))
                // only the root has a null parent, and we never "see" the root bc it never gets
                // any witnesses installed
                .unwrap();
            witness.see_component(scope.id(), parent, &self.records);

            let children = scope.inner.children.lock();
            for child_scope in children.values() {
                // TODO recurse and show this witness the child nodes
            }
        }
    }
}

impl<Node> Default for RecordStorage<Node>
where
    Node: Recorded,
{
    fn default() -> Self {
        Self {
            records: Default::default(),
            witnesses: Default::default(),
        }
    }
}
