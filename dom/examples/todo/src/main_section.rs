use crate::*;

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
    fn contents(self) {}
}
