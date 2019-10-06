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
    html5ever::{
        rcdom::{Node as VirtNode, NodeData as VirtNodeData},
        LocalName, Namespace, QualName,
    },
    std::{
        cell::RefCell,
        rc::{Rc, Weak},
    },
};

impl Node {
    fn expect_concrete(&self) -> &sys::Node {
        match self {
            Node::Concrete(n) => n,

            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => panic!("expected a Node::Concrete, found a Node::Virtual"),
        }
    }

    pub(crate) fn create_element(&self, ty: &str) -> Self {
        match self {
            Node::Concrete(_) => crate::document().create_element(ty).unwrap().into(),

            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => Node::Virtual(rsdom::create_element(ty)),
        }
    }

    pub(crate) fn create_text_node(&self, contents: &str) -> Self {
        match self {
            Node::Concrete(_) => crate::document().create_text_node(contents).into(),

            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => Node::Virtual(rsdom::create_text_node(contents)),
        }
    }

    pub(crate) fn first_child(&self) -> Option<Self> {
        match self {
            Node::Concrete(e) => e.first_child().map(Node::Concrete),

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => rsdom::first_child(v).map(Node::Virtual),
        }
    }

    pub(crate) fn append_child(&self, child: &Self) {
        match self {
            Node::Concrete(e) => {
                let _ = e.append_child(child.expect_concrete()).unwrap();
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => rsdom::append_child(v, child.expect_virtual()),
        };
    }

    pub(crate) fn next_sibling(&self) -> Option<Self> {
        match self {
            Node::Concrete(e) => e.next_sibling().map(Node::Concrete),

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => rsdom::next_sibling(v).map(Node::Virtual),
        }
    }

    pub(crate) fn remove_child(&self, to_remove: &Self) -> Option<Self> {
        match self {
            Node::Concrete(e) => e
                .remove_child(to_remove.expect_concrete())
                .ok()
                .map(Node::Concrete),

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => {
                rsdom::remove_child(v, to_remove.expect_virtual()).map(Node::Virtual)
            }
        }
    }

    pub(crate) fn replace_child(&self, new_child: &Node, existing: &Node) {
        match self {
            Node::Concrete(e) => {
                e.replace_child(new_child.expect_concrete(), existing.expect_concrete())
                    .unwrap();
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => {
                rsdom::replace_child(v, new_child.expect_virtual(), existing.expect_virtual())
            }
        };
    }

    pub(crate) fn set_attribute(&self, name: &str, value: &str) {
        match self {
            Node::Concrete(e) => {
                let e: &sys::Element = e.dyn_ref().unwrap();
                e.set_attribute(name, value).unwrap()
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => rsdom::set_attribute(v, name, value),
        };
    }

    pub(crate) fn remove_attribute(&self, name: &str) {
        match self {
            Node::Concrete(e) => {
                let e: &sys::Element = e.dyn_ref().unwrap();
                e.remove_attribute(name).ok();
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => rsdom::remove_attribute(v, name),
        };
    }
}

#[cfg(feature = "rsdom")]
pub(crate) mod rsdom {
    use {
        super::*,
        html5ever::{
            rcdom::{Node as VirtNode, NodeData as VirtNodeData},
            tree_builder::Attribute,
            LocalName, Namespace, QualName,
        },
        std::{
            cell::RefCell,
            rc::{Rc, Weak},
        },
    };

    impl From<Rc<VirtNode>> for Node {
        fn from(e: Rc<VirtNode>) -> Self {
            Node::Virtual(e)
        }
    }

    pub(super) fn create_element(ty: &str) -> Rc<VirtNode> {
        VirtNode::new(VirtNodeData::Element {
            name: QualName::new(
                None,                    //prefix
                Namespace::from("html"), // TODO attempt to parse other namespace from ty
                LocalName::from(ty),
            ),
            attrs: RefCell::new(vec![]),
            template_contents: None,
            mathml_annotation_xml_integration_point: false,
        })
    }

    pub(super) fn create_text_node(contents: &str) -> Rc<VirtNode> {
        VirtNode::new(VirtNodeData::Text {
            contents: RefCell::new(contents.into()),
        })
    }

    pub(super) fn first_child(v: &Rc<VirtNode>) -> Option<Rc<VirtNode>> {
        v.children.borrow().get(0).map(|n| n.clone())
    }

    pub(super) fn next_sibling(v: &Rc<VirtNode>) -> Option<Rc<VirtNode>> {
        // these lines are a silly dance to get a deref-able parent pointer from rcdom
        let parent = v.parent.replace(None);
        parent.and_then(|w| w.upgrade()).and_then(|parent| {
            v.parent.replace(Some(Rc::downgrade(&parent)));

            // loop over our parent's children and see if we can find ourselves
            let children = parent.children.borrow();
            for (i, child) in children.iter().enumerate() {
                if Rc::ptr_eq(child, v) {
                    // we found ourselves! look one ahead of us and return them
                    return children.get(i + 1).map(|c| c.clone());
                }
            }
            None
        })
    }

    pub(super) fn append_child(v: &Rc<VirtNode>, new_child: &Rc<VirtNode>) {
        v.children.borrow_mut().push(new_child.clone())
    }

    pub(super) fn remove_child(v: &Rc<VirtNode>, to_remove: &Rc<VirtNode>) -> Option<Rc<VirtNode>> {
        let parent = v.parent.replace(None);
        parent.and_then(|w| w.upgrade()).and_then(|parent| {
            v.parent.replace(Some(Rc::downgrade(&parent)));

            let mut remove_idx = None;
            for (i, child) in parent.children.borrow().iter().enumerate() {
                if Rc::ptr_eq(child, to_remove) {
                    remove_idx = Some(i);
                    break;
                }
            }

            remove_idx.map(|i| parent.children.borrow_mut().remove(i))
        })
    }

    pub(super) fn replace_child(
        v: &Rc<VirtNode>,
        new_child: &Rc<VirtNode>,
        existing: &Rc<VirtNode>,
    ) {
        if let Some(parent) = v.parent.replace(None).and_then(|w| w.upgrade()) {
            v.parent.replace(Some(Rc::downgrade(&parent)));

            let mut replace_idx = None;
            for (i, child) in parent.children.borrow().iter().enumerate() {
                if Rc::ptr_eq(child, existing) {
                    replace_idx = Some(i);
                    break;
                }
            }

            if let Some(i) = replace_idx {
                let children = &mut *parent.children.borrow_mut();
                std::mem::replace(&mut children[i], new_child.clone());
            }
        }
    }

    pub(super) fn set_attribute(v: &Rc<VirtNode>, name: &str, value: &str) {
        let mut attrs = match &v.data {
            VirtNodeData::Element { ref attrs, .. } => attrs.borrow_mut(),
            data @ _ => panic!("expected NodeData::Element, found {:?}", data),
        };

        let new_value = value.to_string().into();

        if let Some(existing) = attrs.iter_mut().find(|a| &*a.name.local == name) {
            existing.value = new_value;
        } else {
            attrs.push(Attribute {
                name: QualName::new(
                    None, //prefix
                    Namespace::from(""),
                    LocalName::from(name),
                ),
                value: new_value,
            });
        }
    }

    pub(super) fn remove_attribute(v: &Rc<VirtNode>, name: &str) {
        let mut attrs = match &v.data {
            VirtNodeData::Element { ref attrs, .. } => attrs.borrow_mut(),
            data @ _ => panic!("expected NodeData::Element, found {:?}", data),
        };
        attrs.retain(|a| &*a.name.local != name);
    }

    impl Node {
        pub(super) fn expect_virtual(&self) -> &Rc<VirtNode> {
            match self {
                Node::Concrete(_) => panic!("expected a Node::Virtual, found a Node::Concrete"),
                Node::Virtual(n) => n,
            }
        }
    }
}
