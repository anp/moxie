use {
    crate::*,
    downcast_rs::*,
    std::{any::TypeId, collections::HashMap, fmt::Debug},
};

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

/// A Recorder is responsible for capturing the recorded side effects of components, and replaying
/// them to witnesses afterwards in component bind order, regardless of composition execution
/// order.
#[derive(Debug, Default)]
pub struct Recorder {
    storage_by_node: Mutex<HashMap<TypeId, Box<dyn Records>>>,
}

impl Recorder {
    pub fn record<Node: Recorded>(&self, node: Node) {
        self.with_storage(|storage| {
            storage.records.push(node);
        });
    }

    pub fn install<W>(&self, witness: W)
    where
        W: Witness,
    {
        self.with_storage(|storage: &mut RecordStorage<W::Node>| {
            storage
                .witnesses
                .insert(TypeId::of::<W>(), Box::new(witness))
        });
    }

    pub fn remove<W>(&self) -> Option<W>
    where
        W: Witness,
    {
        self.with_storage(|storage: &mut RecordStorage<W::Node>| {
            storage.witnesses.remove(&TypeId::of::<W>())
        })
        .map(Downcast::into_any)
        .map(|any: Box<std::any::Any>| any.downcast().unwrap())
        .map(|boxed: Box<W>| *boxed)
    }

    fn with_storage<Node, Ret>(&self, op: impl FnOnce(&mut RecordStorage<Node>) -> Ret) -> Ret
    where
        Node: Recorded,
    {
        let mut storage_by_node = self.storage_by_node.lock();
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

    fn for_each_storage(&self, op: impl Fn(&mut dyn Records)) {
        self.storage_by_node
            .lock()
            .values_mut()
            .map(|b| &mut **b)
            .for_each(op)
    }

    pub(crate) fn flush_before_composition(&self) {
        self.for_each_storage(Records::flush_before_composition);
    }

    pub(crate) fn show_witnesses_after_composition(&self) {
        self.for_each_storage(Records::show_witnesses_after_composition)
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
    /// Probably needs a better name.
    fn show_witnesses_after_composition(&mut self);
}
impl_downcast!(Records);

impl<Node> Records for RecordStorage<Node>
where
    Node: Recorded,
{
    fn flush_before_composition(&mut self) {
        self.records.clear();
    }

    fn show_witnesses_after_composition(&mut self) {
        // TODO actually show witnesses the nodes, traverse the component tree down
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
