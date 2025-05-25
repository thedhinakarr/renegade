use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    web_sys::console::log_1(&"Hello from Renegade WASM!".into());
}
