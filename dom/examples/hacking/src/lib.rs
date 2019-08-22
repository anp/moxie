use moxie_dom::prelude::*;

#[derive(Clone, Debug)]
struct HackedApp;

impl Component for HackedApp {
    fn contents(self) {
        let count = state!(|| 0);
        show![
            text!("hello world from moxie! ({})", &count),
            element("button")
                .attr("type", "button")
                .on(|_: ClickEvent, count| Some(count + 1), count)
                .child(text!("increment")),
            vec![text!("first"), text!(" second"), text!(" third"),]
        ];
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(tracing::log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{:#?}", info);
    }));

    tracing::info!("mounting moxie-dom to root");
    mount!(document().body().unwrap(), HackedApp);
}
