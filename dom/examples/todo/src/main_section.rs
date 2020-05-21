use crate::{filter::*, footer::*, item::todo_item, Todo};
use moxie_dom::{
    elements::{html::*, sectioning::Section, text_content::Ul, text_semantics::Span},
    prelude::*,
};

#[topo::nested]
#[illicit::from_env(todos: Key<Vec<Todo>>)]
pub fn toggle(default_checked: bool) -> Span {
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
            <input class="toggle-all" type="checkbox" checked={default_checked} />
            <label onclick={on_click}/>
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
pub fn main_section() -> Section {
    let num_complete = todos.iter().filter(|t| t.completed).count();

    let mut section = section().class("main");

    if !todos.is_empty() {
        section = section.child(toggle(num_complete == todos.len()));
    }
    section = section.child(todo_list());

    if !todos.is_empty() {
        section = section.child(filter_footer(num_complete, todos.len() - num_complete));
    }

    section.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use moxie_dom::raw::testing::Query;
    use pretty_assertions::assert_eq;

    #[wasm_bindgen_test::wasm_bindgen_test]
    pub async fn list_filtering() {
        let root = document().create_element("div").unwrap();
        // document().body().unwrap().append_child(&root).unwrap();
        crate::App::boot(
            &[Todo::new("first"), Todo::new("second"), Todo::new("third")],
            root.clone(),
            main_section,
        );

        root.find().by_text("first").until().many().await;
        assert_eq!(
            root.pretty_outer_html(2),
            r#"<div>
  <section class="main">
    <span>
      <input class="toggle-all" type="checkbox" checked="false">
      </input>
      <label>
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
