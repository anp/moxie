use {
    crate::{filter::*, Todo},
    moxie_dom::{element, prelude::*, text},
};

#[topo::aware]
pub fn items_remaining(num_active: usize) {
    element!("span")
        .attr("class", "todo-count")
        // lol this is awful
        .inner(|| {
            element!("strong").inner(|| {
                if num_active == 0 {
                    text!("No")
                } else {
                    text!(num_active)
                }
            });
            text!(format!(
                " {} left",
                if num_active == 1 { "item" } else { "items" }
            ))
        });
}

#[topo::aware]
pub fn clear_completed_button() {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();

    element!("button")
        .attr("class", "clear-completed")
        .on(
            |_: ClickEvent, todos| Some(todos.iter().filter(|t| !t.completed).cloned().collect()),
            todos.clone(),
        )
        .inner(|| text!("Clear completed"));
}

#[topo::aware]
pub fn footer(num_complete: usize, num_active: usize) {
    element!("footer").attr("class", "footer").inner(|| {
        items_remaining!(num_active);
        filter!();

        if num_complete > 0 {
            clear_completed_button!();
        }
    });
}
