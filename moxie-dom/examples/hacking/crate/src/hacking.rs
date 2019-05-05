use {
    log::*,
    moxie_dom::prelude::*,
    stdweb::{traits::*, *},
    wasm_bindgen::prelude::*,
};

#[props]
struct HackedApp;

impl Component for HackedApp {
    fn compose(scp: Scope, HackedApp: Self) {
        info!("logging from moxie-dom's first component");
        mox! { scp <- Span {
            text: Some("hello world from moxie!".into()),
        }};
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_log::init().unwrap();

    let document = web::document();
    let body = document.body().unwrap();

    let val = document.create_element("p").unwrap();
    val.set_text_content("hello world from stdweb, not moxie");

    body.append_child(&val);

    info!("spawning moxie runtime");

    WebRuntime::default().spawn_self(DomBinding {
        root: HackedApp,
        node: body.as_node().to_owned(),
    });

    Ok(())
}
