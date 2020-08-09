use super::Liveness;
use illicit::AsContext;
use parking_lot::Mutex;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    sync::{Arc, Weak},
};

#[derive(Clone, Debug, Default)]
pub(crate) struct DepNode {
    inner: Arc<Mutex<InnerDepNode>>,
}

impl DepNode {
    pub fn new(dependent: Dependent) -> Self {
        let this = Self { inner: Arc::new(Mutex::new(Default::default())) };
        this.root(dependent);
        this
    }

    pub fn root(&self, dependent: Dependent) {
        self.inner.lock().root(dependent);
    }

    pub fn as_dependent(&self) -> Dependent {
        Dependent { inner: Arc::downgrade(&self.inner) }
    }

    pub fn is_known_live(&self) -> bool {
        // TODO(#174) find a better way to handle cycles
        if let Some(l) = self.inner.try_lock() {
            matches!(l.liveness, Liveness::Live)
        } else {
            false
        }
    }

    pub fn update_liveness(&mut self) {
        // TODO(#174) find a better way to handle cycles
        if let Some(mut this) = self.inner.try_lock() {
            this.update_liveness();
        }
    }

    pub fn mark_dead(&mut self) {
        self.inner.lock().mark_dead();
    }

    /// Return the memory address of this `Dependent`.
    fn addr(&self) -> usize {
        Arc::as_ptr(&self.inner) as *const _ as _
    }
}

impl_common_traits_for_type_with_addr!(DepNode);

#[derive(Debug)]
struct InnerDepNode {
    liveness: Liveness,
    dependents: Vec<Dependent>,
}

impl Default for InnerDepNode {
    fn default() -> Self {
        Self { liveness: Liveness::Live, dependents: Vec::new() }
    }
}

impl InnerDepNode {
    /// Root this dep node in the current revision with the given `dependent`.
    fn root(&mut self, dependent: Dependent) {
        self.dependents.push(dependent);
        self.liveness = Liveness::Live;
    }

    /// Check incoming dependents for roots, marking ourselves live if a root
    /// exists. Drops stale dependents.
    fn update_liveness(&mut self) {
        self.dependents.sort_unstable();
        self.dependents.dedup();

        if matches!(self.liveness, Liveness::Live) {
            // we've already been here this gc, nothing new to see here
            return;
        }

        let mut has_root = false;
        self.dependents.retain(|dependent| {
            let mut keep = false;

            if let Some(mut dependent) = dependent.upgrade() {
                dependent.update_liveness();
                if dependent.is_known_live() {
                    has_root = true;
                }
                keep = true;
            }

            keep
        });

        // if we found a transitive root then mark ourselves live
        if has_root {
            self.liveness = Liveness::Live;
        }
    }

    fn mark_dead(&mut self) {
        self.liveness = Liveness::Dead;
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Dependent {
    inner: Weak<Mutex<InnerDepNode>>,
}

impl Dependent {
    /// Return the corresponding `DepNode` if it is still live.
    fn upgrade(&self) -> Option<DepNode> {
        self.inner.upgrade().map(|inner| DepNode { inner })
    }

    /// Returns the current incoming `Dependent`. If about to execute a
    /// top-level query this will return a null/no-op `Dependent`.
    pub fn incoming() -> Self {
        if let Ok(dep) = illicit::get::<Self>() { dep.clone() } else { Self::default() }
    }

    /// Initialize the dependency query with `self` marked as its immediate
    /// dependent.
    pub fn init_dependency<R>(self, op: impl FnOnce() -> R) -> R {
        self.offer(op)
    }

    /// Return the memory address of this `Dependent`.
    fn addr(&self) -> usize {
        self.inner.as_ptr() as *const _ as _
    }
}

impl_common_traits_for_type_with_addr!(Dependent);
