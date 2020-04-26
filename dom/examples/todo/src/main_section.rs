use crate::{filter::*, footer::*, item::todo_item, Todo};
use moxie_dom::{elements::all::*, prelude::*};

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
pub fn toggle(default_checked: bool) {
    let todos = todos.clone();
    let on_click = move |_| {
        todos.update(|t| {
            Some(
                t.iter()
                    .map(|t| {
                        let mut new = t.clone();
                        new.completed = !default_checked;
                        new
                    })
                    .collect(),
            )
        })
    };

    mox! {
        <span>
            <input class="toggle-all" type="checkbox" defaultChecked={default_checked} />
            <label onclick={on_click}/>
        </span>
    };
}

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>, visibility: &Key<Visibility>)]
pub fn todo_list() {
    mox! {
        <ul class="todo-list">
        {
            for todo in todos.iter() {
                if visibility.should_show(todo) {
                    mox! { <todo_item _=(todo)/> };
                }
            }
        }
        </ul>
    };
}

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
pub fn main_section() {
    let num_complete = todos.iter().filter(|t| t.completed).count();

    mox! {
        <section class="main">
        {
            if !todos.is_empty() {
                mox! {
                    <toggle _=(num_complete == todos.len())/>
                };
            }
        }

        <todo_list/>

        {
            if !todos.is_empty() {
                mox! {
                    <filter_footer _=(num_complete, todos.len() - num_complete)/>
                };
            }
        }
        </section>
    }
}

// TODO test where:
// assert starting list of todos matches starting elements
// switch filter to active, everything stays the same
// switch filter to completed, list is empty
