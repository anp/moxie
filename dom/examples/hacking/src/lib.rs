use moxie_dom::{
    elements::{forms::button, text_content::div},
    prelude::*,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn begin() {
    console_log::init_with_level(tracing::log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{:#?}", info);
    }));

    tracing::info!("mounting moxie-dom to root");
    moxie_dom::boot(document().body().unwrap(), || {
        let count = state(|| 0);

        mox! {<>
            <div>{% "hello world from moxie! ({})", &count }</div>

            <button type="button" onclick={move |_| count.update(|c| Some(c + 1))}>
                "increment"
            </button>
        </>};

        for t in &["first", "second", "third"] {
            mox! { <div>{% "{}", t }</div> };
        }
    });
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    pub fn hello_browser() {
        println!("hello");
    }
}
