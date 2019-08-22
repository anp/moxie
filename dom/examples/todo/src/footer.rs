use {
    crate::{filter::Filter, Todo},
    moxie_dom::prelude::*,
};

#[derive(Debug)]
pub struct Footer {
    pub num_complete: usize,
    pub num_active: usize,
}

impl Component for Footer {
    fn contents(self) {
        let Self {
            num_complete,
            num_active,
        } = self;
        let todos = topo::Env::expect::<Key<Vec<Todo>>>();

        show!(element("footer").attr("class", "footer").inner(|| {
            show!(
                element("span")
                    .attr("class", "todo-count")
                    // lol this is awful
                    .child(element("strong").child(if num_active == 0 {
                        text!("No")
                    } else {
                        text!(num_active)
                    }))
                    .child(text!(
                        " {} left",
                        if num_active == 1 { "item" } else { "items" }
                    )),
                Filter
            );

            if num_complete > 0 {
                show!(element("button")
                    .attr("class", "clear-completed")
                    .on(
                        |_: ClickEvent, todos| {
                            Some(todos.iter().filter(|t| !t.completed).cloned().collect())
                        },
                        todos.clone()
                    )
                    .child(text!("Clear completed")));
            }
        }));
    }
}
