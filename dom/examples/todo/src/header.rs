use {
    crate::{input::*, Todo},
    moxie_dom::{element, prelude::*, text},
    tracing::info,
};

#[topo::aware]
pub fn header() {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();

    element!("header", |e| e.attr("class", "header").inner(|| {
        element!("h1", |e| e.inner(|| text!("todos")));
        text_input!("What needs to be done?", false, move |value: String| {
            todos.update(|prev| {
                let mut todos: Vec<Todo> = prev.to_vec();
                todos.push(Todo::new(value));
                info!({ ?todos }, "added new todo");
                Some(todos)
            });
        });
    }));
}
