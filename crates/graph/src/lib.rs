pub mod data;
pub mod graph;
pub mod op;
pub mod utils;

#[cfg(test)]
mod tests;

/// Type alias for the Stwo compiler used in LuminAIR.
///
/// Represents the collection of compilers needed to transform a computation graph
/// defined in LuminAIR into an AIR format compatible with the STWO prover.
/// It bundles primitive operations and copy constraints compilers.
pub type StwoCompiler = (op::prim::PrimitiveCompiler, op::other::CopyCompiler);
