use {
    moxie::*,
    std::hash::{Hash, Hasher},
    wasm_bindgen::prelude::*,
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

pub fn greet() {
    alert("Hello, moxie-dom!");
}

#[derive(Clone, Debug)]
pub struct Node(web_sys::Node);

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        unimplemented!()
    }
}
impl Eq for Node {}

impl From<web_sys::Node> for Node {
    fn from(inner: web_sys::Node) -> Self {
        Node(inner)
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        unimplemented!()
    }
}

// TODO(anp): better name for this struct
#[props]
pub struct DomBinding<Root: Component> {
    node: Node,
    root: Root,
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

#[props]
pub struct Span {}

impl Component for Span {
    fn compose(scp: Scope, props: Self) {}
}
