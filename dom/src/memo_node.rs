//! Nodes which memoize mutations.

use augdom::{Dom, Node};

use std::{
    cell::Cell,
    fmt::{Debug, Formatter, Result as FmtResult},
};

use super::prelude::*;

/// Create and mount a [DOM text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
/// This is normally called by the `moxie::mox!` macro.
#[topo::nested]
#[illicit::from_env(parent: &MemoNode)]
pub fn text(s: impl ToString) {
    // TODO(#99) avoid allocating this extra string when it hasn't changed
    let text_node = memo(s.to_string(), |s| parent.node.create_text_node(s));
    parent.ensure_child_attached(&text_node);
}

/// A topologically-nested "incremental smart pointer" for an HTML element.
///
/// Created during execution of the (element) macro and the element-specific
/// wrappers. Offers a "stringly-typed" API for mutating the contained DOM
/// nodes, adhering fairly closely to the upstream web specs.
pub struct MemoNode {
    curr: Cell<Option<Node>>,
    node: Node,
}

impl MemoNode {
    pub(crate) fn new(node: Node) -> Self {
        Self { curr: Cell::new(None), node }
    }

    pub(crate) fn raw_node(&self) -> &Node {
        &self.node
    }

    pub(crate) fn ensure_child_attached(&self, new_child: &Node) {
        let prev_sibling = self.curr.replace(Some(new_child.clone()));

        let existing = if prev_sibling.is_none() {
            self.node.first_child()
        } else {
            prev_sibling.and_then(|p| p.next_sibling())
        };

        if let Some(ref existing) = existing {
            if existing != new_child {
                self.node.replace_child(new_child, existing);
            }
        } else {
            self.node.append_child(new_child);
        }
    }

    /// Declare the inner contents of the element, usually declaring children
    /// within the inner scope. After any children have been run and their
    /// nodes attached, this clears any trailing child nodes to ensure the
    /// element's children are correct per the latest declaration.
    #[topo::nested]
    pub fn inner<Ret>(&self, children: impl FnOnce() -> Ret) -> Ret {
        let elem = self.node.clone();
        let (ret, last_desired_child) =
            illicit::child_env!(MemoNode => MemoNode::new(self.node.clone())).enter(|| {
                // before this melement is dropped when the environment goes out of scope,
                // we need to get the last recorded child from this revision
                (children(), illicit::Env::expect::<MemoNode>().curr.replace(None))
            });

        // if there weren't any children declared this revision, we need to make sure we
        // clean up any from the last revision
        let mut next_to_remove =
            if let Some(c) = last_desired_child { c.next_sibling() } else { elem.first_child() };

        while let Some(to_remove) = next_to_remove {
            next_to_remove = to_remove.next_sibling();
            elem.remove_child(&to_remove).unwrap();
        }

        ret
    }
}

impl Debug for MemoNode {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("MemoNode").field("node", &self.node).finish()
    }
}
