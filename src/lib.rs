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
pub trait Component {
    /// Defines the `Component` at a given point in time.
    ///
    /// TODO explain "right now" declaration
    /// TODO explain memoization of this call
    /// TODO explain show macro
    fn contents(&self);
}

#[topo::bound]
pub fn show(component: impl Component + Debug + PartialEq + 'static) {
    let show_span = once!(|| trace_span!("show component"));
    let state_revision = once!(|| RevisionChain::new());

    let _in_span = show_span.enter();
    topo::call!(
        {
            memo!((state_revision.current(), component), |(rev, c)| {
                trace!({ props = ?c }, "showing");
                c.contents();
            });
        },
        env! {
            RevisionChain => state_revision.clone(),
        }
    );
}
