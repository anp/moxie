use {
    super::*,
    moxie_dom::{elements::*, events::*, *},
    wasm_bindgen::JsCast,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    todos: Key<Vec<Todo>>,
}

impl Header {
    pub fn new(todos: Key<Vec<Todo>>) -> Self {
        Self { todos }
    }
}

impl Component for Header {
    fn contents(self) {
        show!(element("header")
            .attr("class", "header")
            .child(element("h1").child(text!("todos")))
            .child(TextInput {
                placeholder: "What needs to be done?".into(),
                todos: self.todos,
            }))
    }
}

#[derive(Debug)]
struct TextInput {
    placeholder: String,
    todos: Key<Vec<Todo>>,
}

impl Component for TextInput {
    fn contents(self) {
        // let class_name = if text.is_none() {  } else { "edit" };
        let text = state!((), |()| String::new());
        let text2 = text.clone();

        fn input_value(ev: impl AsRef<sys::Event>) -> String {
            let event: &sys::Event = ev.as_ref();
            let target = event.target().unwrap();
            let input: sys::HtmlInputElement = target.dyn_into().unwrap();
            let val = input.value();
            input.set_value(""); // it's a little weird to clear the text every time, TODO clean up later
            val
        }

        show!(element("input")
            .attr("autoFocus", "true")
            .attr("class", "new-todo")
            .attr("placeholder", self.placeholder)
            .attr("type", "text")
            .attr("value", &*text)
            // TODO find a way to bind ChangeEvent to `on` more eagerly in the syntax here
            .on(text2, |_prev, change: ChangeEvent| Some(input_value(
                change
            )))
            .on(self.todos, move |todos, keypress: KeyDownEvent| {
                if keypress.key() == "Enter" {
                    // first, zero out the persistent state, we're taking the value from the dom node and hoisting it into parent state as a Todo
                    text.update(|_| Some("".into()));

                    let mut todos: Vec<Todo> = todos.to_vec();
                    todos.push(Todo::new(input_value(keypress)));

                    info!({ ?todos }, "updated todos");
                    Some(todos)
                } else {
                    None
                }
            }));
    }
}
