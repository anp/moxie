use {
    crate::sys,
    wasm_bindgen::{prelude::*, JsCast},
};

#[derive(Clone)]
pub enum Node {
    Concrete(sys::Node),
    #[cfg(feature = "rsdom")]
    Virtual(Rc<VirtNode>),
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Concrete(s), Node::Concrete(o)) => s.is_same_node(Some(o)),
            #[cfg(rsdom)]
            (Node::Virtual(s), Node::Virtual(o)) => s == o,
            _ => unreachable!("if moxie-dom is comparing two different types of nodes...uh-oh."),
        }
    }
}

impl From<sys::Node> for Node {
    fn from(e: sys::Node) -> Self {
        Node::Concrete(e)
    }
}

impl From<sys::Element> for Node {
    fn from(e: sys::Element) -> Self {
        Node::Concrete(e.into())
    }
}

impl From<sys::Text> for Node {
    fn from(e: sys::Text) -> Self {
        Node::Concrete(e.into())
    }
}

#[cfg(feature = "rsdom")]
use {
    html5ever::rcdom::{Node as VirtNode, NodeData as VirtNodeData},
    std::{cell::RefCell, rc::Rc},
};

#[cfg(feature = "rsdom")]
impl From<Rc<VirtNode>> for Node {
    fn from(e: Rc<VirtNode>) -> Self {
        Node::Virtual(e)
    }
}

impl Node {
    fn expect_concrete(&self) -> &sys::Node {
        match self {
            Node::Concrete(n) => n,
            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => panic!("expected a Node::Concrete, found a Node::Virtual"),
        }
    }

    #[cfg(feature = "rsdom")]
    fn expect_virtual(&self) -> &Rc<VirtNode> {
        match self {
            Node::Concrete(_) => panic!("expected a Node::Virtual, found a Node::Concrete"),
            Node::Virtual(n) => n,
        }
    }

    pub(crate) fn create_element(&self, ty: &str) -> Self {
        match self {
            Node::Concrete(_) => crate::document().create_element(ty).unwrap().into(),
            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => VirtNode::new(VirtNodeData::Element {
                name: unimplemented!(),
                attrs: RefCell::new(vec![]),
                template_contents: None,
                mathml_annotation_xml_integration_point: false,
            })
            .into(),
        }
    }

    pub(crate) fn create_text_node(&self, contents: &str) -> Self {
        match self {
            Node::Concrete(_) => crate::document().create_text_node(contents).into(),
            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => {
                let contents = RefCell::new(contents.into());
                VirtNode::new(VirtNodeData::Text { contents }).into()
            }
        }
    }

    pub(crate) fn next_sibling(&self) -> Option<Self> {
        match self {
            Node::Concrete(e) => e.next_sibling().map(Node::Concrete),
            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => unimplemented!(),
        }
    }

    pub(crate) fn first_child(&self) -> Option<Self> {
        match self {
            Node::Concrete(e) => e.first_child().map(Node::Concrete),
            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => v.children.borrow().get(0).map(|n| n.clone().into()),
        }
    }

    pub(crate) fn remove_child(&self, child: &Self) -> Option<Self> {
        match self {
            Node::Concrete(e) => e
                .remove_child(child.expect_concrete())
                .ok()
                .map(Node::Concrete),
            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => unimplemented!(),
        }
    }

    pub(crate) fn append_child(&self, child: &Self) {
        match self {
            Node::Concrete(e) => e.append_child(child.expect_concrete()),
            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => unimplemented!(),
        };
    }

    pub(crate) fn replace_child(&self, new_child: &Node, existing: &Node) {
        match self {
            Node::Concrete(e) => {
                e.replace_child(new_child.expect_concrete(), existing.expect_concrete())
            }
            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => unimplemented!(),
        };
    }

    pub(crate) fn set_attribute(&self, name: &str, value: &str) {
        match self {
            Node::Concrete(e) => {
                let e: &sys::Element = e.dyn_ref().unwrap();
                e.set_attribute(name, value).unwrap()
            }
            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => unimplemented!(),
        };
    }

    pub(crate) fn remove_attribute(&self, name: &str) {
        match self {
            Node::Concrete(e) => {
                let e: &sys::Element = e.dyn_ref().unwrap();
                e.remove_attribute(name).ok()
            }
            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => unimplemented!(),
        };
    }
}
