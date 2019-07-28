use {
    moxie_dom::{elements::*, events::*, *},
    tracing::*,
    wasm_bindgen::prelude::*,
};

#[derive(Clone, Debug, PartialEq)]
struct HackedApp;

impl Component for HackedApp {
    fn contents(self) {
        let (count, count_key) = state!(|| 0);
        show![
            text!("hello world from moxie! ({})", count),
            Button::new()
                .on(count_key, |count, event: ClickEvent| Some(count + 1))
                .child(text!("increment")),
            vec![text!("first"), text!(" second"), text!(" third"),]
        ];
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init();
    std::panic::set_hook(Box::new(|info| {
        error!("{:#?}", info);
    }));

    info!("mounting moxie-dom to root");
    mount!(document().body().unwrap(), HackedApp);
}
