use super::{Gc, Liveness};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct DepNode {
    inner: AtomicBool,
}

impl DepNode {
    pub fn new() -> Self {
        Self { inner: AtomicBool::new(true) }
    }

    pub fn mark_live(&self) {
        self.inner.store(true, Ordering::Relaxed);
    }
}

impl Gc for DepNode {
    /// Always marks itself as dead in a GC, returning its previous value.
    fn sweep(&mut self) -> Liveness {
        if self.inner.swap(false, Ordering::Relaxed) { Liveness::Live } else { Liveness::Dead }
    }
}
