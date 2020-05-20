#![feature(track_caller)]

use moxie_dom::{
    elements::{
        forms::button,
        text_content::{div, DivBuilder},
    },
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
    moxie_dom::boot(document().body().unwrap(), root);
}

#[topo::nested]
fn root() -> DivBuilder {
    let count = state(|| 0);

    let mut root = div();

    root = root.child(mox! { <div>{% "hello world from moxie! ({})", &count }</div> });
    root = root.child(mox! {
        <button type="button" onclick={move |_| count.update(|c| Some(c + 1))}>
            "increment"
        </button>
    });

    for t in &["first", "second", "third"] {
        root = root.child(mox! { <div>{% "{}", t }</div> });
    }

    root.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use augdom::{event::Click, testing::Query};
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    pub async fn hello_browser() {
        let test_root = augdom::Node::new("div");
        moxie_dom::boot(test_root.clone(), root);

        let button = test_root.find().by_text("increment").until().one().await.unwrap();
        assert_eq!(
            test_root.first_child().unwrap().to_string(),
            r#"<div>
  <div>hello world from moxie! (0)</div>
  <button type="button">increment</button>
  <div>first</div>
  <div>second</div>
  <div>third</div>
</div>"#
        );

        button.dispatch::<Click>();
        test_root.find().by_text("hello world from moxie! (1)").until().one().await.unwrap();
        button.dispatch::<Click>();
        test_root.find().by_text("hello world from moxie! (2)").until().one().await.unwrap();
    }
}
