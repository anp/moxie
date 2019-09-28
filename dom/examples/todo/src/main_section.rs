use {
    crate::{filter::*, footer::*, item::*, Todo},
    moxie_dom::{input, label, moxml, prelude::*, section, span, ul},
};

#[topo::aware]
#[topo::from_env(todos: Key<Vec<Todo>>)]
pub fn toggle(default_checked: bool) {
    let todos = todos.clone();
    let on_click = move |_: ClickEvent| {
        todos.update(|t| {
            Some(
                t.iter()
                    .map(|t| {
                        let mut new = t.clone();
                        new.completed = !default_checked;
                        new
                    })
                    .collect::<Vec<_>>()
                    .into(),
            )
        })
    };

    moxml! {
        <span>
            <input class="toggle-all" type="checkbox" defaultChecked={default_checked} />
            <label on={on_click}/>
        </span>
    };
}

#[topo::aware]
#[topo::from_env(todos: Key<Vec<Todo>>, visibility: Key<Visibility>)]
pub fn todo_list() {
    moxml! {
        <ul class="todo-list">
        {
            for todo in todos.iter() {
                if visibility.should_show(todo) {
                    todo_item!(todo);
                }
            }
        }
        </ul>
    };
}

#[topo::aware]
#[topo::from_env(todos: Key<Vec<Todo>>)]
pub fn main_section() {
    let num_complete = todos.iter().filter(|t| t.completed).count();

    moxml! {
        <section class="main">
        {
            if !todos.is_empty() {
                toggle!(num_complete == todos.len());
            }
        }

        <todo_list/>

        {
            if !todos.is_empty() {
                footer!(num_complete, todos.len() - num_complete);
            }
        }
        </section>
    }
}

// TODO test where:
/*
assert starting list of todos matches starting elements
switch filter to active, everything stays the same
switch filter to completed, list is empty
*/
