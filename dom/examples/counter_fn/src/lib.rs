use mox::mox;
use moxie_dom::{elements::html::*, prelude::*};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn boot(root: moxie_dom::raw::sys::Node) {
    moxie_dom::boot(root, || {
        let (count, incrementer) = state(|| 0);
        let decrementer = incrementer.clone();
        mox! {
            <div>
                <button onclick={move |_| decrementer.mutate(|count| *count -= 1)}>"-"</button>
                { count }
                <button onclick={move |_| incrementer.mutate(|count| *count += 1)}>"+"</button>
            </div>
        }
    });
}
