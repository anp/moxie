use crate::{input::text_input, Todo};
use mox::mox;
use moxie_dom::{
    elements::{
        forms::Input,
        html::*,
        text_content::{Div, Li},
    },
    prelude::*,
};

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
fn item_edit_input(todo: Todo, editing: Key<bool>) -> Input {
    let todos = todos.clone();
    let text = todo.title.clone();
    text_input(&text, true, move |value: String| {
        editing.set(false);
        todos.update(|todos| {
            let mut todos = todos.to_vec();
            if let Some(mut todo) = todos.iter_mut().find(|t| t.id == todo.id) {
                todo.title = value;
            }
            Some(todos)
        });
    })
}

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
fn item_with_buttons(todo: Todo, editing: Key<bool>) -> Div {
    let todos = todos.clone();
    let id = todo.id;
    let toggle_todos = todos.clone();

    let on_click = move |_| {
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
            <input class="toggle" type="checkbox" checked={todo.completed} onclick={on_click} />

            <label ondblclick={move |_| editing.set(true)}>
                {% "{}", todo.title }
            </label>

            <button class="destroy" onclick={move |_| {
                todos.update(|t| Some(t.iter().filter(|t| t.id != id).cloned().collect()));
            }} />
        </div>
    }
}

#[topo::nested]
pub fn todo_item(todo: &Todo) -> Li {
    let (editing, set_editing) = state(|| false);

    let mut classes = String::new();
    if todo.completed {
        classes.push_str("completed ");
    }
    if *editing {
        classes.push_str("editing");
    }

    let mut item = li();
    item = item.class(classes);

    if *editing {
        item = item.child(item_edit_input(todo.clone(), set_editing));
    } else {
        item = item.child(item_with_buttons(todo.clone(), set_editing));
    }

    item.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[wasm_bindgen_test::wasm_bindgen_test]
    pub async fn single_item() {
        let root = document().create_element("div").unwrap();
        crate::App::boot(&[Todo::new("weeeee")], root.clone(), || {
            let todo = &illicit::expect::<Key<Vec<Todo>>>()[0];
            todo_item(todo)
        });

        assert_eq!(
            root.pretty_outer_html(2),
            r#"<div>
  <li class="">
    <div class="view">
      <input class="toggle" type="checkbox" checked="false">
      </input>
      <label>weeeee</label>
      <button class="destroy">
      </button>
    </div>
  </li>
</div>"#
        );
    }
}
