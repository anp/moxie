use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn bare();
}
// @@ end-expected @@ //

// TODO write tests that make sure we can use the function?
