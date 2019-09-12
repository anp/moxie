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
    tracing::info!({ id = ?topo::Id::current(), ?to_set }, "filter_link");
    let visibility = topo::Env::expect::<Key<Visibility>>().clone();

    element!("li", |e| e.inner(|| {
        tracing::info!({ id = ?topo::Id::current() }, "inside li");
        element!("a", |link| {
            link.attr("style", "cursor: pointer;");
            if *visibility == to_set {
                link.attr("class", "selected");
            }

            link.on(move |_: ClickEvent, _| Some(to_set), visibility)
                .inner(|| text!(to_set));
        });
    }));
}

#[topo::aware]
pub fn filter() {
    tracing::info!({ id = ?topo::Id::current() }, "filter");
    element!("ul", |e| e.attr("class", "filters").inner(|| {
        tracing::info!({ id = ?topo::Id::current() }, "list inner");
        for &to_set in &[All, Active, Completed] {
            tracing::info!({ id = ?topo::Id::current(), ?to_set }, "filter loop");
            filter_link!(to_set)
        }
    }));
}
