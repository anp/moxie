use tracing::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn setup_tracing() {
    tracing_wasm::set_as_global_default();
    console_error_panic_hook::set_once();
    info!("tracing initialized");
}
