use {
    header::*,
    moxie_dom::{elements::*, events::*, *},
    std::sync::atomic::{AtomicU32, Ordering},
    stdweb::{traits::*, *},
    tracing::*,
};

pub mod header;

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
struct TodoApp;

impl Component for TodoApp {
    fn contents(self) {
        let (visibility, visibility_key) = state!(|| Visibility::default());
        let (todos, todos_key): (Commit<Vec<Todo>>, _) = state!((), |()| vec![Todo::new("whoaaa")]);

        show_many!(Header::new(todos_key.clone()), MainSection);
    }
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

struct TodoTextInput {}

fn main() {
    web_logger::init();
    std::panic::set_hook(Box::new(|info| {
        error!("{:#?}", info);
    }));
    mount!(web::document().body().unwrap(), TodoApp);
}
