//! Integration points for the moxie runtime. Generally not called directly by
//! applications.
//!
//! To integrate moxie with an existing system, a `Runtime` must be created with
//! a user-provided root closure to execute each `Revision`. If the embedding
//! system will change its scheduling based on mutations of moxie state
//! variables (e.g. scheduling a new frame render), then it should also call
//! `Runtime::set_state_change_waker` to be notified when state values have been
//! updated.
//!
//! Once these have been created, the embedding system should call
//! `Runtime::run_once` whenever appropriate for the context. This may be on a
//! regular cadence (i.e. once per frame interval) or only when state mutations
//! have occurred (as is the default in [moxie-dom]).
//!
//! [moxie-dom]: https://docs.rs/moxie-dom

mod executor;

use crate::memo::LocalCache;
use executor::InBandExecutor;
use futures::{
    stream::{Stream, StreamExt},
    task::{noop_waker, LocalSpawn},
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

/// Revisions measure moxie's notion of time passing. Each `Runtime` increments
/// its Revision on every iteration. `crate::Commit`s to state variables are
/// annotated with the Revision during which they were made.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

impl Revision {
    /// Returns the current revision. Will return `Revision(0)` if called
    /// outside of a Runtime's execution.
    pub fn current() -> Self {
        if let Some(r) = illicit::get::<RuntimeHandle>() { r.revision } else { Revision::default() }
    }
}

impl std::fmt::Debug for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("r{}", self.0))
    }
}

/// The primary integration point between `moxie` and an embedding environment.
/// Owns the cache and state for a moxie application. Each instance is
/// independent.
///
/// ## Minimal Example
///
/// This example has no side effects in its root closure, and doesn't have any
/// state variables which might require mutation wakeups.
///
/// ```
/// # use moxie::embed::{Revision, Runtime};
/// let mut rt = Runtime::new();
/// assert_eq!(rt.revision().0, 0);
/// for i in 1..10 {
///     rt.run_once(|| ());
///     assert_eq!(rt.revision(), Revision(i));
/// }
/// ```
pub struct Runtime {
    revision: Revision,
    store: LocalCache,
    spawner: Spawner,
    executor: InBandExecutor,
    wk: RunLoopWaker,
}

impl Default for Runtime {
    fn default() -> Runtime {
        Runtime::new()
    }
}

impl Runtime {
    /// Construct a new [`Runtime`] with blank storage and no external waker or
    /// task executor.
    ///
    /// By default the task executor used for `load` and its siblings is the
    /// same single-threaded one as the executor used for `handler` futures.
    /// It is strongly recommended that outside of testing users of this
    /// struct call `set_task_executor` with a reference to a more robust
    /// I/O- or compute-oriented executor.
    pub fn new() -> Self {
        let executor = InBandExecutor::default();
        let spawner = executor.spawner();
        Self {
            executor,
            spawner,
            revision: Revision(0),
            store: LocalCache::default(),
            wk: RunLoopWaker(noop_waker()),
        }
    }

    /// The current revision of the runtime, or how many times `run_once` has
    /// been invoked.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// Runs the root closure once with access to memoization storage,
    /// increments the runtime's `Revision`, and drops any memoized values
    /// which were not marked `Liveness::Live`.
    pub fn run_once<Out>(&mut self, func: impl FnOnce() -> Out) -> Out {
        self.revision.0 += 1;

        let ret = illicit::Layer::new().with(self.handle()).enter(|| {
            topo::call(|| {
                self.executor.run_until_stalled(&self.wk.0);
                let ret = func();

                // run executor again to make sure that newly spawned ones can install wakers
                self.executor.run_until_stalled(&self.wk.0);
                ret
            })
        });

        self.store.gc();
        ret
    }

    /// Sets the [`std::task::Waker`] which will be called when state variables
    /// receive commits. By default the runtime no-ops on a state change,
    /// which is probably the desired behavior if the embedding system will
    /// call `Runtime::run_once` on a regular interval regardless.
    pub fn set_state_change_waker(&mut self, wk: Waker) {
        self.wk = RunLoopWaker(wk);
    }

    /// Sets the executor that will be used to spawn normal priority tasks.
    pub fn set_task_executor(&mut self, sp: impl LocalSpawn + 'static) {
        self.spawner = Spawner(Rc::new(sp));
    }

    /// Calls `run_once` in a loop until `filter` returns `Poll::Ready`,
    /// returning the result of that `Revision`.
    pub fn run_until_ready<Out>(
        &mut self,
        mut func: impl FnMut() -> Out,
        mut filter: impl FnMut(Out) -> Poll<Out>,
    ) -> Out {
        loop {
            if let Poll::Ready(out) = filter(self.run_once(&mut func)) {
                return out;
            }
        }
    }

    /// Calls `run_once` in a loop until the runtime's revision is equal to the
    /// one provided.
    ///
    /// Always calls `run_once` at least once.
    pub fn run_until_at_least_revision<Out>(
        &mut self,
        rev: Revision,
        mut func: impl FnMut() -> Out,
    ) -> Out {
        loop {
            let out = self.run_once(&mut func);
            if self.revision >= rev {
                return out;
            }
        }
    }

    fn handle(&self) -> RuntimeHandle {
        RuntimeHandle {
            revision: self.revision,
            spawner: self.spawner.clone(),
            store: self.store.clone(),
            waker: self.wk.clone(),
        }
    }
}

/// A [`Runtime`] that is bound with a particular root function.
pub struct RootedRuntime<Root> {
    inner: Runtime,
    root: Root,
}

impl<Root, Out> RootedRuntime<Root>
where
    Root: FnMut() -> Out + Unpin,
{
    /// Attach a root function to this `Runtime` so that only that function will
    /// be called by the runtime.
    pub fn new(root: Root) -> RootedRuntime<Root> {
        RootedRuntime { root, inner: Runtime::new() }
    }

    /// Set's the [`std::task::Waker`] which will be called when state variables
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

    /// Poll this runtime without stopping. Discards any value returned from the
    /// root function.
    pub async fn run_forever(mut self) {
        loop {
            self.next().await;
        }
    }
}

impl<Root, Out> Stream for RootedRuntime<Root>
where
    Root: FnMut() -> Out + Unpin,
{
    type Item = (Revision, Out);

    /// This `Stream` implementation runs a single revision for each call to
    /// `poll_next`, always returning `Poll::Ready(Some(...))`.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.inner.set_state_change_waker(cx.waker().clone());
        let out = this.run_once();
        Poll::Ready(Some((this.inner.revision, out)))
    }
}

/// A handle to the current [`Runtime`] which is offered via [`illicit`]
/// contexts and provides access to the current revision, memoization storage,
/// task spawning, and the waker for the loop.
#[derive(Debug)]
pub(crate) struct RuntimeHandle {
    pub(crate) revision: Revision,
    pub(crate) spawner: Spawner,
    pub(crate) store: LocalCache,
    pub(crate) waker: RunLoopWaker,
}

#[derive(Clone)]
pub(crate) struct Spawner(pub Rc<dyn LocalSpawn>);

impl Debug for Spawner {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Spawner({:p})", self.0)
    }
}

/// Responsible for waking the Runtime task. Because the illicit environment is
/// namespaced by type, we create a newtype here so that other crates don't
/// accidentally cause strange behavior by overriding our access to it when
/// passing their own wakers down.
#[derive(Clone, Debug)]
pub(crate) struct RunLoopWaker(Waker);

impl RunLoopWaker {
    pub(crate) fn wake(&self) {
        self.0.wake_by_ref();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagating_env_to_runtime() {
        let first_byte = 0u8;

        let mut runtime = RootedRuntime::new(|| {
            let from_env: u8 = *illicit::expect();
            assert_eq!(from_env, first_byte);
        });

        assert!(illicit::get::<u8>().is_none());
        illicit::Layer::new().with(first_byte).enter(|| {
            topo::call(|| runtime.run_once());
        });
        assert!(illicit::get::<u8>().is_none());
    }

    #[test]
    fn revision_bound_runtime() {
        let mut rt = Runtime::new();
        let run = || Revision::current();
        let rev = rt.run_until_at_least_revision(Revision(4), run);
        assert_eq!(rev, Revision(4), "exactly 4 iterations should occur");

        let rev = rt.run_until_at_least_revision(Revision(4), run);
        assert_eq!(rev, Revision(5), "exactly 1 more iteration should occur");
    }

    #[test]
    fn readiness_bound_runtime() {
        let mut rt = Runtime::new();
        let rev = rt.run_until_ready(Revision::current, |rev| {
            if rev == Revision(3) { Poll::Ready(rev) } else { Poll::Pending }
        });
        assert_eq!(rev, Revision(3), "exactly 3 iterations should occur");
    }
}
