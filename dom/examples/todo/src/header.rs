use {
    moxie_dom::{elements::*, events::*, *},
    stdweb::{traits::*, *},
    tracing::*,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    todos: Key<Vec<Todo>>,
}

impl Header {
    fn new(todos: Key<Vec<Todo>>) -> Self {
        Self { todos }
    }
}

impl Component for Header {
    fn contents(self) {
        // show!(header().class_name("header").contains(sibs![
        //     h1().contains(text!("todos")),
        //     // TodoTextInput::new().on_save()
        // ]))
    }
}
