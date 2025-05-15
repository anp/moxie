//! Utilities for testing moxie-based programs.

use futures::task::ArcWake;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

/// A value which keeps track of how many times it's been cloned. Useful for
/// testing caching behaviors.
#[derive(Default)]
pub struct CountsClones(Arc<AtomicU64>);

impl CountsClones {
    /// Returns the number of times this value has been cloned.
    pub fn clone_count(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

impl Clone for CountsClones {
    fn clone(&self) -> Self {
        self.0.fetch_add(1, Ordering::Relaxed);
        Self(self.0.clone())
    }
}

/// A waker which keeps track of whether it's been woken or not. Useful for
/// testing state update behaviors.
pub struct BoolWaker(AtomicBool);

impl BoolWaker {
    /// Returns a new instance.
    pub fn new() -> Arc<Self> {
        Arc::new(Self(AtomicBool::new(false)))
    }

    /// Returns true if woken since the last call to this method.
    pub fn is_woken(&self) -> bool {
        self.0.swap(false, Ordering::Relaxed)
    }
}

impl ArcWake for BoolWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.0.store(true, Ordering::Relaxed);
    }
}
