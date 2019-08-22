use {
    crate::{filter::Visibility, footer::Footer, input::TextInput, Todo},
    moxie_dom::prelude::*,
};

#[derive(Clone, Debug)]
pub struct MainSection;

impl Component for MainSection {
    fn contents(self) {
        let todos = topo::Env::expect::<Key<Vec<Todo>>>();
        let num_complete = todos.iter().filter(|t| t.completed).count();

        show!(element("section").attr("class", "main").inner(move || {
            if !todos.is_empty() {
                show!(Toggle {
                    default_checked: num_complete == todos.len(),
                })
            }

            show!(TodoList);

            if !todos.is_empty() {
                show!(Footer {
                    num_complete,
                    num_active: todos.len() - num_complete,
                });
            }
        }));
    }
}

#[derive(Debug, Default)]
struct TodoList;

impl Component for TodoList {
    fn contents(self) {
        let todos = topo::Env::expect::<Key<Vec<Todo>>>();
        let visibility = topo::Env::expect::<Key<Visibility>>();
        show!(element("ul").attr("class", "todo-list").inner(|| {
            for todo in todos.iter().filter(|t| visibility.should_show(t)) {
                show!(TodoItem {
                    todo: todo.to_owned(),
                });
            }
        }))
    }
}

#[derive(Debug)]
struct TodoItem {
    todo: Todo,
}

impl Component for TodoItem {
    fn contents(self) {
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

        show!(element("li").attr("class", classes).inner(|| {
            if *editing {
                let todos = todos;
                let this_todo = self.todo;
                let placeholder = this_todo.text.clone();

                show!(TextInput {
                    placeholder,
                    editing: true,
                    on_save: move |value: String| {
                        editing.set(false);
                        todos.update(|todos| {
                            let mut todos = todos.to_vec();
                            if let Some(mut todo) = todos.iter_mut().find(|t| t.id == this_todo.id)
                            {
                                todo.text = value;
                            }
                            Some(todos)
                        });
                    },
                });
            } else {
                show!(element("div")
                    .attr("class", "view")
                    .child(
                        element("input")
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
                                todos.clone()
                            )
                    )
                    .child(
                        element("label")
                            .on(|_: DoubleClickEvent, _editing| { Some(true) }, editing)
                            .child(text!(self.todo.text))
                    )
                    .child(element("button").attr("class", "destroy").on(
                        move |_: ClickEvent, todos| {
                            Some(todos.iter().filter(|t| t.id != id).cloned().collect())
                        },
                        todos.clone()
                    )))
            }
        }))
    }
}

#[derive(Debug)]
struct Toggle {
    default_checked: bool,
}

impl Component for Toggle {
    fn contents(self) {
        let todos = topo::Env::expect::<Key<Vec<Todo>>>();
        let toggle_to = !self.default_checked;
        show!(element("span")
            .child(
                element("input")
                    .attr("class", "toggle-all")
                    .attr("type", "checkbox")
                    .attr("defaultChecked", self.default_checked)
            )
            .child(element("label").on(
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
            )));
    }
}
