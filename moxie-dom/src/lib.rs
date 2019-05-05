pub mod prelude {
    pub use {
        crate::{DomBinding, Span, WebRuntime},
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

#[props]
pub struct DomBinding<Root: Component> {
    pub node: web::Node,
    pub root: Root,
}

impl<Root> Component for DomBinding<Root>
where
    Root: Component,
{
    fn compose(scp: Scope, Self { node, root }: Self) {
        scp.compose_child_with_witness(scope!(scp.id()), root, Weaver::attached_to(scp.id(), node));
    }
}

#[props]
pub struct Span {
    pub text: Option<String>,
}

impl Component for Span {
    fn compose(scp: Scope, props: Self) {
        use stdweb::web::{INode, Node};
        let node = state!(scp <- web::document().create_element("p").unwrap());

        if let Some(text) = &props.text {
            node.set_text_content(text);
        }

        let raw: Node = node.clone().into();
        scp.record(raw);
    }
}
