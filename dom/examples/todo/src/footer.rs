use {
    crate::{filter::*, Todo},
    moxie_dom::{moxml, prelude::*, text},
};

#[topo::aware]
pub fn items_remaining(num_active: usize) {
    moxml! {
        <span class="todo-count">
            <strong>
            {
                if num_active == 0 {
                    text!("No")
                } else {
                    text!(num_active)
                }
            }
            </strong>
            {% " {} left", if num_active == 1 { "item" } else { "items" } }
        </span>
    };
}

#[topo::aware]
#[topo::from_env(todos: Key<Vec<Todo>>)]
pub fn clear_completed_button() {
    let todos = todos.clone();
    let on_click = move |_: ClickEvent| {
        todos.update(|t| Some(t.iter().filter(|t| !t.completed).cloned().collect()))
    };
    moxml! {
        <button class="clear-completed" on={on_click}>
            "Clear completed"
        </button>
    };
}

#[topo::aware]
pub fn footer(num_complete: usize, num_active: usize) {
    moxml! {
        <footer class="footer">
        {
            items_remaining!(num_active);
            filter!();

            if num_complete > 0 {
                clear_completed_button!();
            }
        }
        </footer>
    }
}
