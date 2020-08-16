use filter::Visibility;
use header::input_header;
use main_section::main_section;

use illicit::AsContext;
use mox::mox;
use moxie_dom::{
    elements::sectioning::{section, Section},
    interfaces::node::Node,
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

    pub fn boot<Root: Node + 'static>(
        default_todos: &[Todo],
        node: impl Into<moxie_dom::raw::Node>,
        mut root: impl FnMut() -> Root + 'static,
    ) {
        let defaults = default_todos.to_vec();
        moxie_dom::boot(node, move || defaults.clone().offer(|| App::current().enter(&mut root)));
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
    tracing_wasm::set_as_global_default();
    std::panic::set_hook(Box::new(|info| {
        error!(?info, "crashed");
    }));
    info!("tracing initialized");
}

#[wasm_bindgen]
pub fn boot(root: moxie_dom::raw::sys::Node) {
    App::boot(&[], root, todo_app);
    info!("running");
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
