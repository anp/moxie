#![allow(unused_imports)]
use {
    header::*,
    main_section::MainSection,
    moxie_dom::{elements::*, events::*, *},
    std::sync::atomic::{AtomicU32, Ordering},
    tracing::*,
    wasm_bindgen::prelude::*,
};

pub mod footer;
pub mod header;
pub mod main_section;

#[derive(Clone, Debug, PartialEq)]
struct TodoApp;

impl Component for TodoApp {
    fn contents(self) {
        let visibility = state!(|| footer::Visibility::default());
        let todos = state!(|| vec![Todo::new("whoaaa")]);

        show!(element("div")
            .attr("class", "todoapp")
            .child(Header::new(todos.clone()))
            .child(MainSection::new(todos, visibility)));
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
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
            text: s.into(),
            completed: false,
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(|info| {
        error!("{:#?}", info);
    }));
    mount!(document().body().unwrap(), TodoApp);
}
