use {
    log::*,
    moxie_dom::prelude::*,
    stdweb::{web::INode, *},
    wasm_bindgen::prelude::*,
};

#[props]
struct HackedApp;

impl Component for HackedApp {
    fn compose(scp: Scope, HackedApp: Self) {
        // todo
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_log::init().unwrap();
    stdweb::initialize();

    let document = web::document();
    let body = document.body().unwrap();

    Runtime::go(
        moxie_dom::WebSpawner,
        DomBinding {
            root: HackedApp,
            node: body.as_node().to_owned(),
        },
    );

    Ok(())
}
