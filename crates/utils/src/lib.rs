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

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Errors that can occur during AIR trace generation or processing.
#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum TraceError {
    /// Indicates that a component trace was unexpectedly empty.
    #[error("The trace is empty.")]
    EmptyTrace,
}
