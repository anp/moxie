use {
    crate::prelude::*,
    std::collections::HashMap,
    stdweb::{traits::*, *},
    tokio_trace::*,
};

#[derive(Clone, Debug)]
pub(crate) struct Weaver {
    root_scope: ScopeId,
    root_node: web::Node,
    parents: HashMap<ScopeId, ScopeId>,
    output: HashMap<ScopeId, web::Node>,
}

impl Weaver {
    pub(crate) fn attached_to(root_scope: ScopeId, root_node: web::Node) -> Self {
        info!("creating weaver");
        let mut output: HashMap<_, _> = Default::default();
        output.insert(root_scope, root_node.clone());
        let parents = Default::default();
        Self {
            output,
            parents,
            root_node,
            root_scope,
        }
    }
}

impl moxie::Witness for Weaver {
    type Node = web::Node;

    fn see(&mut self, id: ScopeId, parent: ScopeId, node: &Self::Node) {
        info!("seeing {:?} with parent {:?}", id, parent);
        self.output.insert(id, node.to_owned());
        self.parents.insert(id, parent);
        info!("current tree contents: {:#?}", self);

        let mut potential_parent = parent;
        let dom_parent;

        'find_parent: loop {
            if let Some(parent_node) = self.output.get(&potential_parent) {
                dom_parent = parent_node;
                break 'find_parent;
            }

            potential_parent = self.parents[&potential_parent];
        }

        dom_parent.append_child(node);
    }
}
