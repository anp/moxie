//! An implementation of `augdom`'s APIs on top of the actual web using the
//! `web-sys` crate and `wasm-bindgen`.

use super::Node;
use crate::{document, event::Event};
use std::io::Write;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys as sys;

/// A dynamically-allocated closure that is callable from JavaScript.
pub struct Callback {
    cb: Closure<dyn FnMut(JsValue)>,
}

impl Callback {
    /// Allocate a new JS-compatible callback.
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

    /// Returns an reference to the underlying JS function. If the reference is
    /// used after this `Callback` is dropped it will panic.
    pub fn as_fn(&self) -> &js_sys::Function {
        self.cb.as_ref().unchecked_ref()
    }
}

impl crate::Dom for sys::Node {
    type Nodes = NodeList;

    fn write_xml<W: Write>(&self, writer: &mut quick_xml::Writer<W>) {
        use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
        if let Some(elem) = self.dyn_ref::<sys::Element>() {
            let name = elem.tag_name().to_lowercase();
            let attrs = elem.attributes();
            let attrs = (0..attrs.length())
                .filter_map(|i| attrs.item(i))
                .map(|a| (a.name(), a.value()))
                .collect::<Vec<_>>();

            writer
                .write_event(Event::Start(
                    BytesStart::borrowed_name(name.as_bytes())
                        .with_attributes(attrs.iter().map(|(n, v)| (n.as_str(), v.as_str()))),
                ))
                .expect("writing start of element");

            let children = sys::Node::child_nodes(elem.as_ref());
            for i in 0..children.length() {
                children.item(i).unwrap().write_xml(writer);
            }

            writer
                .write_event(Event::End(BytesEnd::owned(name.into_bytes())))
                .expect("writing start of element");
        } else if let Some(text) = self.dyn_ref::<sys::Text>() {
            writer
                .write_event(quick_xml::events::Event::Text(BytesText::from_plain_str(
                    &text.data(),
                )))
                .expect("writing text node");
        } else {
            unreachable!("augdom only creates elements and text nodes. this is a bug.");
        }
    }

    fn create_element(&self, ty: &str) -> Self {
        document().create_element(ty).unwrap().into()
    }

    fn create_text_node(&self, contents: &str) -> Self {
        document().create_text_node(contents).into()
    }

    fn first_child(&self) -> Option<Self> {
        self.first_child()
    }

    fn append_child(&self, child: &Self) {
        self.append_child(child).unwrap();
    }

    fn next_sibling(&self) -> Option<Self> {
        self.next_sibling()
    }

    fn remove_child(&self, to_remove: &Self) -> Option<Self> {
        self.remove_child(to_remove).ok()
    }

    fn replace_child(&self, new_child: &Self, existing: &Self) {
        self.replace_child(new_child, existing).unwrap();
    }

    fn get_attribute(&self, name: &str) -> Option<String> {
        let e: &sys::Element = self.dyn_ref().unwrap();
        e.get_attribute(name)
    }

    fn set_attribute(&self, name: &str, value: &str) {
        let e: &sys::Element = self.dyn_ref().unwrap();
        e.set_attribute(name, value).unwrap();
    }

    fn remove_attribute(&self, name: &str) {
        let e: &sys::Element = self.dyn_ref().unwrap();
        e.remove_attribute(name).ok();
    }

    fn query_selector(&self, selectors: &str) -> Option<Self> {
        let e: &sys::Element = self.dyn_ref().unwrap();
        sys::Element::query_selector(e, selectors).unwrap().map(Into::into)
    }

    fn query_selector_all(&self, selectors: &str) -> Self::Nodes {
        let e: &sys::Element = self.dyn_ref().unwrap();
        NodeList { idx: 0, inner: sys::Element::query_selector_all(e, selectors).unwrap() }
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

impl From<sys::HtmlElement> for Node {
    fn from(e: sys::HtmlElement) -> Self {
        Node::Concrete(e.into())
    }
}

impl From<sys::Text> for Node {
    fn from(e: sys::Text) -> Self {
        Node::Concrete(e.into())
    }
}

impl Node {
    /// Returns a reference to a concrete DOM node, panics if this is a virtual
    /// node.
    pub fn expect_concrete(&self) -> &sys::Node {
        match self {
            Node::Concrete(n) => n,

            #[cfg(feature = "rsdom")]
            Node::Virtual(_) => panic!("expected a Node::Concrete, found a Node::Virtual"),
        }
    }
}

/// Wraps [`sys::NodeList`] to implement `Iterator`.
pub struct NodeList {
    inner: sys::NodeList,
    idx: u32,
}

impl Iterator for NodeList {
    type Item = sys::Node;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.inner.item(self.idx);
        self.idx += 1;
        ret
    }
}

impl std::iter::ExactSizeIterator for NodeList {
    fn len(&self) -> usize {
        self.inner.length() as _
    }
}
