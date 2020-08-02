use super::{Gc, Liveness};
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
