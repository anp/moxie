use crate::{input::text_input, Todo};
use moxie_dom::{
    elements::sectioning::{h1, header, Header},
    prelude::*,
};
use tracing::info;

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
pub fn input_header() -> Header {
    let todos = todos.clone();
    mox! {
        <header class="header">
            <h1>"todos"</h1>
            { text_input(
                "What needs to be done?",
                false,
                move |value: String| {
                    todos.update(|prev| {
                        let mut todos: Vec<Todo> = prev.to_vec();
                        todos.push(Todo::new(value));
                        info!({ ?todos }, "added new todo");
                        Some(todos)
                    });
                },
            )}
        </header>
    }
}
