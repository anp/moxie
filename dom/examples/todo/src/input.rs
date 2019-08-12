use {
    moxie_dom::{prelude::*, sys},
    wasm_bindgen::JsCast,
};

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
                        let value = input_value(keypress);
                        if !value.is_empty() {
                            on_save(value);
                        }
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
