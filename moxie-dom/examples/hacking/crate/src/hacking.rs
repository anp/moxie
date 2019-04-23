// use moxie_dom::prelude::*;
use stdweb::{web::INode, *};
use wasm_bindgen::prelude::*;

// #[props]
// struct HackedApp;

// impl Component for HackedApp {
//     fn compose(scp: Scope, HackedApp: Self) {
//         // todo
//     }
// }

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    stdweb::initialize();

    js! {
        console.log("exec'ing main");
    }
    // println!("execing main, printed with std");
    // let document = web::document();
    // let body = document.body().unwrap();

    // Runtime::go(
    //     moxie_dom::WebSpawner,
    //     DomBinding {
    //         root: HackedApp,
    //         node: body.as_node().to_owned(),
    //     },
    // );

    stdweb::event_loop();

    Ok(())
}
