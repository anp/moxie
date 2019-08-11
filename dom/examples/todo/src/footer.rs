use {
    super::*,
    moxie_dom::{elements::*, events::*, *},
};

#[derive(Debug)]
pub enum Visibility {
    All,
    Active,
    Completed,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::All
    }
}

impl Visibility {
    pub fn should_show(&self, todo: &Todo) -> bool {
        match self {
            Visibility::All => true,
            Visibility::Active => !todo.completed,
            Visibility::Completed => todo.completed,
        }
    }
}

#[derive(Debug)]
pub struct Footer {
    pub completed_count: usize,
    pub active_count: usize,
    pub todos: Key<Vec<Todo>>,
    pub visibility: Key<Visibility>,
}

impl Component for Footer {
    fn contents(self) {
        let Self {
            completed_count,
            active_count,
            todos,
            visibility,
        } = self;

        show!(element("footer").attr("class", "footer").inner(|| {
            show!(
                element("span")
                    .attr("class", "todo-count")
                    // lol this is awful
                    .child(element("strong").child(if active_count == 0 {
                        text!("No")
                    } else {
                        text!(active_count)
                    }))
                    .child(text!(
                        " {} left",
                        if active_count == 1 { "item" } else { "items" }
                    )),
                element("ul")
                    .attr("class", "filters")
                    .child(text!("TODO: filters"))
            );

            if completed_count > 0 {
                show!(element("button")
                    .attr("class", "clear-completed")
                    .on(
                        |_: ClickEvent, todos| {
                            Some(todos.iter().filter(|t| !t.completed).cloned().collect())
                        },
                        todos
                    )
                    .child(text!("Clear completed")));
            }
        }));
    }
}
