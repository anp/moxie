#![allow(unused_imports)]
use {
    header::*,
    moxie_dom::{elements::*, events::*, *},
    std::sync::atomic::{AtomicU32, Ordering},
    tracing::*,
    wasm_bindgen::prelude::*,
};

pub mod header;

#[derive(Clone, Debug, PartialEq)]
struct TodoApp;

impl Component for TodoApp {
    fn contents(self) {
        let (_visibility, _visibility_key) = state!(|| Visibility::default());
        let (_todos, todos_key) = state!(|| vec![Todo::new("whoaaa")]);

        show!(Header::new(todos_key), MainSection);
    }
}

#[derive(Debug)]
pub struct Todo {
    id: u32,
    text: String,
    completed: bool,
}

impl Todo {
    fn new(s: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            text: s.into(),
            completed: false,
        }
    }
}

fn next_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Clone, Debug, PartialEq)]
struct MainSection;

impl Component for MainSection {
    fn contents(self) {}
}

enum Visibility {
    All,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::All
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
