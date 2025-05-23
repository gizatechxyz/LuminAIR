use luminair_air::settings::CircuitSettings;
use luminair_prover::LuminairProof;
use luminair_verifier::verifier::verify;
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use wasm_bindgen::prelude::*;

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
pub fn verify_proof_wasm(proof_bytes: &[u8], settings_bytes: &[u8]) -> VerificationResult {
    console_info("Starting WASM proof verification from binary...");

    // Parse the proof from bincode
    let proof: LuminairProof<Blake2sMerkleHasher> = match bincode::deserialize(proof_bytes) {
        Ok(proof) => proof,
        Err(e) => {
            let error_msg = format!("Failed to parse proof binary: {}", e);
            console_error(&error_msg);
            return VerificationResult {
                success: false,
                error_message: Some(error_msg),
            };
        }
    };

    // Parse the settings from bincode
    let settings: CircuitSettings = match bincode::deserialize(settings_bytes) {
        Ok(settings) => settings,
        Err(e) => {
            let error_msg = format!("Failed to parse settings binary: {}", e);
            console_error(&error_msg);
            return VerificationResult {
                success: false,
                error_message: Some(error_msg),
            };
        }
    };

    // Perform verification
    match verify(proof, settings) {
        Ok(()) => {
            console_info("Proof verification successful! ✅");
            VerificationResult {
                success: true,
                error_message: None,
            }
        }
        Err(e) => {
            let error_msg = format!("Proof verification failed: {}", e);
            console_error(&error_msg);
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
