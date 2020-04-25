use crate::{filter::filter, Todo};
use moxie_dom::{
    elements::{button, footer, span, strong},
    prelude::*,
};

#[topo::nested]
pub fn items_remaining(num_active: usize) {
    mox! {
        <span class="todo-count">
            <strong>
            {
                if num_active == 0 {
                    text("No")
                } else {
                    text(num_active)
                }
            }
            </strong>
            {% " {} left", if num_active == 1 { "item" } else { "items" } }
        </span>
    };
}

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
pub fn clear_completed_button() {
    let todos = todos.clone();
    mox! {
        <button class="clear-completed" onclick={ move |_|
            todos.update(|t| Some(t.iter().filter(|t| !t.completed).cloned().collect()))
        }>
            "Clear completed"
        </button>
    };
}

#[topo::nested]
pub fn filter_footer(num_complete: usize, num_active: usize) {
    mox! {
        <footer class="footer">
            <items_remaining _=(num_active)/>
            <filter/>
            {
                if num_complete > 0 {
                    mox! { <clear_completed_button/> };
                }
            }
        </footer>
    }
}
