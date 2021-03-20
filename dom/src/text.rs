//! Text nodes in the DOM.

use crate::{
    cached_node::CachedNode,
    interfaces::{
        content_categories::{FlowContent, PhrasingContent},
        node::NodeBuilder,
    },
    prelude::*,
};
use moxie::cache;

/// Create a [DOM text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
/// This is normally called by the `moxie::mox!` macro.
#[topo::nested]
pub fn text(s: impl AsRef<str>) -> Text {
    let text_node = cache(s.as_ref(), |s| document().create_text_node(s));
    Text(CachedNode::new(text_node))
}

/// A text node in the DOM.
#[must_use = "needs to be bound to a parent"]
pub struct Text(CachedNode);

impl NodeBuilder for Text {
    type Target = Self;

    fn build(self) -> Self::Target {
        self
    }
}

impl crate::interfaces::node::sealed::Memoized for Text {
    fn node(&self) -> &CachedNode {
        &self.0
    }
}
impl crate::interfaces::node::NodeWrapper for Text {}

impl FlowContent for Text {}
impl PhrasingContent for Text {}
