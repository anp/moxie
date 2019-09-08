//! moxie is a toolkit for efficiently constructing and incrementally updating trees, describing
//! their structure and contents from the shape of a memoized function call graph.
//!
//! TODO example of a JSON tree, or graphviz, or something
//!
//! TODO diagram of tree transition over time
//!
//! # Memoization
//!
//! TODO
//!
//! # State
//!
//! TODO explain update lifecycle, pending values, commit/xact
//!
//! # Async Tasks & Actors
//!
//! TODO

#![forbid(unsafe_code)]
#![deny(clippy::all, intra_doc_link_resolution_failure)]
#![warn(missing_docs)]

#[macro_use]
mod memo;
mod state;

#[doc(hidden)]
pub use topo;
#[doc(inline)]
pub use {memo::*, state::*};

use {crate::MemoStore, std::task::Waker, tracing::*};

/// Revisions measure moxie's notion of time passing. Each [`Runtime`] increments its Revision
/// on every iteration. [`crate::Commit`]s to state variables are annotated with the Revision
/// during which they were made.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

impl Revision {
    /// Returns the current revision. Will return `Revision(0)` if called outside of a Runtime's
    /// execution.
    pub fn current() -> Self {
        if let Some(r) = topo::Env::get::<Revision>() {
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

/// A `Runtime` is the entry point of the moxie runtime environment. On each invocation of
/// `run_once`, it calls the root with which it was initialized. Typically this is invoked in a loop
/// which sleeps until the provided waker is invoked, as is the case in the `Future` implementation.
/// Usually root closure will cause some memoized side effect to the render environment in order to
/// produce a view of the input data. A Runtime's root closure will also transitively establish
/// event handlers, either via locally polled `Future`s or via the containing environment's
/// callback or event mechanisms.
///
/// While the Runtime may iterate very frequently (potentially more than once for any given output
/// frame), we use [topological memoization](crate::memo) to minimize the code run each time.
///
/// See the documentation for [`Runtime::run_once`] for details on the core loop body.
///
/// ## Minimal Example
///
/// ```
/// let mut rt = moxie::Runtime::new(|| {});
/// assert_eq!(rt.revision().0, 0);
/// for i in 1..10 {
///     rt.run_once();
///     assert_eq!(rt.revision(), moxie::Revision(i));
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
    /// Construct a new Runtime at revision 0 and blank storage.
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

    /// The current revision of the runtime.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// Run a single iteration of the root closure with access to the [`topo::Env`] provided by the
    /// [`Runtime`] . Increments the [`Revision`] counter of this [`Runtime`] by one.
    pub fn run_once(&mut self) -> Out {
        self.revision.0 += 1;
        self.span.record("rev", &self.revision.0);
        let _entered = self.span.enter();

        let ret = topo::root!(
            (self.root)(),
            env! {
                MemoStore => self.store.clone(),
                Revision => self.revision,
                RunLoopWaker => RunLoopWaker(self.wk.clone()),
            }
        );
        self.store.gc();
        ret
    }

    /// Sets the [`std::task::Waker`] which will be called when state variables receive commits. By
    /// default the runtime has a
    pub fn set_state_change_waker(&mut self, wk: Waker) -> &mut Self {
        self.wk = wk;
        self
    }
}

/// Responsible for waking the Runtime task. Because the topo environment is namespaced by type,
/// we create a newtype here so that other crates don't accidentally cause strange behavior by
/// overriding our access to it.
#[derive(Clone)]
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
            let from_env: u8 = *topo::Env::expect();
            assert_eq!(from_env, first_byte);
        });

        assert!(topo::Env::get::<u8>().is_none());
        topo::call!(
            {
                runtime.run_once();
            },
            env! {
                u8 => first_byte,
            }
        );
        assert!(topo::Env::get::<u8>().is_none());
    }
}
