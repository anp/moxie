//! Nodes which memoize mutations.

use augdom::{Dom, Node};

use std::{
    cell::Cell,
    fmt::{Debug, Formatter, Result as FmtResult},
};

use super::prelude::*;

/// A topologically-nested "incremental smart pointer" for an HTML element.
///
/// Created during execution of the (element) macro and the element-specific
/// wrappers. Offers a "stringly-typed" API for mutating the contained DOM
/// nodes, adhering fairly closely to the upstream web specs.
pub struct MemoNode {
    last_child: Cell<Option<Node>>,
    node: Node,
}

impl MemoNode {
    pub(crate) fn new(node: Node) -> Self {
        Self { last_child: Cell::new(None), node }
    }

    pub(crate) fn raw_node(&self) -> &Node {
        &self.node
    }

    #[topo::nested]
    pub(crate) fn memo_attribute(&self, name: &str, value: String) {
        let name = name.to_owned();
        memo_with(
            value.to_string(),
            |v| {
                self.node.set_attribute(&name, v);
                scopeguard::guard(self.node.clone(), move |node| node.remove_attribute(&name))
            },
            |_| {},
        );
    }

    pub(crate) fn ensure_child_attached(&self, new_child: &Node) {
        let prev_sibling = self.last_child.replace(Some(new_child.clone()));

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

    pub(crate) fn remove_trailing_children(&self) {
        let last_desired_child = self.last_child.replace(None);

        // if there weren't any children declared this revision, we need to
        // make sure we clean up any from the last revision
        let mut next_to_remove = if let Some(c) = last_desired_child {
            // put back the last node we found this revision so this can be called multiple
            // times
            self.last_child.set(Some(c.clone()));
            c.next_sibling()
        } else {
            self.node.first_child()
        };

        while let Some(to_remove) = next_to_remove {
            next_to_remove = to_remove.next_sibling();
            self.node.remove_child(&to_remove).unwrap();
        }
    }
}

impl Debug for MemoNode {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("MemoNode").field("node", &self.node).finish()
    }
}
