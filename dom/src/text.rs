//! Text nodes in the DOM.

use crate::{
    interfaces::content_categories::{FlowContent, PhrasingContent},
    memo_node::MemoNode,
};
use moxie::prelude::*;

/// A text node in the DOM.
#[must_use = "needs to be bound to a parent"]
pub struct Text(MemoNode);

impl crate::interfaces::node::Node for Text {
    type Output = Self;

    fn build(self) -> Self {
        self
    }
}

impl crate::interfaces::node::sealed::Memoized for Text {
    fn node(&self) -> &MemoNode {
        &self.0
    }
}
/// Create a [DOM text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
/// This is normally called by the `moxie::mox!` macro.
#[topo::nested]
#[illicit::from_env(parent: &MemoNode)]
pub fn text(s: impl ToString) -> Text {
    // TODO(#99) avoid allocating this extra string when it hasn't changed
    let text_node = memo(s.to_string(), |s| parent.raw_node().create_text_node(s));
    Text(MemoNode::new(text_node))
}
