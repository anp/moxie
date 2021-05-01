use crate::{filter::*, footer::*, item::todo_item, Todo};
use mox::mox;
use moxie_dom::{
    elements::{html::*, sectioning::Section, text_content::Ul, text_semantics::Span},
    prelude::*,
};
use tracing::info;

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
pub fn toggle(default_checked: bool) -> Span {
    let todos = todos.clone();
    let onchange = move |_| {
        info!("toggling item completions");
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
            <input id="toggle" class="toggle-all" type="checkbox" checked=default_checked onchange />
            <label for="toggle" />
        </span>
    }
}

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>, visibility: &Key<Visibility>)]
pub fn todo_list() -> Ul {
    let mut list = ul().class("todo-list");
    for todo in todos.iter() {
        if visibility.should_show(todo) {
            list = list.child(todo_item(todo));
        }
    }
    list.build()
}

#[topo::nested]
#[illicit::from_env(todos: &Key<Vec<Todo>>)]
pub fn main_section() -> Option<Section> {
    if !todos.is_empty() {
        let num_complete = todos.iter().filter(|t| t.completed).count();

        Some(mox! {
          <section class="main">
              { toggle(num_complete == todos.len()) }
              { todo_list() }
              { filter_footer(num_complete, todos.len() - num_complete) }
          </section>
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[wasm_bindgen_test::wasm_bindgen_test]
    pub async fn list_filtering() {
        let root = document().create_element("div");
        crate::App::boot_fn(
            &[Todo::new("first"), Todo::new("second"), Todo::new("third")],
            root.clone(),
            || main_section().unwrap(),
        );

        assert_eq!(
            root.pretty_outer_html(2),
            r#"<div>
  <section class="main">
    <span>
      <input id="toggle" class="toggle-all" type="checkbox" checked="false">
      </input>
      <label for="toggle">
      </label>
    </span>
    <ul class="todo-list">
      <li class="">
        <div class="view">
          <input class="toggle" type="checkbox" checked="false">
          </input>
          <label>first</label>
          <button class="destroy">
          </button>
        </div>
      </li>
      <li class="">
        <div class="view">
          <input class="toggle" type="checkbox" checked="false">
          </input>
          <label>second</label>
          <button class="destroy">
          </button>
        </div>
      </li>
      <li class="">
        <div class="view">
          <input class="toggle" type="checkbox" checked="false">
          </input>
          <label>third</label>
          <button class="destroy">
          </button>
        </div>
      </li>
    </ul>
    <footer class="footer">
      <span class="todo-count">
        <strong>3</strong> items left</span>
      <ul class="filters">
        <li>
          <a style="cursor: pointer;" class="selected">All</a>
        </li>
        <li>
          <a style="cursor: pointer;" class="">Active</a>
        </li>
        <li>
          <a style="cursor: pointer;" class="">Completed</a>
        </li>
      </ul>
    </footer>
  </section>
</div>"#
        );
    }
}
