use mox::mox;
use moxie_dom::{elements::html::div, interfaces::node::{Child, NodeBuilder}, prelude::*};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

pub async fn render_test<Root>(render_child: impl FnMut() -> Root + 'static, expected: &str) where Root: Child + 'static{
    let test_root = document().create_element("div");
    moxie_dom::boot(test_root.clone(), render_child);

    assert_eq!(
        test_root.first_child().unwrap().to_string(),
        expected,
    );
}

#[wasm_bindgen_test]
pub async fn simple_mox() {
    render_test(|| mox!(<div>"test text"</div>),"<div>test text</div>").await;
}

#[wasm_bindgen_test]
pub async fn block_mox() {
    render_test(|| mox!(<div>{"test text"}</div>),"<div>test text</div>").await;
}

#[wasm_bindgen_test]
pub async fn simple_builder() {
    render_test(|| div().child(text("test text")),"<div>test text</div>").await;
}

#[wasm_bindgen_test]
pub async fn built_builder() {
    render_test(|| div().child(text("test text").build()).build(),"<div>test text</div>").await;
}
