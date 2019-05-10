use {
    crate::{our_prelude::*, scope::WeakScope, Scope, Witness},
    downcast_rs::*,
    parking_lot::Mutex,
    shrinkwraprs::*,
    std::{any::TypeId, collections::HashMap},
};

#[derive(Default, Shrinkwrap)]
pub(crate) struct Recorder {
    inner: Mutex<HashMap<TypeId, Mutex<Box<dyn Records>>>>,
}

impl Recorder {
    pub fn child(&self) -> Self {
        Self {
            inner: Mutex::new(
                self.inner
                    .lock()
                    .iter()
                    .map(|(&id, records)| (id, Mutex::new(records.lock().child())))
                    .collect(),
            ),
        }
    }

    pub fn record<N>(&self, node: N, scope: WeakScope)
    where
        N: 'static,
    {
        self.with_storage(|storage| {
            storage.record = Some((node, scope));
        });
    }

    pub fn install_witness<W>(&self, witness: W)
    where
        W: Witness,
    {
        self.with_storage(|storage: &mut RecordStorage<W::Node>| {
            trace!("installing witness");
            storage.witness = Some(Box::new(witness));
        });
    }

    pub fn remove_witness<W>(&self) -> Option<W>
    where
        W: Witness,
    {
        self.with_storage(|storage: &mut RecordStorage<W::Node>| {
            trace!("removing witness");
            storage.witness.take()
        })
        .map(Downcast::into_any)
        .map(|any: Box<std::any::Any>| any.downcast().unwrap())
        .map(|boxed: Box<W>| *boxed)
    }

    pub fn with_storage<Node: 'static, Ret>(
        &self,
        op: impl FnOnce(&mut RecordStorage<Node>) -> Ret,
    ) -> Ret {
        let mut storage_by_node = self.lock();
        let storage: &mut Mutex<Box<dyn Records>> = storage_by_node
            .entry(TypeId::of::<Node>())
            .or_insert_with(|| {
                let storage: RecordStorage<Node> = RecordStorage::default();
                Mutex::new(Box::new(storage))
            });
        let storage: &mut dyn Records = &mut **storage.lock();
        let storage: &mut std::any::Any = storage.as_any_mut();
        let storage: &mut RecordStorage<Node> = storage.downcast_mut().unwrap();

        // not panic-safe, maybe fix?
        op(storage)
    }

    pub fn for_each_storage(&self, op: impl Fn(&mut dyn Records)) {
        self.lock().values_mut().for_each(|b| {
            let mut guard = b.lock();
            let storage = &mut **guard;
            // not panic-safe, maybe fix?
            op(storage)
        })
    }
}

pub(crate) struct RecordStorage<Node>
where
    Node: 'static,
{
    record: Option<(Node, WeakScope)>,
    nearest_parent: Option<WeakScope>,
    witness: Option<Box<dyn Witness<Node = Node>>>,
}

pub(crate) trait Records: Downcast + 'static {
    /// Clear recorded nodes from storage. Should be called immediately before composing in this
    /// scope.
    fn flush_before_composition(&mut self);

    /// Show the current component hierarchy and associated recordings to all installed witnesses.
    ///
    /// Probably needs a better name. Takes the current scope as an argument so that it can
    /// traverse to children. Vague name, poor API. We'll refactor this another time.
    fn show_witnesses_after_composition(&mut self, scope: Scope);

    fn child(&self) -> Box<dyn Records>;
}

impl_downcast!(Records);

impl<Node> Records for RecordStorage<Node> {
    fn child(&self) -> Box<dyn Records> {
        let nearest_parent = if let Some((_, originating)) = &self.record {
            Some(originating.clone())
        } else {
            self.nearest_parent.clone()
        };

        Box::new(Self {
            record: None,
            witness: None,
            nearest_parent,
        })
    }

    fn flush_before_composition(&mut self) {
        self.record = None;
    }

    fn show_witnesses_after_composition(&mut self, start: Scope) {
        if let Some(mut witness) = self.witness.take() {
            let mut to_visit: Vec<Scope> = vec![];
            let mut show_witness_record_storage = |storage: &mut Self| {
                let parent = storage.nearest_parent.as_ref().and_then(WeakScope::upgrade);
                if let Some((record, _scope)) = &storage.record {
                    match parent {
                        // don't want to deadlock on ourselves!
                        Some(ref p) if !Arc::ptr_eq(&p.0, &start.0) => {
                            p.0.records.with_storage(|parent_storage: &mut Self| {
                                let parent_record = parent_storage.record.as_ref().map(|r| &r.0);
                                witness.see(record, parent_record);
                            });
                        }
                        _ => witness.see(record, None),
                    }
                }
            };

            let add_children_to_be_visited = |to_visit: &mut Vec<Scope>, visiting: &Scope| {
                let children = visiting.0.children.lock();
                let bind_order = visiting.0.bind_order.lock();
                to_visit.extend(bind_order.iter().map(|id| children[id].clone()));
            };

            // needs to run without acquiring this record storage to avoid deadlock,
            // we're being called from within its lock guard's scope after all!
            show_witness_record_storage(self);
            add_children_to_be_visited(&mut to_visit, &start);

            while let Some(visiting) = to_visit.pop() {
                visiting
                    .0
                    .records
                    .with_storage(&mut show_witness_record_storage);
                add_children_to_be_visited(&mut to_visit, &visiting);
            }

            self.witness = Some(witness);
        }
    }
}

impl<Node> Default for RecordStorage<Node>
where
    Node: 'static,
{
    fn default() -> Self {
        Self {
            record: None,
            nearest_parent: None,
            witness: Default::default(),
        }
    }
}
