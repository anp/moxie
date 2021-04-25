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
    pub fn new(dependent: Dependent, revision: u64) -> Self {
        let this = Self { inner: Arc::new(Mutex::new(Default::default())) };
        this.root_write(dependent, revision);
        this
    }

    /// Mark this node as having been read in the current GC revision. If it hasn't been updated at
    /// all before a GC, then its dependencies will inherit its liveness.
    pub fn root_read(&self, dependent: Dependent) {
        self.inner.lock().root_read(dependent);
    }

    /// Mark this node as having been written to in the current GC revision. This indicates that the
    /// liveness of its dependencies should be assessed on their own, because they'll have had a
    /// chance to execute (or not) this revision.
    pub fn root_write(&self, dependent: Dependent, revision: u64) {
        self.inner.lock().root_write(dependent, revision);
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

    fn should_inherit_liveness(&self, current_revision: u64) -> bool {
        if let Some(l) = self.inner.try_lock() {
            // if the dependent was updated during this revision, then our dependency should only
            // consider *its own* liveness. consider the following pseudocode:
            //
            //     cache.cache_with(unique_value(), |_| {
            //         if externally_modifiable_bool() {
            //              cache.hold_with((), |v| op(v));
            //         }
            //     });
            //
            // in this case, the inner hold_with() call should not be retained if
            // externally_modifiable_bool() returns false. to achieve this, we want the
            // cache_with call's liveness to never propagate when the initialization closure
            // executes.
            l.updated_at_revision != current_revision
        } else {
            false
        }
    }

    pub fn update_liveness(&mut self, current_revision: u64) {
        // TODO(#174) find a better way to handle cycles
        if let Some(mut this) = self.inner.try_lock() {
            this.update_liveness(current_revision);
        }
    }

    pub fn mark_dead(&mut self) {
        self.inner.lock().mark_dead();
    }

    /// Return the memory address of this `DepNode`.
    fn addr(&self) -> usize {
        Arc::as_ptr(&self.inner) as *const _ as _
    }
}

impl_common_traits_for_type_with_addr!(DepNode);

#[derive(Debug)]
struct InnerDepNode {
    liveness: Liveness,
    updated_at_revision: u64,
    dependents: Vec<Dependent>,
}

impl Default for InnerDepNode {
    fn default() -> Self {
        Self { liveness: Liveness::Live, updated_at_revision: 0, dependents: Vec::new() }
    }
}

impl InnerDepNode {
    fn root_read(&mut self, dependent: Dependent) {
        self.dependents.push(dependent);
        self.liveness = Liveness::Live;
    }

    fn root_write(&mut self, dependent: Dependent, revision: u64) {
        self.dependents.push(dependent);
        self.liveness = Liveness::Live;
        self.updated_at_revision = revision;
    }

    /// Check incoming dependents for roots, marking ourselves live if a root
    /// exists. Drops stale dependents.
    fn update_liveness(&mut self, current_revision: u64) {
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
                dependent.update_liveness(current_revision);
                if dependent.should_inherit_liveness(current_revision) && dependent.is_known_live()
                {
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
        if let Ok(dep) = illicit::get::<Self>() {
            dep.clone()
        } else {
            Self::default()
        }
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
