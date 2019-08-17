use {
    crate::{input::TextInput, Todo},
    moxie_dom::prelude::*,
    tracing::info,
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
        let Self { todos } = self;

        let on_save: Box<dyn Fn(String)> = Box::new(move |value: String| {
            todos.update(|prev| {
                let mut todos: Vec<Todo> = prev.to_vec();
                todos.push(Todo::new(value));
                info!({ ?todos }, "added new todo");
                Some(todos)
            });
        });

        show!(element("header")
            .attr("class", "header")
            .child(element("h1").child(text!("todos")))
            .child(TextInput {
                placeholder: "What needs to be done?".into(),
                editing: false,
                on_save
            }))
    }
}
