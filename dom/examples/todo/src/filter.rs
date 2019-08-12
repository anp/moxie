use {crate::Todo, moxie_dom::prelude::*};

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
