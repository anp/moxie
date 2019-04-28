use {crate::prelude::*, std::collections::HashMap, stdweb::*, tokio_trace::*};

#[derive(Clone, Debug)]
pub(crate) struct Weaver {
    root_node: web::Node,
    root_scope: ScopeId,
    children_ids: HashMap<ScopeId, Vec<ScopeId>>,
    children: HashMap<ScopeId, Vec<web::Node>>,
}

impl Weaver {
    pub(crate) fn attached_to(root_scope: ScopeId, root_node: web::Node) -> Self {
        info!("creating weaver");
        Self {
            root_node,
            root_scope,
            children_ids: Default::default(),
            children: Default::default(),
        }
    }

    pub(crate) fn weave(self) {
        unimplemented!()
    }
}

impl moxie::Witness for Weaver {
    type Node = web::Node;

    fn see_component(&mut self, id: ScopeId, parent: ScopeId, nodes: &[Self::Node]) {
        self.children.insert(id, nodes.to_vec());
        self.children_ids.entry(parent).or_default().push(id);
    }
}
