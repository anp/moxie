use super::{Revision, RevisionControlSystem, Spawner, Var};
use crate::{Commit, Key};
use dyn_cache::local::SharedLocalCache;
use futures::future::abortable;
use parking_lot::RwLock;
use std::{borrow::Borrow, future::Future, sync::Arc, task::Poll};

/// A handle to the current [`Runtime`] which is offered via [`illicit`]
/// contexts and provides access to the current revision, cache storage,
/// task spawning, and the waker for the loop.
#[derive(Debug)]
pub(crate) struct Context {
    rcs: Arc<RwLock<RevisionControlSystem>>,
    pub cache: SharedLocalCache,
    spawner: Spawner,
}

impl Context {
    /// Returns the revision for which this context was created.
    pub fn revision(&self) -> Revision {
        self.rcs.read().revision
    }

    /// Load a [`crate::state::Var`] with the provided argument and initializer.
    /// Re-initializes the `Var` whenever `arg` changes.
    pub fn cache_state<Arg, Input, Output>(
        &self,
        id: &topo::CallId,
        arg: &Arg,
        init: impl FnOnce(&Input) -> Output,
    ) -> (Commit<Output>, Key<Output>)
    where
        Arg: PartialEq<Input> + ToOwned<Owned = Input> + ?Sized,
        Input: Borrow<Arg> + 'static,
        Output: 'static,
    {
        let var = self
            .cache
            .cache(id, arg, |arg| Var::new(topo::CallId::current(), self.rcs.clone(), init(arg)));
        Var::root(var)
    }

    /// Load a value from the future returned by `init` whenever `capture`
    /// changes, returning the result of calling `with` with the loaded
    /// value. Cancels the running future if there's no longer interest
    /// in its output, indicated by a revision in which this was not called with
    /// the given `id`.
    ///
    /// # Panics
    ///
    /// If the [`super::Runtime`] from which `self` was created did not have
    /// a valid call to `set_task_executor`.
    pub fn load_with<Arg, Input, Fut, Output, Ret>(
        &self,
        id: &topo::CallId,
        arg: &Arg,
        init: impl FnOnce(&Input) -> Fut,
        with: impl FnOnce(&Output) -> Ret,
    ) -> Poll<Ret>
    where
        Arg: PartialEq<Input> + ToOwned<Owned = Input> + ?Sized,
        Input: Borrow<Arg> + 'static,
        Fut: Future<Output = Output> + 'static,
        Output: 'static,
        Ret: 'static,
    {
        let var = self.cache.cache_with(
            id,
            arg,
            |arg| {
                // before we spawn the new task we need to mark it pending
                let var = Var::new(topo::CallId::current(), self.rcs.clone(), Poll::Pending);

                let (fut, aborter) = abortable(init(arg));

                let var2 = var.clone();
                let task = async move {
                    if let Ok(to_store) = fut.await {
                        var2.lock().enqueue_commit(Poll::Ready(to_store));
                    }
                };
                self.spawner
                    .0
                    .spawn_local_obj(Box::pin(task).into())
                    .expect("that set_task_executor has been called");
                (var, scopeguard::guard(aborter, |a| a.abort()))
            },
            |(var, _)| var.clone(),
        );

        match *Var::root(var).0 {
            Poll::Ready(ref stored) => Poll::Ready(with(stored)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl super::Runtime {
    pub(crate) fn context_handle(&self) -> Context {
        Context { rcs: self.rcs.clone(), spawner: self.spawner.clone(), cache: self.cache.clone() }
    }
}
