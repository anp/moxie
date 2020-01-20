use crate::{input::text_input, Todo};
use moxie_dom::{
    elements::{button, div, input, label, li},
    prelude::*,
};

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
fn item_edit_input(todo: Todo, editing: Key<bool>) {
    let todos = todos.clone();
    mox! {
        <text_input _=(
            &todo.text.clone(),
            true,
            move |value: String| {
                editing.set(false);
                todos.update(|todos| {
                    let mut todos = todos.to_vec();
                    if let Some(mut todo) = todos.iter_mut().find(|t| t.id == todo.id) {
                        todo.text = value;
                    }
                    Some(todos)
                });
            },
        )/>
    };
}

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
fn item_with_buttons(todo: Todo, editing: Key<bool>) {
    let id = todo.id;
    let todos = todos.clone();
    let toggle_todos = todos.clone();

    let on_change = move |_: event::Change| {
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

    mox! {
        <div class="view">
            <input class="toggle" type="checkbox" checked={todo.completed} on={on_change} />

            <label on={move |_: event::DoubleClick| editing.set(true)}>
                {% "{}", todo.text }
            </label>

            <button class="destroy" on={move |_: event::Click| {
                todos.update(|t| Some(t.iter().filter(|t| t.id != id).cloned().collect()));
            }} />
        </div>
    };
}

#[topo::nested]
pub fn todo_item(todo: &Todo) {
    let editing = state(|| false);

    let mut classes = String::new();
    if todo.completed {
        classes.push_str("completed ");
    }
    if *editing {
        classes.push_str("editing");
    }

    mox! {
        <li class={classes}>
        {
            if *editing {
                mox! {
                    <item_edit_input _=(todo.clone(), editing) />
                };
            } else {
                mox! {
                    <item_with_buttons _=(todo.clone(), editing)/>
                };
            }
        }
        </li>
    };
}
