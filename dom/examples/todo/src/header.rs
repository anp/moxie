use {
    crate::{input::TextInput, Todo},
    moxie_dom::prelude::*,
    tracing::info,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Header;

impl Component for Header {
    fn contents(self) {
        let todos = topo::Env::expect::<Key<Vec<Todo>>>();

        show!(element("header")
            .attr("class", "header")
            .child(element("h1").child(text!("todos")))
            .child(TextInput {
                placeholder: "What needs to be done?".into(),
                editing: false,
                on_save: move |value: String| {
                    todos.update(|prev| {
                        let mut todos: Vec<Todo> = prev.to_vec();
                        todos.push(Todo::new(value));
                        info!({ ?todos }, "added new todo");
                        Some(todos)
                    });
                }
            }))
    }
}
