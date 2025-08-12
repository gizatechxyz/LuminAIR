//! LuminAIR prelude module providing convenient re-exports of commonly used components
//! 
//! This module consolidates imports from various LuminAIR crates to provide
//! a single entry point for users of the framework.

// --- luminal ---
/// Re-exports from the luminal deep learning library
pub use luminal::prelude::*;
/// Re-exports from luminal neural network components
pub use luminal_nn::*;

// --- luminair_graph ---
/// Re-exports the LuminAIR graph trait for computational graph operations
pub use luminair_graph::graph::LuminairGraph;
/// Re-exports the STWO compiler configuration
pub use luminair_graph::StwoCompiler;

// --- luminair_prover ---
/// Re-exports the main proving function
pub use luminair_prover::{prover::prove, LuminairProof};

// --- luminair_verifier ---
/// Re-exports core AIR components and circuit settings
pub use luminair_air::{pie::LuminairPie, settings::CircuitSettings};
/// Re-exports the verification function
pub use luminair_verifier::verifier::verify;
