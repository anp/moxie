use moxie_dom::{elements::input, prelude::*};
use wasm_bindgen::JsCast;

#[topo::nested]
pub fn text_input(placeholder: &str, editing: bool, mut on_save: impl FnMut(String) + 'static) {
    let text = state(|| if editing { placeholder.to_string() } else { String::new() });

    fn input_value(ev: impl AsRef<sys::Event>) -> String {
        let event: &sys::Event = ev.as_ref();
        let target = event.target().unwrap();
        let input: sys::HtmlInputElement = target.dyn_into().unwrap();
        let val = input.value();
        input.set_value(""); // it's a little weird to clear the text every time, TODO clean up
        val
    }

    let change_text = text.clone();
    let on_change = move |change: event::Change| change_text.set(input_value(change));

    let clear_text = text.clone();
    let on_keydown = move |keypress: event::KeyDown| {
        if keypress.key() == "Enter" {
            let value = input_value(keypress);
            if !value.is_empty() {
                on_save(value);
            }
            clear_text.set("".into());
        }
    };

    mox! {
        <input type="text" placeholder={placeholder} value={&*text} autoFocus="true"
            class={if editing { "edit new-todo" } else { "new-todo"}}
            on={on_change} on={on_keydown} />
    };
}
