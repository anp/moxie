use mox::mox;
use moxie_dom::{
    elements::forms::{input, Input},
    prelude::*,
};
use wasm_bindgen::JsCast;

#[topo::nested]
pub fn text_input(
    placeholder: &str,
    editing: bool,
    mut on_save: impl FnMut(String) + 'static,
) -> Input {
    let (text, set_text) = state(|| if editing { placeholder.to_string() } else { String::new() });
    let clear_text = set_text.clone();

    fn input_value(ev: impl AsRef<sys::Event>) -> String {
        let event: &sys::Event = ev.as_ref();
        let target = event.target().unwrap();
        let input: sys::HtmlInputElement = target.dyn_into().unwrap();
        let val = input.value();
        input.set_value(""); // it's a little weird to clear the text every time, TODO clean up
        val
    }

    mox! {
        <input type="text" placeholder value=&text autofocus=true
            class = if editing { "edit new-todo" } else { "new-todo" }
            onchange = move |change| set_text.set(input_value(change))
            onkeydown = move |keypress| {
                if keypress.key() == "Enter" {
                    let value = input_value(keypress);
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        on_save(trimmed.to_owned());
                    }
                    clear_text.set("".into());
                }
            } />
    }
}
