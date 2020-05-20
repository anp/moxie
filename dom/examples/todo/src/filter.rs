use crate::Todo;
use moxie_dom::{
    elements::{
        html::*,
        text_content::{Li, Ul},
    },
    prelude::*,
};
use Visibility::{Active, All, Completed};

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

#[topo::nested]
#[illicit::from_env(visibility: &Key<Visibility>)]
pub fn filter_link(to_set: Visibility) -> Li {
    let visibility = visibility.clone();
    mox! {
        <li>
            <a style="cursor: pointer;"
             class={if *visibility == to_set { "selected" } else { "" }}
             onclick={move |_| visibility.set(to_set)}>
                {% "{}", to_set }
            </a>
        </li>
    }
}

#[topo::nested]
pub fn filter() -> Ul {
    let mut list = ul();
    list = list.class("filters");
    for &to_set in &[All, Active, Completed] {
        list = list.child(filter_link(to_set));
    }

    list.build()
}
