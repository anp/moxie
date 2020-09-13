use moxie_dom::prelude::*;
use wasm_bindgen_test::wasm_bindgen_test;

#[must_use = "needs to live as long as the test"]
fn boot_test() -> impl Drop {
    let document = document();
    let body = document.body();

    let root = document.create_element("div");
    body.append_child(&root);
    let ret = scopeguard::guard(root.clone(), move |root| {
        body.remove_child(&root).unwrap();
    });
    super::boot(root.expect_concrete().clone());
    ret
}

#[wasm_bindgen_test]
pub async fn hello_browser() {
    let _booted = boot_test();
}
