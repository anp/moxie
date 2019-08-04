use {
    super::*,
    moxie_dom::{elements::*, events::*, *},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    todos: Key<Vec<Todo>>,
}

impl Header {
    pub fn new(todos: Key<Vec<Todo>>) -> Self {
        Self { todos }
    }
}

impl Component for Header {
    fn contents(self) {
        show!(element("header")
            .attr("class", "header")
            .child(element("h1").child(text!("todos"))))
    }
}
