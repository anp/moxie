use {
    moxie_dom::{element, prelude::*, sys},
    wasm_bindgen::JsCast,
};

#[topo::aware]
pub fn text_input(placeholder: &str, editing: bool, mut on_save: impl FnMut(String) + 'static) {
    let text = state!(|| if editing {
        placeholder.to_string()
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

    let change_text = text.clone();

    element!("input", |e| {
        e.attr("autoFocus", "true")
            .attr("class", "new-todo")
            .attr("placeholder", placeholder)
            .attr("type", "text")
            .attr("value", &*text)
            .on(move |change: ChangeEvent| change_text.set(input_value(change)))
            .on(move |keypress: KeyDownEvent| {
                if keypress.key() == "Enter" {
                    let value = input_value(keypress);
                    if !value.is_empty() {
                        on_save(value);
                    }
                    text.set("".into());
                }
            });

        if editing {
            e.attr("class", "edit");
        }
    });
}
