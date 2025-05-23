mod utils;
mod verifier;

pub use verifier::*;
pub use utils::{set_tracing_level, get_verification_phases};

use wasm_bindgen::prelude::*;

// When the `wee_alloc_feature` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc_feature")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize the WASM module
/// This sets up panic hooks and logging for better debugging experience
#[wasm_bindgen(start)]
pub fn main() {
    utils::set_panic_hook();
    
    // Initialize tracing for WASM
    tracing_wasm::set_as_global_default();
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Console log macro for debugging
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}