mod api;
mod global;
mod shared_memory;
mod sound_engine;
mod utils;

use console_error_panic_hook;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub extern "C" fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
