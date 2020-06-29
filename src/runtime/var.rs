use crate::{Commit, Key};
use parking_lot::Mutex;
use std::{sync::Arc, task::Waker};

/// The underlying container of state variables. Vends copies of the latest
/// [`Commit`] for [`Key`]s.
pub(crate) struct Var<State> {
    current: Commit<State>,
    id: topo::Id,
    pending: Option<Commit<State>>,
    waker: Waker,
}

impl<State> Var<State> {
    pub fn new(id: topo::Id, waker: Waker, inner: State) -> Arc<Mutex<Self>> {
        let current = Commit { id, inner: Arc::new(inner) };
        Arc::new(Mutex::new(Var { id, current, waker, pending: None }))
    }

    /// Attach this `Var` to its callsite, performing any pending commit and
    /// returning the resulting latest commit.
    pub fn root(var: Arc<Mutex<Self>>) -> (Commit<State>, Key<State>) {
        let (id, commit_at_root) = {
            let mut var = var.lock();
            if let Some(pending) = var.pending.take() {
                var.current = pending;
            }
            (var.id, var.current.clone())
        };

        (commit_at_root.clone(), Key { id, commit_at_root, var })
    }

    /// Returns a reference to the latest value, pending or committed.
    pub fn latest(&self) -> &State {
        &self.pending.as_ref().unwrap_or(&self.current)
    }

    /// Initiate a commit to the state variable. The commit will actually
    /// complete asynchronously when the state variable is next rooted in a
    /// topological function, flushing the pending commit.
    pub fn enqueue_commit(&mut self, state: State) {
        self.pending = Some(Commit { inner: Arc::new(state), id: self.id });
        self.waker.wake_by_ref();
    }
}
