#![recursion_limit = "1024"]

use filter::Visibility;
use header::input_header;
use main_section::main_section;

use illicit::AsContext;
use mox::mox;
use moxie_dom::{
    elements::sectioning::{section, Section},
    interfaces::element::Element,
    prelude::*,
};
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::*;
use wasm_bindgen::prelude::*;

pub mod filter;
pub mod footer;
pub mod header;
pub mod input;
pub mod item;
pub mod main_section;

#[topo::nested]
fn todo_app() -> Section {
    mox! {
        <section class="todoapp">
            { input_header() }
            { main_section() }
        </section>
    }
}

pub(crate) struct App {
    pub todos: Key<Vec<Todo>>,
    pub visibility: Key<Visibility>,
}

impl App {
    #[topo::nested]
    pub fn current() -> Self {
        let (_, visibility) = state(Visibility::default);
        let (_, todos) =
            // we allow the default empty to be overridden for testing
            // TODO support localStorage
            state(|| illicit::get::<Vec<Todo>>().map(|d| d.clone()).unwrap_or_default());

        Self { todos, visibility }
    }

    pub fn enter<T>(self, f: impl FnMut() -> T) -> T {
        illicit::Layer::new().offer(self.todos).offer(self.visibility).enter(f)
    }

    pub fn boot(node: impl Into<moxie_dom::raw::Node>) {
        Self::boot_fn(&[], node, todo_app)
    }

    fn boot_fn<Root: Element + 'static>(
        default_todos: &[Todo],
        node: impl Into<moxie_dom::raw::Node>,
        mut root: impl FnMut() -> Root + 'static,
    ) {
        let defaults = default_todos.to_vec();
        moxie_dom::boot(node, move || defaults.clone().offer(|| App::current().enter(&mut root)));
        info!("running");
    }
}

#[derive(Clone, Debug)]
pub struct Todo {
    id: u32,
    title: String,
    completed: bool,
}

impl Todo {
    fn new(s: impl Into<String>) -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        Self { id: NEXT_ID.fetch_add(1, Ordering::SeqCst), title: s.into(), completed: false }
    }
}

#[wasm_bindgen(start)]
pub fn setup_tracing() {
    static SETUP: std::sync::Once = std::sync::Once::new();
    SETUP.call_once(|| {
        tracing_wasm::set_as_global_default_with_config(tracing_wasm::WASMLayerConfig {
            report_logs_in_console: true,
            report_logs_in_timings: false,
            use_console_color: false,
        });
        std::panic::set_hook(Box::new(|info| {
            error!(?info, "crashed");
        }));
        info!("tracing initialized");
    });
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn boot(root: moxie_dom::raw::sys::Node) {
    App::boot(root);
}

/// Included as a module within the crate rather than a separate file because
/// cargo is grumpy about resolving the crate-under-test.
#[cfg(test)]
mod integration_tests;

#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
