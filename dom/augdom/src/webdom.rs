//! An implementation of `augdom`'s APIs on top of the actual web using the
//! `web-sys` crate and `wasm-bindgen`.

use super::Node;
use crate::document;
use futures::{channel::mpsc::UnboundedReceiver, Stream};
use prettiest::Pretty;
use std::{
    any::type_name,
    io::Write,
    pin::Pin,
    task::{Context, Poll},
};
use tracing::error;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys as sys;

/// A dynamically-allocated closure that is callable from JavaScript.
pub struct Callback {
    cb: Closure<dyn FnMut(JsValue)>,
}

impl Callback {
    /// Allocate a new JS-compatible callback.
    pub fn new<T>(mut cb: impl FnMut(T) + 'static) -> Self
    where
        T: JsCast,
    {
        let cb = Closure::wrap(Box::new(move |raw: JsValue| match raw.dyn_into() {
            Ok(value) => cb(value),
            Err(v) => {
                error!(
                    failed_cast_to = %type_name::<T>(),
                    value = %v.pretty(),
                    "received unexpected value in callback",
                );
            }
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
    type MutationRecord = sys::MutationRecord;
    type Nodes = NodeList;
    type Observer = Mutations;

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
        let e: Option<&sys::Element> = self.dyn_ref();
        e.map(|e| sys::Element::get_attribute(e, name)).flatten()
    }

    fn set_attribute(&self, name: &str, value: &str) {
        let e: &sys::Element = self.dyn_ref().unwrap();
        e.set_attribute(name, value).unwrap();
    }

    fn remove_attribute(&self, name: &str) {
        let e: &sys::Element = self.dyn_ref().unwrap();
        e.remove_attribute(name).ok();
    }

    fn get_inner_text(&self) -> String {
        let e: Option<&sys::HtmlElement> = self.dyn_ref();
        e.map(sys::HtmlElement::inner_text).unwrap_or_default()
    }

    fn dispatch<E: crate::event::Event>(&self, event: E) {
        event.dispatch(self);
    }

    fn query_selector(&self, selectors: &str) -> Option<Self> {
        let e: &sys::Element = self.dyn_ref().unwrap();
        sys::Element::query_selector(e, selectors).unwrap().map(Into::into)
    }

    fn query_selector_all(&self, selectors: &str) -> Self::Nodes {
        let e: &sys::Element = self.dyn_ref().unwrap();
        NodeList { idx: 0, inner: sys::Element::query_selector_all(e, selectors).unwrap() }
    }

    fn observe_mutations(&self) -> Self::Observer {
        Mutations::new(self)
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

/// Wraps a [`web_sys::MutationObserver`], providing a `Stream`.
pub struct Mutations {
    observer: crate::sys::MutationObserver,
    _callback: Callback,
    records: UnboundedReceiver<Vec<sys::MutationRecord>>,
}

impl Mutations {
    fn new(node: &crate::sys::Node) -> Self {
        let (sender, records) = futures::channel::mpsc::unbounded();
        let _callback = Callback::new(move |arr: js_sys::Array| {
            let records = arr
                .iter()
                .map(|val| val.dyn_into::<crate::sys::MutationRecord>().unwrap())
                .collect();
            sender.unbounded_send(records).unwrap();
        });
        let observer = crate::sys::MutationObserver::new(_callback.as_fn()).unwrap();
        let mut options = crate::sys::MutationObserverInit::new();
        options.attributes(true);
        options.character_data(true);
        options.child_list(true);
        options.subtree(true);
        observer.observe_with_options(node, &options).unwrap();

        Self { observer, _callback, records }
    }
}

impl Stream for Mutations {
    type Item = Vec<sys::MutationRecord>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let records = &mut self.get_mut().records;
        futures::pin_mut!(records);
        records.poll_next(cx)
    }
}

impl Drop for Mutations {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}
