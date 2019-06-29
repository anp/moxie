#![warn(doc_missing)]

#[doc(hidden)] // TODO fix the topo function export stuff
//pub use moxie::*;
use {
    futures::{
        future::{FutureExt, LocalFutureObj, TryFutureExt},
        task::SpawnError,
    },
    moxie::{self, *},
    stdweb::{traits::*, *},
};

#[topo::bound]
pub fn mount(root: impl Component, on: web::Node) {
    unimplemented!()
}

#[derive(Clone, PartialEq)]
pub struct DomBinding<Root: Component> {
    pub node: web::Node,
    pub root: Root,
}

impl<Root> Component for DomBinding<Root>
where
    Root: Component,
{
    fn contents(&self) {
        let Self { node, root } = self;
        unimplemented!()
    }
}

#[derive(Debug, PartialEq)]
pub struct Text(pub String);

impl Component for Text {
    fn contents(&self) {
        let node = web::document().create_text_node(&self.0);
        let raw: web::Node = node.into();
        unimplemented!()
    }
}

#[derive(Debug, PartialEq)]
pub struct Span<Children> {
    pub children: Children,
}

impl<Children> Component for Span<Children>
where
    Children: Component,
{
    fn contents(&self) {
        let node = memo!((), |()| web::document().create_element("span").unwrap());
        let raw: web::Node = node.clone().into();
        unimplemented!()

        // TODO compose children
    }
}
