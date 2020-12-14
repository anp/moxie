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
        RunLoop { inner: self, root }
    }
}

impl<Root, Out> RunLoop<Root>
where
    Root: FnMut() -> Out + Unpin,
{
    /// Creates a new `Runtime` attached to the provided root function.
    pub fn new(root: Root) -> RunLoop<Root> {
        RunLoop { root, inner: Runtime::new() }
    }

    /// Returns the runtime's current Revision.
    pub fn revision(&self) -> Revision {
        self.inner.revision()
    }

    /// Sets the executor that will be used to spawn normal priority tasks.
    pub fn set_task_executor(&mut self, sp: impl LocalSpawn + 'static) {
        self.inner.set_task_executor(sp);
    }

    /// Run the root function once within this runtime's context, returning the
    /// result.
    pub fn force_next(&mut self) -> Out {
        self.inner.force_once(&mut self.root)
    }

    /// Run the root function once within this runtime's context, returning the
    /// result.
    pub fn force_next_with(&mut self, waker: Waker) -> Out {
        self.inner.force_once_with(&mut self.root, waker)
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
        this.inner.poll_once(&mut this.root, Some(cx.waker().clone())).map(|o| Some(o))
    }
}
