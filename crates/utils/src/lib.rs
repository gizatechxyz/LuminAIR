use luminair_air::components::TraceError;
use stwo_prover::core::prover::{ProvingError, VerificationError};
use thiserror::Error;

/// Errors that can occur during LuminAIR graph processing, proof generation, or verification.
#[derive(Clone, Debug, Error)]
pub enum LuminairError {
    #[error(transparent)]
    TraceError(#[from] TraceError),

    #[error("Main trace generation failed.")]
    MainTraceEvalGenError,

    #[error("Interaction trace generation failed.")]
    InteractionTraceEvalGenError,

    #[error(transparent)]
    ProverError(#[from] ProvingError),

    #[error(transparent)]
    StwoVerifierError(#[from] VerificationError),

    #[error("{0} LogUp values do not match.")]
    InvalidLogUp(String),
}
