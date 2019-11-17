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

use {crate::memo::MemoStore, std::task::Waker, tracing::*};

/// Revisions measure moxie's notion of time passing. Each [`Runtime`] increments its Revision
/// on every iteration. [`crate::Commit`]s to state variables are annotated with the Revision
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
pub struct Runtime<Root, Out>
where
    Root: FnMut() -> Out,
{
    revision: Revision,
    store: MemoStore,
    root: Root,
    wk: Waker,
    span: Span,
}

impl<Root, Out> Runtime<Root, Out>
where
    Root: FnMut() -> Out,
{
    /// Construct a new [`Runtime`] with blank storage and with a no-op waker for state changes.
    pub fn new(root: Root) -> Self {
        let span = trace_span!("runtime", rev = 0);
        Self {
            span,
            revision: Revision(0),
            store: MemoStore::default(),
            root,
            wk: futures::task::noop_waker(),
        }
    }

    /// The current revision of the runtime, or how many times `run_once` has been invoked.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// Runs the root closure once with access to memoization storage, increments the runtime's
    /// `Revision`, and drops any memoized values which were not marked `Liveness::Live`.
    pub fn run_once(&mut self) -> Out {
        self.revision.0 += 1;
        self.span.record("rev", &self.revision.0);
        let span = self.span.clone();
        let _entered = span.enter();

        let ret = illicit::child_env! {
            MemoStore => self.store.clone(),
            Revision => self.revision,
            RunLoopWaker => RunLoopWaker(self.wk.clone())
        }
        .enter(|| topo::call!((self.root)()));

        self.store.gc();
        ret
    }

    /// Sets the [`std::task::Waker`] which will be called when state variables receive commits. By
    /// default the runtime no-ops on a state change, which is probably the desired behavior if
    /// the embedding system will call `Runtime::run_once` on a regular interval regardless.
    pub fn set_state_change_waker(&mut self, wk: Waker) -> &mut Self {
        self.wk = wk;
        self
    }
}

/// Responsible for waking the Runtime task. Because the topo environment is namespaced by type,
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
            topo::call!(runtime.run_once());
        });
        assert!(illicit::Env::get::<u8>().is_none());
    }
}
