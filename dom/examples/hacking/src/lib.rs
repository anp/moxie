use moxie_dom::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(tracing::log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{:#?}", info);
    }));

    tracing::info!("mounting moxie-dom to root");
    moxie_dom::boot(document().body().unwrap(), || {
        let count = state!(|| 0);

        moxml! {<>
            <div>{% "hello world from moxie! ({})", &count }</div>

            // TODO figure out how this could be `onclick` and still be nice
            <button type="button" on={move |_: ClickEvent| count.update(|c| Some(c + 1))}>
                "increment"
            </button>
        </>};

        for t in &["first", "second", "third"] {
            moxml! { <div>{% "{}", t }</div> };
        }
    });
}
