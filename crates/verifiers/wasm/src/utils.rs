use wasm_bindgen::prelude::*;

/// Set up better panic messages for debugging in the browser
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// A wrapper for `console.log` for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Log an error to the browser console
pub fn console_error(message: &str) {
    web_sys::console::error_1(&message.into());
}

/// Log info to the browser console
pub fn console_info(message: &str) {
    web_sys::console::info_1(&message.into());
}
