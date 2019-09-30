use {
    crate::Todo,
    moxie_dom::*,
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
#[topo::from_env(visibility: Key<Visibility>)]
pub fn filter_link(to_set: Visibility) {
    let visibility = visibility.clone();
    mox! {
        <li>
            <a style="cursor: pointer;"
             class={if *visibility == to_set { "selected" } else { "" }}
             on={move |_: event::Click| visibility.set(to_set)}>
                {% "{}", to_set }
            </a>
        </li>
    }
}

#[topo::aware]
pub fn filter() {
    mox! {
        <ul class="filters">
        {
            for &to_set in &[All, Active, Completed] {
                mox! { <filter_link _=(to_set) /> };
            }
        }
        </ul>
    };
}
