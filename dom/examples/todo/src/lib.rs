#![feature(track_caller)]

use filter::Visibility;
use header::input_header;
use main_section::main_section;
use moxie_dom::{elements::text_content::div, prelude::*};
use std::sync::atomic::{AtomicU32, Ordering};
use wasm_bindgen::prelude::*;

pub mod filter;
pub mod footer;
pub mod header;
pub mod input;
pub mod item;
pub mod main_section;

#[topo::nested]
fn todo_app() {
    let visibility = state(Visibility::default);
    let todos = state(|| vec![Todo::new("whoaaa")]);

    illicit::child_env![
        Key<Vec<Todo>> => todos,
        Key<Visibility> => visibility
    ]
    .enter(|| {
        topo::call(|| {
            mox! {
                <div class="todoapp">
                    <input_header/>
                    <main_section/>
                </div>
            }
        });
    });
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
pub fn main() {
    console_log::init_with_level(tracing::log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{:#?}", info);
    }));
    moxie_dom::boot(document().body().unwrap(), todo_app);
}
