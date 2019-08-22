use {
    crate::Todo,
    moxie_dom::prelude::*,
    Visibility::{Active, All, Completed},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    All,
    Active,
    Completed,
}

impl Default for Visibility {
    fn default() -> Self {
        All
    }
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            All => "All",
            Active => "Active",
            Completed => "Completed",
        })
    }
}

impl Visibility {
    pub fn should_show(self, todo: &Todo) -> bool {
        match self {
            All => true,
            Active => !todo.completed,
            Completed => todo.completed,
        }
    }
}

#[derive(Debug, Default)]
pub struct Filter;

impl Component for Filter {
    fn contents(self) {
        let visibility = topo::Env::expect::<Key<Visibility>>();
        show!(element("ul").attr("class", "filters").inner(|| {
            for &to_set in [All, Active, Completed].iter() {
                show!(FilterLink {
                    to_set,
                    key: visibility.clone(),
                });
            }
        }));
    }
}

#[derive(Debug)]
struct FilterLink {
    key: Key<Visibility>,
    to_set: Visibility,
}

impl Component for FilterLink {
    fn contents(self) {
        let Self { to_set, key } = self;

        let mut link = element("a").attr("style", "cursor: pointer;");

        if *key == to_set {
            link = link.attr("class", "selected");
        }

        show!(element("li").child(
            link.on(move |_: ClickEvent, _| Some(to_set), key)
                .child(text!(to_set.to_string()))
        ));
    }
}
