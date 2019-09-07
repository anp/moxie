use {
    crate::{filter::*, footer::*, input::*, Todo},
    moxie_dom::{element, prelude::*, text},
};

#[topo::aware]
pub fn toggle(default_checked: bool) {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();
    let toggle_to = !self.default_checked;
    element!("span").inner(|| {
        element!("input")
            .attr("class", "toggle-all")
            .attr("type", "checkbox")
            .attr("defaultChecked", self.default_checked);

        element!("label").on(
            move |_: ClickEvent, todos| -> Option<Vec<Todo>> {
                todos
                    .iter()
                    .map(|t| {
                        let mut new = t.clone();
                        new.completed = toggle_to;
                        new
                    })
                    .collect::<Vec<_>>()
                    .into()
            },
            todos.clone(),
        );
    });
}

#[topo::aware]
pub fn todo_item(todo: &Todo) {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();
    let editing = state!(|| false);

    let mut classes = String::new();
    if self.todo.completed {
        classes.push_str("completed ");
    }
    if *editing {
        classes.push_str("editing");
    }

    let id = self.todo.id;

    element!("li").attr("class", classes).inner(|| {
        if *editing {
            let todos = todos;
            let this_todo = self.todo;
            let placeholder = this_todo.text.clone();

            text_input!(placeholder, true, move |value: String| {
                editing.set(false);
                todos.update(|todos| {
                    let mut todos = todos.to_vec();
                    if let Some(mut todo) = todos.iter_mut().find(|t| t.id == this_todo.id) {
                        todo.text = value;
                    }
                    Some(todos)
                });
            });
        } else {
            element!("div").attr("class", "view").inner(|| {
                element!("input")
                    .attr("class", "toggle")
                    .attr("type", "checkbox")
                    .attr("checked", self.todo.completed)
                    .on(
                        move |_: ChangeEvent, todos| {
                            Some(
                                todos
                                    .iter()
                                    .cloned()
                                    .map(move |mut t| {
                                        if t.id == id {
                                            t.completed = !t.completed;
                                            t
                                        } else {
                                            t
                                        }
                                    })
                                    .collect(),
                            )
                        },
                        todos.clone(),
                    );

                element!("label")
                    .on(|_: DoubleClickEvent, _editing| Some(true), editing)
                    .inner(|| text!(self.todo.text));

                element!("button").attr("class", "destroy").on(
                    move |_: ClickEvent, todos| {
                        Some(todos.iter().filter(|t| t.id != id).cloned().collect())
                    },
                    todos.clone(),
                );
            });
        }
    });
}

#[topo::aware]
pub fn todo_list() {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();
    let visibility = topo::Env::expect::<Key<Visibility>>();
    element!("ul").attr("class", "todo-list").inner(|| {
        for todo in todos {
            if visibility.should_show(todo) {
                todo_item!(todo);
            }
        }
    });
}

#[topo::aware]
pub fn main_section() {
    let todos = topo::Env::expect::<Key<Vec<Todo>>>();
    let num_complete = todos.iter().filter(|t| t.completed).count();

    element!("section").attr("class", "main").inner(move || {
        if !todos.is_empty() {
            toggle!(num_complete == todos.len());
        }

        todo_list!();

        if !todos.is_empty() {
            footer!(num_complete, todos.len() - num_complete);
        }
    });
}
