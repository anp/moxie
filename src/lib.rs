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

#![deny(clippy::all, missing_docs, intra_doc_link_resolution_failure)]
#![feature(async_await, gen_future)]

#[macro_use]
mod memo;
mod state;

#[doc(hidden)]
pub use topo;
#[doc(inline)]
pub use {memo::*, state::*};

use {
    futures::Poll,
    std::future::Future,
    std::pin::Pin,
    std::task::{Context, Waker},
    topo::bound,
};

/// TODO explain a component...somehow
pub trait Component {
    /// Defines the `Component` at a given point in time.
    ///
    /// TODO explain "right now" declaration
    /// TODO explain memoization of this call
    /// TODO explain show macro
    fn contents(&self);
}

#[bound]
pub fn show(component: impl Component + PartialEq + 'static) {
    use crate::*;
    memo!(component, |c| c.contents());
}

/// Revisions measure moxie's notion of time passing. Each [`Runtime`] increments its Revision
/// on every iteration. [`Commit`]s to state variables are annotated with the Revision during which
/// they were made.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
/// The simplest possible Runtime does nothing and is only called once. Most practical usages of
/// the Runtime rely on its continued execution, however.
///
/// ```
/// let mut rt = moxie::Runtime::new(|| {});
/// rt.run_once();
/// assert_eq!(rt.revision(), moxie::Revision(1));
/// ```
pub struct Runtime<Root> {
    revision: Revision,
    store: MemoStore,
    root: Root,
    wk: Waker,
    // TODO add tasks executor
}

impl<Root> Runtime<Root>
where
    Root: FnMut(),
{
    /// Construct a new Runtime at revision 0 and blank storage.
    pub fn new(root: Root) -> Self {
        Self {
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

    /// Run a single iteration of the root closure with access to the [`Env`] provided by the
    /// [`Runtime`] . Increments the [`Revision`] counter of this [`Runtime`] by one.
    pub fn run_once(&mut self) {
        self.revision.0 += 1;

        topo::root!(
            (self.root)(),
            env! {
                MemoStore => self.store.clone(),
                Revision => self.revision,
                RunLoopWaker => RunLoopWaker(self.wk.clone()),
            }
        );
    }

    /// Sets the [`std::task::Waker`] which will be called when [`state::Var`]s receive commits.
    ///
    /// In the `Future` impl for `Runtime`, this is set to match the waker of the task to which
    /// the Runtime is bound. Other implementations may have integrations with systems that e.g.
    /// expect a callback to be enqueued, and those implementations should make a custom `Waker`
    /// using [`std::task::RawWaker`].
    pub fn set_state_change_waker(&mut self, wk: Waker) -> &mut Self {
        self.wk = wk;
        self
    }
}

/// A [`Runtime`] can be run as a `Future`, and is primarily used for testing as of writing.
impl<Root> Future for Runtime<Root>
where
    Root: FnMut() + Unpin,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        self.get_mut()
            .set_state_change_waker(cx.waker().clone())
            .run_once();
        Poll::Pending
    }
}

/// Responsible for waking the Runtime task. Because the topo environment is namespaced by type,
/// we create a newtype here so that other crates don't accidentally cause strange behavior by
/// overriding our access to it.
#[derive(Clone)]
struct RunLoopWaker(std::task::Waker);

impl RunLoopWaker {
    fn wake(&self) {
        self.0.wake_by_ref();
    }
}
