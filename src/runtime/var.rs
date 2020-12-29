use crate::{Commit, Key};
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

use super::{Revision, RevisionControlSystem};

/// The underlying container of state variables. Vends copies of the latest
/// [`Commit`] for [`Key`]s.
pub(crate) struct Var<State> {
    current: Commit<State>,
    id: topo::CallId,
    // can only contain commits from previous revisions
    staged: Option<Commit<State>>,
    pending: Option<(Revision, Commit<State>)>,
    rcs: Arc<RwLock<RevisionControlSystem>>,
}

impl<State> Var<State> {
    pub fn new(
        id: topo::CallId,
        rcs: Arc<RwLock<RevisionControlSystem>>,
        inner: State,
    ) -> Arc<Mutex<Self>> {
        let current = Commit { id, inner: Arc::new(inner) };
        Arc::new(Mutex::new(Var { id, current, rcs, staged: None, pending: None }))
    }

    /// Attach this `Var` to its callsite, performing any pending commit and
    /// returning the resulting latest commit.
    pub fn root(var: Arc<Mutex<Self>>) -> (Commit<State>, Key<State>) {
        let (id, commit_at_root) = {
            let mut var = var.lock();
            let current = Revision::current();

            // stage pending commit if it's from previous revision
            match var.pending {
                Some((revision, _)) if revision < current => {
                    var.staged = var.pending.take().map(|(_r, c)| c)
                }
                _ => (),
            }

            // perform staged commit
            if let Some(staged) = var.staged.take() {
                var.current = staged;
            }

            (var.id, var.current.clone())
        };

        (commit_at_root, Key { id, var })
    }

    /// Returns a reference to the latest value, pending or committed.
    pub fn latest(&self) -> &State {
        self.pending
            .as_ref()
            .map(|(_revision, ref commit)| commit)
            .or_else(|| self.staged.as_ref())
            .unwrap_or(&self.current)
    }

    ///
    pub fn current_commit(&self) -> &Commit<State> {
        &self.current
    }

    /// Initiate a commit to the state variable. The commit will actually
    /// complete asynchronously when the state variable is next rooted in a
    /// topological function, flushing the pending commit.
    pub fn enqueue_commit(&mut self, state: State) {
        let new_commit = Commit { inner: Arc::new(state), id: self.id };
        let rcs_read = self.rcs.read();
        let current = rcs_read.revision;

        match self.pending.replace((current, new_commit)) {
            Some((revision, old_commit)) if revision < current => self.staged = Some(old_commit),
            _ => (),
        }

        rcs_read.pending_changes.store(true, std::sync::atomic::Ordering::Relaxed);
        rcs_read.waker.wake_by_ref();
    }
}
