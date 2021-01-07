use mox::mox;
use moxie_dom::{
    elements::{
        forms::button,
        text_content::{div, Div},
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
    moxie_dom::boot(document().body(), root);
}

#[topo::nested]
fn root() -> Div {
    let (count, set_count) = state(|| 0);

    let mut root = div();

    root = root.child(mox! { <div>{% "hello world from moxie! ({})", &count }</div> });
    root = root.child(mox! {
        <button type="button" onclick = move |_| set_count.update(|c| Some(c + 1))>
            "increment"
        </button>
    });

    for t in &["first", "second", "third"] {
        root = root.child(mox! { <div>{ t }</div> });
    }

    root.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use augdom::testing::{Query, TargetExt};
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    pub async fn hello_browser() {
        let test_root = document().create_element("div");
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

        button.click();
        test_root.find().by_text("hello world from moxie! (1)").until().one().await.unwrap();
        button.click();
        test_root.find().by_text("hello world from moxie! (2)").until().one().await.unwrap();
    }
}
