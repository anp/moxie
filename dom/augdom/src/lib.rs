//! `augdom` provides an "augmented DOM" implementation that can run almost anywhere Rust can. By
//! default the `webdom` feature is enabled and this crate is a wrapper around `web-sys` for
//! creating and manipulating HTML elements. See the [crate::Dom] trait for the provided behavior.
//!
//! The `rsdom` feature enables a DOM emulation layer written in pure Rust which can be
//! used for testing or to render HTML strings.
//!
//! # Known Limitations
//!
//! As of today the `<web_sys::Element as Dom>::*_attribute` methods will panic if called on a text
//! node. This cost seems appropriate today because this is a dependency for other crates which
//! enforce this requirement themselves. `web_sys` enforces this restriction statically.
#![deny(missing_docs)]

static_assertions::assert_cfg!(
    any(feature = "webdom", feature = "rsdom"),
    "At least one DOM implementation's feature must be enabled (`webdom`, `rsdom`)"
);

#[cfg(feature = "webdom")]
pub use {wasm_bindgen::JsCast, web_sys as sys};

#[cfg(feature = "rsdom")]
use {rsdom::VirtNode, std::rc::Rc};

use {
    quick_xml::Writer as XmlWriter,
    std::{
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        io::{prelude::*, Cursor},
    },
};

#[cfg(feature = "rsdom")]
pub mod rsdom;
#[cfg(feature = "webdom")]
pub mod webdom;

pub mod event;

/// Returns the current window. Panics if no window is available.
#[cfg(feature = "webdom")]
pub fn window() -> sys::Window {
    sys::window().expect("must run from within a `window`")
}

/// Returns the current document. Panics if called outside a web document context.
#[cfg(feature = "webdom")]
pub fn document() -> sys::Document {
    window()
        .document()
        .expect("must run from within a `window` with a valid `document`")
}

/// A value which implements a subset of the web's document object model.
pub trait Dom: Sized {
    // TODO is there a way to pass the starting indentation down from a formatter?
    /// Write this value as XML via the provided writer. Consider using [Xml::inner_html] or
    /// [Xml::pretty_inner_html] unless you need the performance.
    fn write_xml<W: Write>(&self, writer: &mut XmlWriter<W>);

    /// Returns a string of serialized XML without newlines or indentation.
    fn inner_html(&self) -> String {
        let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        {
            let mut writer = XmlWriter::new(&mut buf);
            self.write_xml(&mut writer);
        }
        String::from_utf8(buf.into_inner()).unwrap()
    }

    /// Returns a string of "prettified" serialized XML with the provided indentation.
    fn pretty_inner_html(&self, indent: usize) -> String {
        let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        {
            let mut writer = XmlWriter::new_with_indent(&mut buf, ' ' as u8, indent);
            self.write_xml(&mut writer);
        }
        String::from_utf8(buf.into_inner()).unwrap()
    }

    /// Create a new element within the same tree as the method receiver.
    fn create_element(&self, ty: &str) -> Self;

    /// Create a new text node within the same tree as the method receiver.
    fn create_text_node(&self, contents: &str) -> Self;

    /// Set an attribute on this DOM node.
    fn set_attribute(&self, name: &str, value: &str);

    /// Ensure the provided attribute has been removed from this DOM node.
    fn remove_attribute(&self, name: &str);

    /// Returns the next child of this node's parent after this node itself.
    fn next_sibling(&self) -> Option<Self>;

    /// Returns the first child of this node.
    fn first_child(&self) -> Option<Self>;

    /// Adds a new child to the end of this node's children.
    fn append_child(&self, child: &Self);

    /// Replaces the provided child of this node with a new one.
    fn replace_child(&self, new_child: &Self, existing: &Self);

    /// Removes the provided child from this node.
    fn remove_child(&self, to_remove: &Self) -> Option<Self>;
}

/// A `Node` in the augmented DOM.
#[derive(Clone)]
pub enum Node {
    /// A handle to a concrete DOM node running in the browser.
    #[cfg(feature = "webdom")]
    Concrete(sys::Node),

    /// A handle to a "virtual" DOM node, emulating the web in memory. While this implementation
    /// lacks many features, it can run on any target that Rust supports.
    #[cfg(feature = "rsdom")]
    Virtual(Rc<VirtNode>),
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let s = if f.alternate() {
            self.pretty_inner_html(4)
        } else {
            self.inner_html()
        };
        f.write_str(&s)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&self.pretty_inner_html(2))
    }
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

impl Dom for Node {
    fn write_xml<W: Write>(&self, writer: &mut XmlWriter<W>) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => {
                n.write_xml(writer);
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => {
                n.write_xml(writer);
            }
        }
    }

    fn create_element(&self, ty: &str) -> Self {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => Node::Concrete(n.create_element(ty).into()),

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => Node::Virtual(n.create_element(ty).into()),
        }
    }

    fn create_text_node(&self, contents: &str) -> Self {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => Node::Concrete(n.create_text_node(contents)),

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => Node::Virtual(n.create_text_node(contents)),
        }
    }

    fn first_child(&self) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::first_child(n).map(Node::Concrete),

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.first_child().map(Node::Virtual),
        }
    }

    fn append_child(&self, child: &Self) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => {
                <sys::Node as Dom>::append_child(n, child.expect_concrete());
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => {
                n.append_child(child.expect_virtual());
            }
        }
    }

    fn next_sibling(&self) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::next_sibling(n).map(Node::Concrete),

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.next_sibling().map(Node::Virtual),
        }
    }

    fn remove_child(&self, to_remove: &Self) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => {
                <sys::Node as Dom>::remove_child(n, to_remove.expect_concrete()).map(Node::Concrete)
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n
                .remove_child(to_remove.expect_virtual())
                .map(Node::Virtual),
        }
    }

    fn replace_child(&self, new_child: &Node, existing: &Node) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => {
                <sys::Node as Dom>::replace_child(
                    n,
                    new_child.expect_concrete(),
                    existing.expect_concrete(),
                );
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => {
                n.replace_child(new_child.expect_virtual(), existing.expect_virtual());
            }
        }
    }

    fn set_attribute(&self, name: &str, value: &str) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::set_attribute(n, name, value),
            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.set_attribute(name, value),
        }
    }

    fn remove_attribute(&self, name: &str) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::remove_attribute(n, name),
            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.remove_attribute(name),
        }
    }
}
