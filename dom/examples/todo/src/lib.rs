use filter::Visibility;
use header::input_header;
use main_section::main_section;
use mox::mox;
use moxie_dom::{
    elements::text_content::{div, Div},
    interfaces::node::Node,
    prelude::*,
};
use std::sync::atomic::{AtomicU32, Ordering};
use wasm_bindgen::prelude::*;

pub mod filter;
pub mod footer;
pub mod header;
pub mod input;
pub mod item;
pub mod main_section;

#[topo::nested]
fn todo_app() -> Div {
    mox! {
        <div class="todoapp">
            { input_header() }
            { main_section() }
        </div>
    }
}

pub(crate) struct App {
    pub todos: Key<Vec<Todo>>,
    pub visibility: Key<Visibility>,
}

impl App {
    #[topo::nested]
    pub fn current(default_todos: &[Todo]) -> Self {
        let (_, visibility) = state(Visibility::default);
        let (_, todos) = state(|| default_todos.into());

        Self { todos, visibility }
    }

    pub fn enter<T>(self, f: &mut dyn FnMut() -> T) -> T {
        illicit::Layer::new().offer(self.todos).offer(self.visibility).enter(f)
    }

    pub fn boot<Root: Node + 'static>(
        default_todos: &[Todo],
        node: impl Into<moxie_dom::raw::Node>,
        mut root: impl FnMut() -> Root + 'static,
    ) {
        let defaults = default_todos.to_vec();
        moxie_dom::boot(node, move || App::current(&defaults).enter(&mut root));
    }
}

#[derive(Clone, Debug)]
pub struct Todo {
    id: u32,
    text: String,
    completed: bool,
}

impl Todo {
    fn new(s: impl Into<String>) -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        Self { id: NEXT_ID.fetch_add(1, Ordering::SeqCst), text: s.into(), completed: false }
    }
}

#[wasm_bindgen(start)]
pub fn begin() {
    console_log::init_with_level(tracing::log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{:#?}", info);
    }));
    App::boot(&[Todo::new("whoaaa")], document().body().unwrap(), todo_app);
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
