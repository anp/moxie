pub mod prelude {
    pub use {
        crate::{DomBinding, WebSpawner},
        moxie::{self, *},
    };
}

use {
    crate::prelude::*,
    futures::{
        future::{FutureExt, LocalFutureObj, TryFutureExt},
        task::SpawnError,
    },
    stdweb::*,
};

pub struct WebSpawner;

impl PrioritySpawn for WebSpawner {
    fn spawn_local(&mut self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        wasm_bindgen_futures::spawn_local(future.unit_error().compat());
        Ok(())
    }

    fn child(&self) -> Box<dyn PrioritySpawn> {
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
        let child_id = scope!(scp.id());
        let child_scope = scp.child(child_id);
        // child_scope.install_witness(Weaver::attached_to(node));

        scp.compose_child(child_id, root);

        // let weaver: Weaver = child_scope.remove_witness().unwrap();

        // TODO make all the nodes go together?
    }
}

// #[props]
// pub struct Span {}

// impl Component for Span {
//     fn compose(scp: Scope, props: Self) {}
// }
