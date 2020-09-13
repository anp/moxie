use moxie_dom::{
    prelude::*,
    raw::{
        testing::{Query, TargetExt},
        Node,
    },
};
use std::ops::Deref;
use tracing::*;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
pub async fn add_2_todos() {
    let test = Test::new();
    test.add_todo("learn testing").await;
    test.add_todo("be cool").await;
    // TODO assert .todo-list has two <li> in the right order
}

struct Test {
    root: Node,
}

impl Deref for Test {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl Test {
    fn new() -> Self {
        tracing_wasm::set_as_global_default_with_config(tracing_wasm::WASMLayerConfig {
            report_logs_in_console: true,
            report_logs_in_timings: false,
            use_console_color: false,
        });
        std::panic::set_hook(Box::new(|info| {
            error!(?info, "crashed");
        }));
        info!("tracing initialized");

        let root = document().create_element("div");
        document().body().append_child(&root);
        super::boot(root.expect_concrete().clone());
        Test { root }
    }

    async fn add_todo(&self, todo: &str) {
        self.input().keyboardln(todo);
        // wait for it to show up
        // TODO make sure it shows up at the end
        // TODO assert there's only one matching <li>
        self.find().by_text(todo).until().many().await.unwrap();
    }

    fn input(&self) -> Node {
        self.find().by_placeholder_text(INPUT_PLACEHOLDER).one().unwrap()
    }
}

impl Drop for Test {
    fn drop(&mut self) {
        document().body().remove_child(&self.root);
        // TODO blur active element just to be safe
        // TODO stop app and block until cleaned up
        // TODO clear local storage
    }
}

const INPUT_PLACEHOLDER: &str = "What needs to be done?";
