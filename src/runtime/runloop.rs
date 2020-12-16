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
/// [`futures::Stream`] of [`crate::runtime::Revision`]s in order to provide
/// the [`super::Runtime`] with a [`std::task::Waker`].
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
        // RunLoop always forces first revision
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
        Runtime::new().looped(root)
    }

    /// Returns the runtime's current Revision.
    pub fn revision(&self) -> Revision {
        self.inner.revision()
    }

    /// Sets the [`std::task::Waker`] which will be called when state variables
    /// change.
    pub fn set_state_change_waker(&mut self, wk: Waker) {
        self.inner.set_state_change_waker(wk);
    }

    /// Sets the executor that will be used to spawn normal priority tasks.
    pub fn set_task_executor(&mut self, sp: impl LocalSpawn + 'static) {
        self.inner.set_task_executor(sp);
    }

    /// Run the root function once within this runtime's context, returning the
    /// result.
    pub fn run_once(&mut self) -> Out {
        self.inner.run_once(&mut self.root)
    }

    /// Run the root function once within this runtime's context, returning the
    /// result.
    pub fn run_once_with(&mut self, waker: Waker) -> Out {
        self.inner.run_once_with(&mut self.root, waker)
    }

    /// TODO description
    pub fn force(&self) {
        self.inner.force()
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

    /// This `Stream` implementation runs a single revision for each call to
    /// `poll_next`, always returning `Poll::Ready(Some(...))`.
    fn poll_next(self: Pin<&mut Self>, cx: &mut FutContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.inner.poll_once_with(&mut this.root, cx.waker().clone()).map(Some)
    }
}
