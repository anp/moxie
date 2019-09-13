use {
    crate::{filter::*, footer::*, item::*, Todo},
    moxie_dom::{element, prelude::*},
};

#[topo::aware]
pub fn toggle(default_checked: bool) {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();
    element!("span", |e| e.inner(|| {
        element!("input", |e| {
            e.attr("class", "toggle-all")
                .attr("type", "checkbox")
                .attr("defaultChecked", default_checked);
        });

        element!("label", |e| {
            e.on(
                move |_: ClickEvent, todos| -> Option<Vec<Todo>> {
                    todos
                        .iter()
                        .map(|t| {
                            let mut new = t.clone();
                            new.completed = !default_checked;
                            new
                        })
                        .collect::<Vec<_>>()
                        .into()
                },
                todos.clone(),
            );
        });
    }));
}

#[topo::aware]
pub fn todo_list() {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();
    let visibility = topo::Env::expect::<Key<Visibility>>();
    element!("ul", |e| e.attr("class", "todo-list").inner(|| {
        for todo in todos.iter() {
            if visibility.should_show(todo) {
                todo_item!(todo);
            }
        }
    }));
}

#[topo::aware]
pub fn main_section() {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();
    let num_complete = todos.iter().filter(|t| t.completed).count();

    element!("section", |e| e.attr("class", "main").inner(move || {
        if !todos.is_empty() {
            toggle!(num_complete == todos.len());
        }

        todo_list!();

        if !todos.is_empty() {
            footer!(num_complete, todos.len() - num_complete);
        }
    }));
}

// TODO test where:
/*
assert starting list of todos matches starting elements
switch filter to active, everything stays the same
switch filter to completed, list is empty
*/
