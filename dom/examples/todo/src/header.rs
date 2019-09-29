use {
    crate::{input::*, Todo},
    moxie_dom::*,
    tracing::info,
};

#[topo::aware]
#[topo::from_env(todos: Key<Vec<Todo>>)]
pub fn input_header() {
    let todos = todos.clone();
    mox! {
        <header class="header">
            <h1>"todos"</h1>
            <text_input _=(
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
            )/>
        </header>
    };
}
