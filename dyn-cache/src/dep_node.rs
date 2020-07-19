use super::{Gc, Liveness};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct DepNode {
    is_root: AtomicBool,
}

impl DepNode {
    pub fn new() -> Self {
        Self { is_root: AtomicBool::new(true) }
    }

    pub fn root(&self) {
        self.is_root.store(true, Ordering::Release);
    }
}

impl Gc for DepNode {
    /// Always marks itself as dead in a GC, returning its previous value.
    fn sweep(&mut self) -> Liveness {
        if self.is_root.swap(false, Ordering::AcqRel) { Liveness::Live } else { Liveness::Dead }
    }
}
