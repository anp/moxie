use {
    crate::{filter::Visibility, footer::Footer, input::TextInput, Todo},
    moxie_dom::prelude::*,
};

#[derive(Clone, Debug, PartialEq)]
pub struct MainSection {
    todos: Key<Vec<Todo>>,
    visibility: Key<Visibility>,
}

impl MainSection {
    pub fn new(todos: Key<Vec<Todo>>, visibility: Key<Visibility>) -> Self {
        Self { todos, visibility }
    }
}

impl Component for MainSection {
    fn contents(self) {
        let todos_empty = self.todos.is_empty();
        let todos_count = self.todos.len();
        let completed_count = self.todos.iter().filter(|t| t.completed).count();

        show!(element("section").attr("class", "main").inner(move || {
            if !todos_empty {
                show!(Toggle {
                    default_checked: completed_count == todos_count,
                    todos: self.todos.clone(),
                })
            }

            show!(TodoList {
                todos: self.todos.clone(),
                visibility: self.visibility.clone(),
            });

            if !todos_empty {
                show!(Footer {
                    completed_count,
                    active_count: todos_count - completed_count,
                    todos: self.todos,
                    visibility: self.visibility,
                });
            }
        }));
    }
}

#[derive(Debug)]
struct TodoList {
    todos: Key<Vec<Todo>>,
    visibility: Key<Visibility>,
}

impl Component for TodoList {
    fn contents(self) {
        show!(element("ul").attr("class", "todo-list").inner(|| {
            for todo in self.todos.iter().filter(|t| self.visibility.should_show(t)) {
                show!(TodoItem {
                    todo: todo.to_owned(),
                    todos: self.todos.clone(),
                });
            }
        }))
    }
}

#[derive(Debug)]
struct TodoItem {
    todo: Todo,
    todos: Key<Vec<Todo>>,
}

impl Component for TodoItem {
    fn contents(self) {
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
                let todos = self.todos;
                let this_todo = self.todo;
                let placeholder = this_todo.text.clone();

                let on_save: Box<dyn Fn(String)> = Box::new(move |value: String| {
                    editing.set(false);
                    todos.update(|todos| {
                        let mut todos = todos.to_vec();
                        if let Some(mut todo) =
                            todos.iter_mut().filter(|t| t.id == this_todo.id).next()
                        {
                            todo.text = value;
                        }
                        Some(todos)
                    });
                });

                show!(TextInput {
                    placeholder,
                    editing: true,
                    on_save,
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
                                self.todos.clone()
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
                        self.todos
                    )))
            }
        }))
    }
}

#[derive(Debug)]
struct Toggle {
    default_checked: bool,
    todos: Key<Vec<Todo>>,
}

impl Component for Toggle {
    fn contents(self) {
        show!(element("span")
            .child(
                element("input")
                    .attr("class", "toggle-all")
                    .attr("type", "checkbox")
                    .attr("defaultChecked", self.default_checked)
            )
            .child(element("label").on(
                |_: ClickEvent, todos| -> Option<Vec<Todo>> {
                    todos
                        .iter()
                        .map(|t| {
                            let mut new = t.clone();
                            new.completed = true;
                            new
                        })
                        .collect::<Vec<_>>()
                        .into()
                },
                self.todos
            )));
    }
}
