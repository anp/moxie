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
        let Self { todos } = self;

        let on_save: Box<dyn Fn(String)> = Box::new(move |value: String| {
            todos.update(|prev| {
                let mut todos: Vec<Todo> = prev.to_vec();
                todos.push(Todo::new(value));
                info!({ ?todos }, "added new todo");
                Some(todos)
            });
        });

        show!(element("header")
            .attr("class", "header")
            .child(element("h1").child(text!("todos")))
            .child(TextInput {
                placeholder: "What needs to be done?".into(),
                editing: false,
                on_save
            }))
    }
}

pub struct TextInput<F> {
    pub placeholder: String,
    pub editing: bool,
    pub on_save: F,
}

// need to avoid putting a debug constraint around on_save
impl<F> std::fmt::Debug for TextInput<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("TextInput")
            .field("placeholder", &self.placeholder)
            .finish()
    }
}

impl<F> Component for TextInput<F>
where
    F: Fn(String) + 'static,
{
    fn contents(self) {
        let Self {
            editing,
            on_save,
            placeholder,
        } = self;

        let text = state!(|| if editing {
            placeholder.clone()
        } else {
            String::new()
        });

        fn input_value(ev: impl AsRef<sys::Event>) -> String {
            let event: &sys::Event = ev.as_ref();
            let target = event.target().unwrap();
            let input: sys::HtmlInputElement = target.dyn_into().unwrap();
            let val = input.value();
            input.set_value(""); // it's a little weird to clear the text every time, TODO clean up
            val
        }

        let mut elem = element("input")
            .attr("autoFocus", "true")
            .attr("class", "new-todo")
            .attr("placeholder", placeholder)
            .attr("type", "text")
            .attr("value", &*text)
            .on(
                |change: ChangeEvent, _| Some(input_value(change)),
                text.clone(),
            )
            .on(
                move |keypress: KeyDownEvent, _| {
                    if keypress.key() == "Enter" {
                        on_save(input_value(keypress));
                        Some("".into())
                    } else {
                        None
                    }
                },
                text,
            );

        if editing {
            elem = elem.attr("class", "edit");
        }

        show!(elem);
    }
}
