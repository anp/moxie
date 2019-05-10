pub mod prelude {
    pub use {
        crate::{DomBinding, Span, Text, WebRuntime},
        moxie::{self, *},
    };
}

use {
    crate::{prelude::*, weaver::Weaver},
    futures::{
        future::{FutureExt, LocalFutureObj, TryFutureExt},
        task::SpawnError,
    },
    stdweb::*,
};

mod weaver;

#[derive(Default)]
pub struct WebRuntime {}

impl Runtime for WebRuntime {
    type Spawner = WebSpawner;
    fn spawner(&self) -> Self::Spawner {
        WebSpawner
    }
}

#[derive(Default)]
pub struct WebSpawner;

impl ComponentSpawn for WebSpawner {
    fn spawn_local(&mut self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        wasm_bindgen_futures::spawn_local(future.unit_error().compat());
        Ok(())
    }

    fn child(&self) -> Box<dyn ComponentSpawn> {
        Box::new(WebSpawner)
    }
}

#[derive(Clone)]
pub struct DomBinding<Root: Component> {
    pub node: web::Node,
    pub root: Root,
}

impl<Root> Component for DomBinding<Root>
where
    Root: Component,
{
    fn run(self, scp: Scope) {
        let Self { node, root } = self;
        scp.install_witness(Weaver);
        scp.record(node);
        scp.compose_child(scope!(scp.id()), root);
    }
}

#[derive(Debug)]
pub struct Text<'txt>(pub &'txt str);

impl<'txt> Component for Text<'txt> {
    fn run(self, scp: Scope) {
        let node = web::document().create_text_node(self.0);
        let raw: web::Node = node.into();
        scp.record(raw);
    }
}

pub struct Span<'parent, Children>
where
    Children: IntoIterator<Item = &'parent dyn Component>,
{
    pub children: Children,
}

impl<'p, Children> Component for Span<'p, Children>
where
    Children: IntoIterator<Item = &'p dyn Component>,
{
    fn run(self, scp: Scope) {
        let node = state!(scp <- web::document().create_element("span").unwrap());
        let raw: web::Node = node.clone().into();
        scp.record(raw);

        // TODO compose children
    }
}
