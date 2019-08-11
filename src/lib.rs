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

#![deny(clippy::all, intra_doc_link_resolution_failure)]
#![warn(missing_docs)]
#![feature(async_await)]

#[macro_use]
mod memo;
mod nodes;
mod runtime;
mod state;

#[doc(hidden)]
pub use topo;
#[doc(inline)]
pub use {memo::*, nodes::*, runtime::*, state::*};

use {std::fmt::Debug, tracing::*};

/// TODO explain a component...somehow
pub trait Component: Debug + Sized {
    /// Defines the `Component` at a given point in time.
    ///
    /// TODO explain "right now" declaration
    /// TODO explain memoization of this call
    /// TODO explain show macro
    fn contents(self);
}

impl<I, C> Component for I
where
    I: IntoIterator<Item = C> + Debug,
    C: Component,
{
    fn contents(self) {
        for component in self {
            show!(component);
        }
    }
}

#[doc(hidden)]
#[topo::bound]
pub fn show_one(component: impl Component) {
    let show_span = once!(|| trace_span!("show component"));
    let state_revision = once!(|| RevisionChain::new());

    let _in_span = show_span.enter();
    topo::call!(
        {
            let rev = state_revision.current();
            show_span.record("rev", &rev);
            trace!({ props = ?component }, "showing");
            component.contents();
        },
        env! {
            RevisionChain => state_revision.clone(),
        }
    );
}

#[macro_export]
macro_rules! show {
    ($($child:expr),+) => {
        $(show_one!($child);)+
    };
}

pub trait Parent<Next: Component>: Component {
    // TODO can we express these automatically in terms of the Self<T> -> Self<SibList<T, Next>> xform?
    type Output: Component;

    fn child(self, next: Next) -> Self::Output;
}

pub fn sib_cons<Current, Next>(curr: Current, next: Next) -> SibList<Current, Next> {
    SibList { curr, next }
}

#[derive(Debug)]
pub struct SibList<Current, Next> {
    curr: Current,
    next: Next,
}

impl<Current, Next> Component for SibList<Current, Next>
where
    Current: Component,
    Next: Component,
{
    fn contents(self) {
        show!(self.curr);
        show!(self.next);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Empty;

impl Component for Empty {
    fn contents(self) {}
}

/// TODO find a better name. A wrapper for a closure which is called as if it were a child component, i.e. in its own topological point.
pub struct Clomp<F>(pub F);

impl<F> Clomp<F>
where
    F: FnOnce(),
{
    fn new(op: F) -> Self {
        Clomp(op)
    }
}

impl<F> Component for Clomp<F>
where
    F: FnOnce(),
{
    fn contents(self) {
        (self.0)()
    }
}

impl<F> std::fmt::Debug for Clomp<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("Clomp(TODO better output)"))
    }
}
