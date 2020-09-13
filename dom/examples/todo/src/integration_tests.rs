use moxie_dom::{prelude::*, raw::Node};
use tracing::*;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
pub async fn add_2_todos() {
    let test = Test::new();
}

struct Test {
    root: Node,
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
}

impl Drop for Test {
    fn drop(&mut self) {
        document().body().remove_child(&self.root);
        // TODO blur active element just to be safe
        // TODO stop app and block until cleaned up
        // TODO clear local storage
    }
}
