#[cfg(feature = "webdom")]
use {crate::sys, wasm_bindgen::JsCast};

#[cfg(feature = "rsdom")]
use {rsdom::VirtNode, std::rc::Rc};

#[derive(Clone)]
pub enum Node {
    #[cfg(feature = "webdom")]
    Concrete(sys::Node),

    #[cfg(feature = "rsdom")]
    Virtual(Rc<VirtNode>),
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            #[cfg(feature = "webdom")]
            (Node::Concrete(s), Node::Concrete(o)) => s.is_same_node(Some(o)),

            #[cfg(feature = "rsdom")]
            (Node::Virtual(s), Node::Virtual(o)) => Rc::ptr_eq(s, o),

            #[cfg(all(feature = "webdom", feature = "rsdom"))]
            _ => unreachable!("if moxie-dom is comparing two different types of nodes...uh-oh."),
        }
    }
}

impl Node {
    pub(crate) fn create_element(&self, ty: &str) -> Self {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(_) => crate::document().create_element(ty).unwrap().into(),

            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => Node::Virtual(rsdom::create_element(ty)),
        }
    }

    pub(crate) fn create_text_node(&self, contents: &str) -> Self {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(_) => crate::document().create_text_node(contents).into(),

            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => Node::Virtual(rsdom::create_text_node(contents)),
        }
    }

    pub(crate) fn first_child(&self) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(e) => e.first_child().map(Node::Concrete),

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => rsdom::first_child(v).map(Node::Virtual),
        }
    }

    pub(crate) fn append_child(&self, child: &Self) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(e) => {
                let _ = e.append_child(child.expect_concrete()).unwrap();
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => rsdom::append_child(v, child.expect_virtual()),
        };
    }

    pub(crate) fn next_sibling(&self) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(e) => e.next_sibling().map(Node::Concrete),

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => rsdom::next_sibling(v).map(Node::Virtual),
        }
    }

    pub(crate) fn remove_child(&self, to_remove: &Self) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
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
            #[cfg(feature = "webdom")]
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
            #[cfg(feature = "webdom")]
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
            #[cfg(feature = "webdom")]
            Node::Concrete(e) => {
                let e: &sys::Element = e.dyn_ref().unwrap();
                e.remove_attribute(name).ok();
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(v) => rsdom::remove_attribute(v, name),
        };
    }
}

#[cfg(feature = "webdom")]
pub(crate) mod webdom {
    use {
        super::Node,
        crate::event::Event,
        wasm_bindgen::{prelude::*, JsCast},
        web_sys as sys,
    };

    pub struct Callback {
        cb: Closure<dyn FnMut(JsValue)>,
    }

    impl Callback {
        pub fn new<Ev>(mut cb: impl FnMut(Ev) + 'static) -> Self
        where
            Ev: Event,
        {
            let cb = Closure::wrap(Box::new(move |ev: JsValue| {
                let ev: Ev = ev.dyn_into().unwrap();
                cb(ev);
            }) as Box<dyn FnMut(JsValue)>);
            Self { cb }
        }

        pub fn as_fn(&self) -> &js_sys::Function {
            self.cb.as_ref().unchecked_ref()
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

    impl Node {
        pub(super) fn expect_concrete(&self) -> &sys::Node {
            match self {
                Node::Concrete(n) => n,

                #[cfg(feature = "rsdom")]
                Node::Virtual(_) => panic!("expected a Node::Concrete, found a Node::Virtual"),
            }
        }
    }
}

#[cfg(feature = "rsdom")]
pub mod rsdom {
    use {
        super::*,
        quick_xml::{
            events::{BytesEnd, BytesStart, BytesText, Event},
            Writer as XmlWriter,
        },
        std::{
            cell::{Cell, RefCell},
            fmt::{Debug, Display, Formatter, Result as FmtResult},
            io::{prelude::*, Cursor},
            rc::{Rc, Weak},
        },
    };

    pub struct VirtNode {
        parent: Cell<Option<Weak<VirtNode>>>,
        children: RefCell<Vec<Rc<VirtNode>>>,
        data: VirtData,
    }

    impl VirtNode {
        pub fn inner_html(&self) -> String {
            let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            {
                let mut writer = XmlWriter::new(&mut buf);
                self.write_xml(&mut writer);
            }
            String::from_utf8(buf.into_inner()).unwrap()
        }

        pub fn pretty_inner_html(&self) -> String {
            let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            {
                let mut writer = XmlWriter::new_with_indent(&mut buf, ' ' as u8, 2);
                self.write_xml(&mut writer);
            }
            String::from_utf8(buf.into_inner()).unwrap()
        }

        // TODO is there a way to pass the starting indentation down from a formatter?
        pub fn write_xml<W: Write>(&self, writer: &mut XmlWriter<W>) {
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

    impl Debug for VirtNode {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let s = if f.alternate() {
                self.pretty_inner_html()
            } else {
                self.inner_html()
            };
            f.write_str(&s)
        }
    }

    impl Display for VirtNode {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            f.write_str(&self.pretty_inner_html())
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
}
