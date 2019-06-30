#![warn(missing_docs)]

use {
    moxie::{self, *},
    stdweb::*,
};

#[topo::bound]
pub fn mount(new_parent: web::Node, contents: impl FnOnce()) {
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
        let _raw: web::Node = node.into();
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
        let _raw: web::Node = node.clone().into();
        unimplemented!()

        // TODO compose children
    }
}
