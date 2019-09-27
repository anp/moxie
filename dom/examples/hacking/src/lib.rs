use moxie_dom::{prelude::*, *};

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(tracing::log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{:#?}", info);
    }));

    tracing::info!("mounting moxie-dom to root");
    moxie_dom::boot(document().body().unwrap(), || {
        let count = state!(|| 0);
        text!(&format!("hello world from moxie! ({})", &count));

        element!("button", |e| e
            .attr("type", "button")
            .on(move |_: ClickEvent| count.update(|c| Some(c + 1)))
            .inner(|| text!("increment")));

        vec![text!("first"), text!(" second"), text!(" third")];
    });
}
