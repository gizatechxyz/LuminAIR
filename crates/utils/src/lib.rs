use stwo::core::prover::{ProvingError, VerificationError};
use thiserror::Error;

/// Main error type for LuminAIR operations
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

/// Errors related to trace generation and processing
#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum TraceError {
    #[error("The trace is empty.")]
    EmptyTrace,
}
