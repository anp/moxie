use moxie_dom::{elements::html::*, prelude::*};
use wasm_bindgen::prelude::*;

/// The counter_fn example, but using the DOM builder API.
#[wasm_bindgen]
pub fn boot(root: moxie_dom::raw::sys::Node) {
    moxie_dom::boot(root, || {
        let (count, incrementer) = state(|| 0);
        let decrementer = incrementer.clone();

        div()
            .child(button().onclick(move |_| decrementer.mutate(|count| *count -= 1)).child("-"))
            .child(count)
            .child(button().onclick(move |_| incrementer.mutate(|count| *count += 1)).child("+"))
    });
}
