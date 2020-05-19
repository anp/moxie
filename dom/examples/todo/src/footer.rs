use crate::{filter::filter, Todo};
use moxie_dom::{elements::html::*, prelude::*};

#[topo::nested]
pub fn items_remaining(num_active: usize) -> impl Node {
    let bolded = if num_active == 0 { text("No") } else { text(num_active) };
    mox! {
        <span class="todo-count">
            <strong>{bolded}</strong>
            {% " {} left", if num_active == 1 { "item" } else { "items" } }
        </span>
    }
}

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
pub fn clear_completed_button(num_complete: usize) -> impl Node {
    let todos = todos.clone();
    let remove_completed =
        move |_| todos.update(|t| Some(t.iter().filter(|t| !t.completed).cloned().collect()));
    mox! {
        <button class="clear-completed" disabled={num_complete == 0} onclick={remove_completed}>
            "Clear completed"
        </button>
    }
}

#[topo::nested]
pub fn filter_footer(num_complete: usize, num_active: usize) -> impl Node {
    let mut footer =
        footer().class("footer").child(items_remaining(num_active).build()).child(filter().build());

    if num_complete > 0 {
        footer = footer.child(clear_completed_button(num_complete).build());
    }

    footer.build()
}
