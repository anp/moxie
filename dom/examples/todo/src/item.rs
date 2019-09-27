use {
    crate::{input::*, Todo},
    moxie_dom::{moxml, prelude::*},
};

#[topo::aware]
#[topo::from_env(todos: Key<Vec<Todo>>)]
fn item_edit_input(todo: Todo, editing: Key<bool>) {
    let todos = todos.clone();
    text_input!(&todo.text.clone(), true, move |value: String| {
        editing.set(false);
        todos.update(|todos| {
            let mut todos = todos.to_vec();
            if let Some(mut todo) = todos.iter_mut().find(|t| t.id == todo.id) {
                todo.text = value;
            }
            Some(todos)
        });
    });
}

#[topo::aware]
#[topo::from_env(todos: Key<Vec<Todo>>)]
fn item_with_buttons(todo: Todo, editing: Key<bool>) {
    let id = todo.id;
    let todos = todos.clone();
    let toggle_todos = todos.clone();

    let on_change = move |_: ChangeEvent| {
        toggle_todos.update(|t| {
            Some(
                t.iter()
                    .cloned()
                    .map(move |mut t| {
                        if t.id == id {
                            t.completed = !t.completed;
                            t
                        } else {
                            t
                        }
                    })
                    .collect(),
            )
        })
    };

    moxml! {
        <div class="view">
            <input class="toggle" type="checkbox" checked={todo.completed} on={on_change} />

            <label on={move |_: DoubleClickEvent| editing.set(true)}>
                {% "{}", todo.text }
            </label>

            <button class="destroy" on={move |_: ClickEvent| {
                todos.update(|t| Some(t.iter().filter(|t| t.id != id).cloned().collect()));
            }} />
        </div>
    };
}

#[topo::aware]
pub fn todo_item(todo: &Todo) {
    let editing = state!(|| false);

    let mut classes = String::new();
    if todo.completed {
        classes.push_str("completed ");
    }
    if *editing {
        classes.push_str("editing");
    }

    moxml! {
        <li class={classes}>
        {
            if *editing {
                item_edit_input!(todo.clone(), editing);
            } else {
                item_with_buttons!(todo.clone(), editing);
            }
        }
        </li>
    };
}
