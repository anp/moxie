use super::{Revision, Var};
use crate::{Commit, Key};
use futures::{future::abortable, task::LocalSpawn};
use std::{
    borrow::Borrow,
    fmt::{Debug, Formatter, Result as FmtResult},
    future::Future,
    rc::Rc,
    task::{Poll, Waker},
};

/// A handle to the current [`Runtime`] which is offered via [`illicit`]
/// contexts and provides access to the current revision, memoization storage,
/// task spawning, and the waker for the loop.
pub(crate) struct Context {
    revision: Revision,
    pub cache: topo::LocalCacheHandle,
    spawner: Rc<dyn LocalSpawn>,
    waker: Waker,
}

impl Context {
    /// Returns the revision for which this context was created.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// Load a [`crate::state::Var`] with the provided argument and initializer.
    /// Re-initializes the `Var` whenever `arg` changes.
    pub fn memo_state<Arg, Input, Output>(
        &self,
        id: topo::Id,
        arg: &Arg,
        init: impl FnOnce(&Input) -> Output,
    ) -> (Commit<Output>, Key<Output>)
    where
        Arg: PartialEq<Input> + ToOwned<Owned = Input> + ?Sized,
        Input: Borrow<Arg> + 'static,
        Output: 'static,
    {
        let var = self.cache.memo_with(
            id,
            arg,
            |arg| Var::new(topo::Id::current(), self.waker.clone(), init(arg)),
            Clone::clone,
        );
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
        id: topo::Id,
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
        let (result, set_result): (_, Key<Poll<Output>>) =
            self.memo_state(id, &(), |()| Poll::Pending);
        self.cache.memo_with(
            id,
            arg,
            |arg| {
                let (fut, aborter) = abortable(init(arg));
                let task = async move {
                    if let Ok(to_store) = fut.await {
                        set_result.update(|_| Some(Poll::Ready(to_store)));
                    }
                };
                self.spawner
                    .spawn_local_obj(Box::pin(task).into())
                    .expect("that set_task_executor has been called");
                scopeguard::guard(aborter, |a| a.abort())
            },
            |_| {},
        );

        match &*result {
            Poll::Ready(ref stored) => Poll::Ready(with(stored)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Context")
            .field("revision", &self.revision)
            .field("cache", &self.cache)
            .field("spawner", &format_args!("{:p}", &self.spawner)) // TODO print the pointer
            .field("waker", &self.waker)
            .finish()
    }
}

impl super::Runtime {
    pub(crate) fn context_handle(&self) -> Context {
        Context {
            revision: self.revision,
            spawner: self.spawner.clone(),
            cache: self.cache.clone(),
            waker: self.wk.clone(),
        }
    }
}
