use {
    crate::prelude::*,
    stdweb::{traits::*, *},
};

#[derive(Clone, Debug, Default)]
pub(crate) struct Weaver;

impl moxie::Witness for Weaver {
    type Node = web::Node;

    fn see(&mut self, node: &Self::Node, parent: Option<&Self::Node>) {
        js!(console.log("seeing node", @{node}, "with parent", @{&parent}););

        while let Some(child) = node.first_child() {
            node.remove_child(&child).unwrap();
        }

        if let Some(parent) = parent {
            parent.append_child(node);
        }
    }
}
