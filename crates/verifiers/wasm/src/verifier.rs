use luminair_air::settings::CircuitSettings;
use luminair_prover::LuminairProof;
use luminair_verifier::verifier::verify as verify_rust;
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use wasm_bindgen::prelude::*;
use tracing::{info, span, Level};

use crate::utils::{console_error, console_info};

/// WASM-exposed verification result
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct VerificationResult {
    success: bool,
    error_message: Option<String>,
}

#[wasm_bindgen]
impl VerificationResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }
}

/// Verifies a LuminAIR proof in WASM.
///
/// Takes binary data for both `LuminairProof` and `CircuitSettings` and returns a verification result.
/// This is the main entry point for WASM-based proof verification.
#[wasm_bindgen]
pub fn verify(proof_bytes: &[u8], settings_bytes: &[u8]) -> VerificationResult {
    let _span = span!(Level::INFO, "wasm_verification_wrapper").entered();
    console_info("🌟 Starting WASM proof verification from binary...");
    info!("🌟 LuminAIR WASM Verifier: Beginning verification process");

    // Parse the proof from bincode
    let proof: LuminairProof<Blake2sMerkleHasher> = match bincode::deserialize(proof_bytes) {
        Ok(proof) => {
            console_info("✅ Successfully parsed proof binary");
            info!("📦 Proof parsing: Success");
            proof
        },
        Err(e) => {
            let error_msg = format!("Failed to parse proof binary: {}", e);
            console_error(&error_msg);
            info!("❌ Proof parsing: Failed - {}", e);
            return VerificationResult {
                success: false,
                error_message: Some(error_msg),
            };
        }
    };

    // Parse the settings from bincode
    let settings: CircuitSettings = match bincode::deserialize(settings_bytes) {
        Ok(settings) => {
            console_info("✅ Successfully parsed settings binary");
            info!("⚙️  Settings parsing: Success");
            settings
        },
        Err(e) => {
            let error_msg = format!("Failed to parse settings binary: {}", e);
            console_error(&error_msg);
            info!("❌ Settings parsing: Failed - {}", e);
            return VerificationResult {
                success: false,
                error_message: Some(error_msg),
            };
        }
    };

    console_info("🔍 Delegating to Rust verifier with detailed tracing...");
    info!("🔍 Delegating to Rust verifier core");

    // Perform verification
    match verify_rust(proof, settings) {
        Ok(()) => {
            console_info("🎉 Proof verification successful! ✅");
            info!("🎉 LuminAIR WASM Verifier: Verification completed successfully");
            VerificationResult {
                success: true,
                error_message: None,
            }
        }
        Err(e) => {
            let error_msg = format!("Proof verification failed: {}", e);
            console_error(&error_msg);
            info!("💥 LuminAIR WASM Verifier: Verification failed - {}", e);
            VerificationResult {
                success: false,
                error_message: Some(error_msg),
            }
        }
    }
}

/// Utility function to check if the WASM module is working correctly
#[wasm_bindgen]
pub fn test_wasm_module() -> String {
    console_info("WASM module is working correctly!");
    "LuminAIR WASM Verifier loaded successfully!".to_string()
}

/// Get version information
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
