use std::path::Path;

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

/// Trait for JSON serialization
pub trait JsonSerialization {
    /// Serialize the object to a JSON string
    fn to_json(&self) -> Result<String, LuminairError>;

    /// Serialize the object to a JSON file
    fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LuminairError>;
}

/// Trait for JSON deserialization
pub trait JsonDeserialization {
    /// Deserialize the object from a JSON string
    fn from_json(json: &str) -> Result<Self, LuminairError>
    where
        Self: Sized;

    /// Deserialize the object from a JSON file
    fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, LuminairError>
    where
        Self: Sized;
}
