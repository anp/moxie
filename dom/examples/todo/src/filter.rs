use {
    crate::Todo,
    moxie_dom::{element, prelude::*, text},
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

#[topo::aware]
pub fn filter_link(to_set: Visibility) {
    let visibility = topo::Env::expect::<Key<Visibility>>();

    let mut link = element("a").attr("style", "cursor: pointer;");

    if *key == to_set {
        link = link.attr("class", "selected");
    }

    element!("li").inner(|| {
        link.on(move |_: ClickEvent, _| Some(to_set), key)
            .child(text!(to_set.to_string()))
    });
}

#[topo::aware]
pub fn filter() {
    let visibility = topo::Env::expect::<Key<Visibility>>();
    element!("ul").attr("class", "filters").inner(|| {
        for &to_set in [All, Active, Completed].iter() {
            filter_link!(to_set)
        }
    });
}
