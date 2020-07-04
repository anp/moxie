//! Text nodes in the DOM.

use crate::{
    interfaces::content_categories::{FlowContent, PhrasingContent},
    memo_node::MemoNode,
};
use augdom::Dom;
use moxie::memo;

/// Create a [DOM text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
/// This is normally called by the `moxie::mox!` macro.
#[topo::nested]
#[illicit::from_env(parent: &MemoNode)]
pub fn text(s: impl AsRef<str>) -> Text {
    let text_node = memo(s.as_ref(), |s| parent.raw_node().create_text_node(s));
    Text(MemoNode::new(text_node))
}

/// A text node in the DOM.
#[must_use = "needs to be bound to a parent"]
pub struct Text(MemoNode);

impl crate::interfaces::node::sealed::Memoized for Text {
    fn node(&self) -> &MemoNode {
        &self.0
    }
}
impl crate::interfaces::node::Node for Text {}

impl FlowContent for Text {}
impl PhrasingContent for Text {}
