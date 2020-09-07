use moxie_dom::raw::{document, Node};
use wasm_bindgen_test::wasm_bindgen_test;

#[must_use = "needs to live as long as the test"]
fn boot_test() -> impl Drop {
    let root = Node::new_concrete("div").expect_concrete().clone();
    document().body().unwrap().append_child(&root).unwrap();
    let ret = scopeguard::guard(root.clone(), |root| {
        document().body().unwrap().remove_child(&root).unwrap();
    });
    super::boot(root);
    ret
}

#[wasm_bindgen_test]
pub async fn hello_browser() {
    let _booted = boot_test();
}
