#![feature(track_caller)]

use augdom::{event::Click, testing::Query};
use moxie_dom::{
    elements::html::{button, div},
    prelude::*,
};
use wasm_bindgen_test::*;

struct Counter {
    button: moxie_dom::elements::forms::Button,
}

impl moxie_dom::interfaces::content_categories::FlowContent for Counter {}

impl moxie_dom::interfaces::node::Child for Counter {
    fn to_bind(&self) -> &augdom::Node {
        self.button.to_bind()
    }
}

fn counter() -> CounterBuilder {
    CounterBuilder::default()
}

#[derive(Default)]
struct CounterBuilder {
    default_value: Option<i32>,
    text: Option<String>,
}

impl CounterBuilder {
    pub fn value(mut self, value: i32) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn button_text(mut self, text: impl ToString) -> Self {
        self.text = Some(text.to_string());
        self
    }

    #[topo::nested]
    pub fn build(self) -> Counter {
        let Self { text, default_value } = self;
        let (value, set_value) = state(|| default_value.unwrap_or(0));
        let text = text.unwrap_or_default();

        let button = mox! {
            <button onclick={move |_| set_value.update(|n| Some(n + 1))}>
                {% "{} ({})", text, value }
            </button>
        };

        Counter { button }
    }
}

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub async fn binds_to_div() {
    let render_counter_as_child = || mox!(<div><counter button_text="child" value=9/></div>);
    let test_root = augdom::Node::new("div");
    moxie_dom::boot(test_root.clone(), render_counter_as_child);

    test_root.find().by_text("child (9)").until().many().await;
    assert_eq!(
        test_root.first_child().unwrap().to_string(),
        "<div>
  <button>child (9)</button>
</div>",
    );
}

#[wasm_bindgen_test]
pub async fn renders_and_interacts() {
    let render_counter = || mox!(<counter button_text="foo" value=0/>);
    let test_root = augdom::Node::new("div");
    moxie_dom::boot(test_root.clone(), render_counter);

    let button = test_root.find().by_text("foo (0)").until().one().await.unwrap();
    assert_eq!(test_root.first_child().unwrap().to_string(), "<button>foo (0)</button>",);

    button.dispatch::<Click>();
    test_root.find().by_text("foo (1)").until().one().await.unwrap();
    assert_eq!(test_root.first_child().unwrap().to_string(), "<button>foo (1)</button>",);

    button.dispatch::<Click>();
    test_root.find().by_text("foo (2)").until().one().await.unwrap();
    assert_eq!(test_root.first_child().unwrap().to_string(), "<button>foo (2)</button>",);
}
