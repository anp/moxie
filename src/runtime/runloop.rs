use super::{Revision, Runtime};
use futures::{
    stream::{Stream, StreamExt},
    task::LocalSpawn,
};
use std::{
    pin::Pin,
    task::{Context as FutContext, Poll, Waker},
};

/// A [`Runtime`] that is bound with a particular root function.
///
/// If running in a context with an async executor, can be consumed as a
/// [`futures::Stream`] in order to provide
/// the [`Runtime`] with a [`Waker`].
pub struct RunLoop<Root> {
    inner: Runtime,
    root: Root,
}

impl super::Runtime {
    /// Returns this runtime bound with a specific root function it will run in
    /// a loop.
    pub fn looped<Root, Out>(self, root: Root) -> RunLoop<Root>
    where
        Root: FnMut() -> Out,
    {
        // RunLoop always forces it's first revision?
        // or maybe just check if current revision is 0
        self.force();
        RunLoop { inner: self, root }
    }
}

impl<Root, Out> RunLoop<Root>
where
    Root: FnMut() -> Out + Unpin,
{
    /// Creates a new `Runtime` attached to the provided root function.
    pub fn new(root: Root) -> RunLoop<Root> {
        // maybe only there force first revision
        Runtime::new().looped(root)
    }

    /// Returns the runtime's current Revision.
    pub fn revision(&self) -> Revision {
        self.inner.revision()
    }

    /// Sets the [`Waker`] which will be called when state variables
    /// changes or if current `Revision` already has any state variables
    /// changed.
    pub fn set_state_change_waker(&mut self, wk: Waker) {
        self.inner.set_state_change_waker(wk);
    }

    /// Sets the executor that will be used to spawn normal priority tasks.
    pub fn set_task_executor(&mut self, sp: impl LocalSpawn + 'static) {
        self.inner.set_task_executor(sp);
    }

    /// Runs the root closure once with access to the runtime context, returning
    /// the result. `Revision` is incremented at the start of a run.
    pub fn run_once(&mut self) -> Out {
        self.inner.run_once(&mut self.root)
    }

    /// Runs the root closure once with access to the runtime context, returning
    /// the result. `Waker` is set for the next `Revision`, which starts after
    /// the start of the run.
    pub fn run_once_with(&mut self, waker: Waker) -> Out {
        self.inner.run_once_with(&mut self.root, waker)
    }

    /// Forces the next `Revision` without any changes.
    pub fn force(&self) {
        self.inner.force()
    }

    /// If change occured durig the last `Revision` then calls `run_once`
    /// else returns `Poll::Pending`. Note that RunLoop always forces it's first
    /// run (for now?)
    pub fn poll_once(&mut self) -> Poll<Out> {
        self.inner.poll_once(&mut self.root)
    }

    /// If change occured durig the last `Revision` then calls `run_once_with`
    /// else returns [`Poll::Pending`]. Note that RunLoop always forces it's
    /// first run (for now?)
    pub fn poll_once_with(&mut self, waker: Waker) -> Poll<Out> {
        self.inner.poll_once_with(&mut self.root, waker)
    }

    /// Poll this runtime without exiting. Discards any value returned from the
    /// root function. The future yields in between revisions and is woken on
    /// state changes.
    pub async fn run_on_state_changes(mut self) {
        loop {
            self.next().await;
        }
    }

    /// Unbinds the runtime from its current root function, returning both.
    pub fn unloop(self) -> (Runtime, Root) {
        (self.inner, self.root)
    }
}

impl<Root, Out> Stream for RunLoop<Root>
where
    Root: FnMut() -> Out + Unpin,
{
    type Item = Out;

    /// This `Stream` implementation yields until state change occurred or
    /// future fully [loads][crate::load].
    fn poll_next(self: Pin<&mut Self>, cx: &mut FutContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.poll_once_with(cx.waker().clone()).map(Some)
    }
}
