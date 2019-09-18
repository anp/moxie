use {
    crate::{filter::*, Todo},
    moxie_dom::{element, prelude::*, text},
};

#[topo::aware]
pub fn items_remaining(num_active: usize) {
    element!("span", |e| e
        .attr("class", "todo-count")
        // lol this is awful
        .inner(|| {
            element!("strong", |e| e.inner(|| {
                if num_active == 0 {
                    text!("No")
                } else {
                    text!(num_active)
                }
            }));
            text!(format!(
                " {} left",
                if num_active == 1 { "item" } else { "items" }
            ));
        }));
}

#[topo::aware]
#[topo::from_env(todos: Key<Vec<Todo>>)]
pub fn clear_completed_button() {
    element!("button", |e| e
        .attr("class", "clear-completed")
        .on(
            |_: ClickEvent, todos| Some(todos.iter().filter(|t| !t.completed).cloned().collect()),
            todos.clone(),
        )
        .inner(|| text!("Clear completed")));
}

#[topo::aware]
pub fn footer(num_complete: usize, num_active: usize) {
    element!("footer", |e| e.attr("class", "footer").inner(|| {
        items_remaining!(num_active);
        filter!();

        if num_complete > 0 {
            clear_completed_button!();
        }
    }));
}
