use {
    crate::{filter::*, Todo},
    moxie_dom::{element, prelude::*, text},
};

#[topo::aware]
pub fn footer(num_complete: usize, num_active: usize) {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();

    element!("footer").attr("class", "footer").inner(|| {
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
        filter!();

        if num_complete > 0 {
            element!("button")
                .attr("class", "clear-completed")
                .on(
                    |_: ClickEvent, todos| {
                        Some(todos.iter().filter(|t| !t.completed).cloned().collect())
                    },
                    todos.clone(),
                )
                .inner(|| text!("Clear completed"));
        }
    });
}
