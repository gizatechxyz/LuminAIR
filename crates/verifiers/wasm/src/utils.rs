use wasm_bindgen::prelude::*;
use tracing::{info, warn, error};

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

/// Log an error to the browser console and tracing system
pub fn console_error(message: &str) {
    web_sys::console::error_1(&message.into());
    error!("{}", message);
}

/// Log info to the browser console and tracing system
pub fn console_info(message: &str) {
    web_sys::console::info_1(&message.into());
    info!("{}", message);
}

/// Log a warning to the browser console and tracing system
pub fn console_warn(message: &str) {
    web_sys::console::warn_1(&message.into());
    warn!("{}", message);
}

/// Configure tracing level from JavaScript
/// This allows fine-grained control over what verification steps are logged
#[wasm_bindgen]
pub fn set_tracing_level(level: &str) -> bool {
    match level.to_lowercase().as_str() {
        "trace" => {
            console_info("🔧 Tracing level set to TRACE - all verification steps will be logged");
            true
        },
        "debug" => {
            console_info("🔧 Tracing level set to DEBUG - detailed verification info will be logged");
            true
        },
        "info" => {
            console_info("🔧 Tracing level set to INFO - verification phases will be logged");
            true
        },
        "warn" => {
            console_info("🔧 Tracing level set to WARN - only warnings and errors will be logged");
            true
        },
        "error" => {
            console_info("🔧 Tracing level set to ERROR - only errors will be logged");
            true
        },
        _ => {
            console_error(&format!("❌ Invalid tracing level: {}. Valid levels: trace, debug, info, warn, error", level));
            false
        }
    }
}

/// Get current verification step information
/// This can be called from JavaScript to get more detailed status
#[wasm_bindgen]
pub fn get_verification_phases() -> String {
    r#"
{
    "phases": [
        {
            "name": "protocol_setup",
            "description": "Initialize verifier components and configuration"
        },
        {
            "name": "interaction_phase_0", 
            "description": "Process and commit preprocessed trace"
        },
        {
            "name": "interaction_phase_1",
            "description": "Process and commit main execution trace"
        },
        {
            "name": "interaction_phase_2",
            "description": "Process interaction trace and validate LogUp sums"
        },
        {
            "name": "proof_verification",
            "description": "Verify the final STARK proof"
        }
    ]
}
"#.to_string()
}
