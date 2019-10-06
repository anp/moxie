use {
    super::*,
    quick_xml::{
        events::{BytesEnd, BytesStart, BytesText, Event},
        Writer as XmlWriter,
    },
    std::{
        cell::{Cell, RefCell},
        fmt::Debug,
        rc::{Rc, Weak},
    },
};

pub struct VirtNode {
    parent: Cell<Option<Weak<VirtNode>>>,
    children: RefCell<Vec<Rc<VirtNode>>>,
    data: VirtData,
}

impl crate::Xml for VirtNode {
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
}

#[derive(Debug)]
pub enum VirtData {
    Elem {
        tag: String,
        attrs: RefCell<Vec<(String, String)>>,
    },
    Text(String),
}

impl From<Rc<VirtNode>> for Node {
    fn from(e: Rc<VirtNode>) -> Self {
        Node::Virtual(e)
    }
}

pub fn create_element(ty: &str) -> Rc<VirtNode> {
    Rc::new(VirtNode {
        parent: Cell::new(None),
        children: RefCell::new(vec![]),
        data: VirtData::Elem {
            tag: ty.to_string(),
            attrs: RefCell::new(vec![]),
        },
    })
}

pub fn create_text_node(contents: &str) -> Rc<VirtNode> {
    Rc::new(VirtNode {
        parent: Cell::new(None),
        children: RefCell::new(vec![]),
        data: VirtData::Text(contents.to_string()),
    })
}

pub fn first_child(v: &Rc<VirtNode>) -> Option<Rc<VirtNode>> {
    v.children.borrow().get(0).map(|n| n.clone())
}

pub fn next_sibling(v: &Rc<VirtNode>) -> Option<Rc<VirtNode>> {
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

pub fn append_child(v: &Rc<VirtNode>, new_child: &Rc<VirtNode>) {
    v.children.borrow_mut().push(new_child.clone());
    new_child.parent.set(Some(Rc::downgrade(v)));
}

pub fn remove_child(v: &Rc<VirtNode>, to_remove: &Rc<VirtNode>) -> Option<Rc<VirtNode>> {
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

pub fn replace_child(v: &Rc<VirtNode>, new_child: &Rc<VirtNode>, existing: &Rc<VirtNode>) {
    if let Some(parent) = v.parent.replace(None).and_then(|w| w.upgrade()) {
        v.parent.replace(Some(Rc::downgrade(&parent)));

        let mut replace_idx = None;
        for (i, child) in parent.children.borrow().iter().enumerate() {
            if Rc::ptr_eq(child, existing) {
                replace_idx = Some(i);
                break;
            }
        }

        let children = &mut *parent.children.borrow_mut();
        std::mem::replace(&mut children[replace_idx.unwrap()], new_child.clone());
        new_child.parent.set(Some(Rc::downgrade(&parent)));
    }
}

pub fn set_attribute(v: &Rc<VirtNode>, name: &str, value: &str) {
    let mut attrs = match &v.data {
        VirtData::Elem { ref attrs, .. } => attrs.borrow_mut(),
        data @ _ => panic!("expected VirtData::Elem, found {:?}", data),
    };

    let new_value = value.to_string().into();

    if let Some(existing) = attrs.iter_mut().find(|(n, _)| n == name) {
        existing.1 = new_value;
    } else {
        attrs.push((name.to_string(), new_value));
    }
}

pub fn remove_attribute(v: &Rc<VirtNode>, name: &str) {
    let mut attrs = match &v.data {
        VirtData::Elem { ref attrs, .. } => attrs.borrow_mut(),
        data @ _ => panic!("expected VirtData::Elem, found {:?}", data),
    };
    attrs.retain(|(n, _)| n != name);
}

impl Node {
    pub(super) fn expect_virtual(&self) -> &Rc<VirtNode> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(_) => panic!("expected a Node::Virtual, found a Node::Concrete"),
            Node::Virtual(n) => n,
        }
    }
}
