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
    fn contents(self) {
        let todos_empty = self.todos.is_empty();
        show!(element("section").attr("class", "main").inner(move || {
            // const { todos } = useStore(TodoStore);
            // const todosCount = todos.length;
            // const completedCount = getCompletedCount(todos);

            if todos_empty {

                // <span>
                //   <input
                //     className="toggle-all"
                //     type="checkbox"
                //     defaultChecked={completedCount === todosCount}
                //   />
                //   <label onClick={completeAllTodos} />
                // </span>
            }

            // show!(TodoList::new());

            if todos_empty {
                // <Footer
                //   completedCount={completedCount}
                //   activeCount={todosCount - completedCount}
                //   onClearCompleted={clearCompletedTodos}
                // />
            }
        }));
    }
}
