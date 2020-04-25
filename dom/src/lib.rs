//! Tools for declaratively constructing and incrementally updating HTML DOM
//! trees on the web. Based on the [`moxie`] UI runtime.

#![deny(clippy::all, missing_docs)]
#![feature(track_caller)]

use augdom::Node;

pub mod elements;
pub mod embed;
pub mod interfaces;
pub mod memo_node;

/// A module for glob-importing the most commonly used moxie-dom items.
pub mod prelude {
    #[cfg(feature = "webdom")]
    pub use augdom::{document, sys};
    pub use augdom::{event, Dom};
    pub use moxie::{mox, prelude::*};

    pub use crate::{
        interfaces::{
            element::Element, event_target::EventTarget, html_element::HtmlElement, node::Node,
        },
        memo_node::text,
    };
}

/// The "boot sequence" for a moxie-dom instance creates a
/// [crate::embed::WebRuntime] with the provided arguments and begins scheduling
/// its execution with `requestAnimationFrame` on state changes.
///
/// If you need to schedule your root function more or less frequently than when
/// state variables are updated, see the [embed](crate::embed) module for
/// granular control over scheduling.
///
/// In terms of the embed module's APIs, this function constructs a new
/// [`WebRuntime`](crate::embed::WebRuntime) and begins scheduling it with an
/// [`AnimationFrameScheduler`](raf::AnimationFrameScheduler) which requests an
/// animation frame only when there are updates to state variables.
///
/// Requires the `webdom` feature.
#[cfg(any(feature = "webdom", doc))]
pub fn boot(new_parent: impl Into<Node>, root: impl FnMut() + 'static) {
    embed::WebRuntime::new(new_parent.into(), root).animation_frame_scheduler().run_on_wake();
}

/// Runs the provided closure once and produces a prettified HTML string from
/// the contents.
///
/// If you need more control over the output of the HTML, see the implementation
/// of this function.
///
/// Requires the `rsdom` feature.
#[cfg(any(feature = "rsdom", doc))]
pub fn render_html(root: impl FnMut() + 'static) -> String {
    use augdom::Dom;

    let (mut tester, root) = embed::WebRuntime::in_rsdom_div(root);
    tester.run_once();
    let outer = augdom::Node::Virtual(root).pretty_outer_html(2);
    // because we use the indented version, we know that only at the top and bottom
    // is what we want
    outer.lines().filter(|l| *l != "<div>" && *l != "</div>").map(|l| l.split_at(2).1).fold(
        String::new(),
        |mut fragment, line| {
            if !fragment.is_empty() {
                fragment.push('\n');
            }
            fragment.push_str(line);
            fragment
        },
    )
}
