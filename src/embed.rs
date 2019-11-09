//! Integration points for the moxie runtime. Generally not called directly by applications.
//!
//! To integrate moxie with an existing system, a `Runtime` must be created with a user-provided
//! root closure to execute each `Revision`. If the embedding system will change its scheduling
//! based on mutations of moxie state variables (e.g. scheduling a new frame render), then it
//! should also call `Runtime::set_state_change_waker` to be notified when state values have been
//! updated.
//!
//! Once these have been created, the embedding system should call `Runtime::run_once` whenever
//! appropriate for the context. This may be on a regular cadence (i.e. once per frame interval) or
//! only when state mutations have occurred (as is the default in [moxie-dom]).
//!
//! [moxie-dom]: https://docs.rs/moxie-dom

mod executor;

use {
    crate::memo::MemoStore,
    executor::InBandExecutor,
    futures::task::LocalSpawn,
    std::{
        fmt::{Debug, Formatter, Result as FmtResult},
        rc::Rc,
        task::{Poll, Waker},
    },
};

/// Revisions measure moxie's notion of time passing. Each `Runtime` increments its Revision
/// on every iteration. `crate::Commit`s to state variables are annotated with the Revision
/// during which they were made.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

impl Revision {
    /// Returns the current revision. Will return `Revision(0)` if called outside of a Runtime's
    /// execution.
    pub fn current() -> Self {
        if let Some(r) = illicit::Env::get::<Revision>() {
            *r
        } else {
            Revision::default()
        }
    }
}

impl std::fmt::Debug for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("r{}", self.0))
    }
}

/// The primary integration point between `moxie` and an embedding environment. Owns the cache and
/// state for a moxie application. Each instance is independent.
///
/// ## Minimal Example
///
/// This example has no side effects in its root closure, and doesn't have any state variables
/// which might require mutation wakeups.
///
/// ```
/// # use moxie::embed::{Revision, Runtime};
/// let mut rt = Runtime::new(|| {});
/// assert_eq!(rt.revision().0, 0);
/// for i in 1..10 {
///     rt.run_once();
///     assert_eq!(rt.revision(), Revision(i));
/// }
/// ```
pub struct Runtime<Root> {
    revision: Revision,
    store: MemoStore,
    root: Root,
    spawner: Spawner,
    handlers: InBandExecutor,
    wk: Waker,
}

impl<Root, Out> Runtime<Root>
where
    Root: FnMut() -> Out,
{
    /// Construct a new [`Runtime`] with blank storage and no external waker or task executor.
    ///
    /// By default the task executor used for `load` and its siblings is the same single-threaded
    /// one as the executor used for `handler` futures. It is strongly recommended that outside of
    /// testing users of this struct call `set_task_executor` with a reference to a more robust
    /// I/O- or compute-oriented executor.
    pub fn new(root: Root) -> Self {
        let handlers = InBandExecutor::default();
        let fallback_spawner = handlers.spawner();
        Self {
            root,
            handlers,
            revision: Revision(0),
            spawner: fallback_spawner,
            store: MemoStore::default(),
            wk: futures::task::noop_waker(),
        }
    }

    /// Constructs a new [`Runtime`], runs the provided `root` once, and returns the result.
    pub fn oneshot(root: Root) -> Out {
        let mut this = Self::new(root);
        this.run_once()
    }

    /// The current revision of the runtime, or how many times `run_once` has been invoked.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// Runs the root closure once with access to memoization storage, increments the runtime's
    /// `Revision`, and drops any memoized values which were not marked `Liveness::Live`.
    pub fn run_once(&mut self) -> Out {
        self.revision.0 += 1;

        let ret = illicit::child_env! {
            MemoStore => self.store.clone(),
            Revision => self.revision,
            RunLoopWaker => RunLoopWaker(self.wk.clone()),
            Spawner => self.spawner.clone()
        }
        .enter(|| {
            topo::call(|| {
                self.handlers.run_until_stalled(&self.wk);
                let ret = (self.root)();

                // run handlers again to make sure that newly spawned ones can install wakers
                self.handlers.run_until_stalled(&self.wk);
                ret
            })
        });

        self.store.gc();
        ret
    }

    /// Calls `run_once` in a loop until `filter` returns `Poll::Ready`, returning the result of
    /// that `Revision`.
    pub fn run_until_ready(&mut self, mut filter: impl FnMut(Out) -> Poll<Out>) -> Out {
        loop {
            if let Poll::Ready(out) = filter(self.run_once()) {
                return out;
            }
        }
    }

    /// Calls `run_once` in a loop until the runtime's revision is equal to the one provided.
    ///
    /// Always calls `run_once` at least once.
    pub fn run_until_at_least_revision(&mut self, rev: Revision) -> Out {
        loop {
            let out = self.run_once();
            if self.revision >= rev {
                return out;
            }
        }
    }

    /// Sets the [`std::task::Waker`] which will be called when state variables receive commits. By
    /// default the runtime no-ops on a state change, which is probably the desired behavior if
    /// the embedding system will call `Runtime::run_once` on a regular interval regardless.
    pub fn set_state_change_waker(&mut self, wk: Waker) -> &mut Self {
        self.wk = wk;
        self
    }

    /// Sets the executor that will be used to spawn normal priority tasks.
    pub fn set_task_executor(&mut self, sp: impl LocalSpawn + 'static) -> &mut Self {
        self.spawner = Spawner(Rc::new(sp));
        self
    }
}

#[derive(Clone)]
pub(crate) struct Spawner(pub Rc<dyn LocalSpawn>);

impl Debug for Spawner {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Spawner({:p})", self.0)
    }
}

/// Responsible for waking the Runtime task. Because the illicit environment is namespaced by type,
/// we create a newtype here so that other crates don't accidentally cause strange behavior by
/// overriding our access to it when passing their own wakers down.
#[derive(Clone, Debug)]
pub(crate) struct RunLoopWaker(std::task::Waker);

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

        let mut runtime = Runtime::new(|| {
            let from_env: u8 = *illicit::Env::expect();
            assert_eq!(from_env, first_byte);
        });

        assert!(illicit::Env::get::<u8>().is_none());
        illicit::child_env!(u8 => first_byte).enter(|| {
            topo::call(|| runtime.run_once());
        });
        assert!(illicit::Env::get::<u8>().is_none());
    }

    #[test]
    fn oneshot_runtime() {
        let rev = Runtime::oneshot(|| Revision::current());
        assert_eq!(rev, Revision(1), "only one iteration should occur");
    }

    #[test]
    fn revision_bound_runtime() {
        let mut rt = Runtime::new(|| Revision::current());
        let rev = rt.run_until_at_least_revision(Revision(4));
        assert_eq!(rev, Revision(4), "exactly 4 iterations should occur");

        let rev = rt.run_until_at_least_revision(Revision(4));
        assert_eq!(rev, Revision(5), "exactly 1 more iteration should occur");
    }

    #[test]
    fn readiness_bound_runtime() {
        let mut rt = Runtime::new(|| Revision::current());
        let rev = rt.run_until_ready(|rev| {
            if rev == Revision(3) {
                Poll::Ready(rev)
            } else {
                Poll::Pending
            }
        });
        assert_eq!(rev, Revision(3), "exactly 3 iterations should occur");
    }
}
