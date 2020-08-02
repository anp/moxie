use super::{Gc, Liveness};
use illicit::AsContext;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Weak,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct DepNode {
    inner: Arc<InnerDepNode>,
}

#[derive(Debug, Default)]
struct InnerDepNode {
    has_root: AtomicBool,
}

impl DepNode {
    pub fn new() -> Self {
        Self { inner: Arc::new(InnerDepNode { has_root: AtomicBool::new(true) }) }
    }

    pub fn root(&self, _dependent: Dependent) {
        self.inner.has_root.store(true, Ordering::Release);
    }

    pub fn as_dependent(&self) -> Dependent {
        Dependent { inner: Arc::downgrade(&self.inner) }
    }
}

impl Gc for DepNode {
    fn mark(&mut self) {
        // TODO
    }

    fn sweep(&mut self) -> Liveness {
        if self.inner.has_root.swap(false, Ordering::AcqRel) {
            Liveness::Live
        } else {
            Liveness::Dead
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Dependent {
    inner: Weak<InnerDepNode>,
}

impl Dependent {
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
}
