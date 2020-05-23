//! An implementation of `augdom`'s APIs on top of an in-memory emulation of the
//! web's DOM.

use super::*;
use futures::channel::mpsc::UnboundedReceiver;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Writer as XmlWriter,
};
use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    rc::{Rc, Weak},
};

/// A node in the "virtual DOM" implemented in `rsdom`.
pub struct VirtNode {
    parent: Cell<Option<Weak<VirtNode>>>,
    children: RefCell<Vec<Rc<VirtNode>>>,
    data: VirtData,
}

/// Create a new virtual element of the provided type.
pub fn create_element(ty: &str) -> Rc<VirtNode> {
    Rc::new(VirtNode {
        parent: Cell::new(None),
        children: RefCell::new(vec![]),
        data: VirtData::Elem { tag: ty.to_string(), attrs: RefCell::new(vec![]) },
    })
}

impl crate::Dom for Rc<VirtNode> {
    type MutationRecord = Mutation;
    type Nodes = Vec<Self>;
    type Observer = UnboundedReceiver<Vec<Mutation>>;

    fn write_xml<W: Write>(&self, writer: &mut XmlWriter<W>) {
        match &self.data {
            VirtData::Elem { tag, attrs } => {
                writer
                    .write_event(Event::Start(
                        BytesStart::borrowed_name(tag.as_bytes()).with_attributes(
                            attrs.borrow().iter().map(|(n, v)| (n.as_str(), v.as_str())),
                        ),
                    ))
                    .expect("writing start of element");

                for child in self.children.borrow().iter() {
                    child.write_xml(writer);
                }

                writer
                    .write_event(Event::End(BytesEnd::borrowed(tag.as_bytes())))
                    .expect("writing start of element");
            }
            VirtData::Text(t) => {
                writer
                    .write_event(Event::Text(BytesText::from_plain_str(&t)))
                    .expect("writing text node");
            }
        }
    }

    fn create_element(&self, ty: &str) -> Rc<VirtNode> {
        create_element(ty)
    }

    fn create_text_node(&self, contents: &str) -> Rc<VirtNode> {
        Rc::new(VirtNode {
            parent: Cell::new(None),
            children: RefCell::new(vec![]),
            data: VirtData::Text(contents.to_string()),
        })
    }

    fn first_child(&self) -> Option<Rc<VirtNode>> {
        self.children.borrow().get(0).cloned()
    }

    fn next_sibling(&self) -> Option<Rc<VirtNode>> {
        let parent = self.parent.replace(None);
        parent.and_then(|w| w.upgrade()).and_then(|parent| {
            self.parent.replace(Some(Rc::downgrade(&parent)));

            // loop over our parent's children and see if we can find ourselves
            let children = parent.children.borrow();
            for (i, child) in children.iter().enumerate() {
                if Rc::ptr_eq(child, self) {
                    // we found ourselves! look one ahead of us and return them
                    return children.get(i + 1).cloned();
                }
            }
            None
        })
    }

    fn append_child(&self, new_child: &Self) {
        self.children.borrow_mut().push(new_child.clone());
        new_child.parent.set(Some(Rc::downgrade(self)));
    }

    fn remove_child(&self, to_remove: &Self) -> Option<Self> {
        let parent = self.parent.replace(None);
        parent.and_then(|w| w.upgrade()).and_then(|parent| {
            self.parent.replace(Some(Rc::downgrade(&parent)));

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

    fn replace_child(&self, new_child: &Self, existing: &Self) {
        if let Some(parent) = self.parent.replace(None).and_then(|w| w.upgrade()) {
            self.parent.replace(Some(Rc::downgrade(&parent)));

            let mut replace_idx = None;
            for (i, child) in parent.children.borrow().iter().enumerate() {
                if Rc::ptr_eq(child, existing) {
                    replace_idx = Some(i);
                    break;
                }
            }

            parent.children.borrow_mut()[replace_idx.unwrap()] = new_child.clone();
            new_child.parent.set(Some(Rc::downgrade(&parent)));
        }
    }

    fn get_attribute(&self, name: &str) -> Option<String> {
        match &self.data {
            VirtData::Text(_) => None,
            VirtData::Elem { tag: _, attrs } => attrs
                .borrow()
                .iter()
                .find_map(|(attr, value)| if attr == name { Some(value.clone()) } else { None }),
        }
    }

    fn set_attribute(&self, name: &str, value: &str) {
        let mut attrs = match &self.data {
            VirtData::Elem { ref attrs, .. } => attrs.borrow_mut(),
            data => panic!("expected VirtData::Elem, found {:?}", data),
        };

        if let Some(existing) = attrs.iter_mut().find(|(n, _)| n == name) {
            existing.1 = value.to_string();
        } else {
            attrs.push((name.to_string(), value.to_string()));
        }
    }

    fn remove_attribute(&self, name: &str) {
        let mut attrs = match &self.data {
            VirtData::Elem { ref attrs, .. } => attrs.borrow_mut(),
            data => panic!("expected VirtData::Elem, found {:?}", data),
        };
        attrs.retain(|(n, _)| n != name);
    }

    fn get_inner_text(&self) -> String {
        todo!("traverse the virtnode tree and accumulate text");
    }

    fn dispatch<E: crate::event::Event>(&self) {
        todo!("...need to add event handling to rsdom");
    }

    fn query_selector(&self, _selectors: &str) -> Option<Self> {
        // TODO(#119) implement
        todo!("still need to integrate selectors crate")
    }

    fn query_selector_all(&self, _selectors: &str) -> Self::Nodes {
        // TODO(#119) implement
        todo!("still need to integrate selectors crate")
    }

    fn observe_mutations(&self) -> Self::Observer {
        todo!("still need to do...this")
    }
}

/// The data of a node in the virtual DOM tree.
#[derive(Debug)]
pub enum VirtData {
    /// A virtual element.
    Elem {
        /// The element's tag.
        tag: String,
        /// The element's attributes.
        attrs: RefCell<Vec<(String, String)>>,
    },
    /// A virtual text node.
    Text(String),
}

impl From<Rc<VirtNode>> for Node {
    fn from(e: Rc<VirtNode>) -> Self {
        Node::Virtual(e)
    }
}

impl Node {
    /// Returns a reference to a virtual node, panics if this is a concrete
    /// node.
    pub fn expect_virtual(&self) -> &Rc<VirtNode> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(_) => panic!("expected a Node::Virtual, found a Node::Concrete"),
            Node::Virtual(n) => n,
        }
    }
}

/// Tracks a mutation in the virtual DOM tree. Currently unimplemented.
pub struct Mutation {}
