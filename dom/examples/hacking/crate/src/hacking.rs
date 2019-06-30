use {
    moxie::*,
    moxie_dom::*,
    stdweb::{traits::*, *},
    wasm_bindgen::prelude::*,
};

#[derive(Clone, PartialEq)]
struct HackedApp;

impl Component for HackedApp {
    fn contents(&self) {
        show!(Span {
            children: Text("hello world from moxie!".into()),
        });
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_log::init().unwrap();

    let body = web::document().body().unwrap().as_node().to_owned();
    let root = web::document().create_element("div").unwrap();
    body.append_child(&root);

    moxie_dom::mount!(root.as_node().to_owned(), || show!(HackedApp));

    Ok(())
}
