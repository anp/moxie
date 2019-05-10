use {
    moxie_dom::prelude::*,
    stdweb::{traits::*, *},
    wasm_bindgen::prelude::*,
};

#[derive(Clone)]
struct HackedApp;

impl Component for HackedApp {
    fn run(self, scp: Scope) {
        run! { scp <- Span {
            children: vec![&Text("hello world from moxie!") as &dyn Component],
        }};
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_log::init().unwrap();

    let body = web::document().body().unwrap().as_node().to_owned();
    let root = web::document().create_element("div").unwrap();
    body.append_child(&root);

    WebRuntime::default().spawn_self(DomBinding {
        root: HackedApp,
        node: root.as_node().to_owned(),
    });

    Ok(())
}
