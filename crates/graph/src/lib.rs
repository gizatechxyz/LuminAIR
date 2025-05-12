pub mod data;
pub mod graph;
pub mod op;
pub mod settings;
pub mod utils;

#[cfg(test)]
mod tests;

/// Type alias for the Stwo compiler used in LuminAIR.
///
/// This compiler transforms graph operations into a provable form ready to be used by Stwo prover.
pub type StwoCompiler = (op::prim::PrimitiveCompiler, op::other::CopyCompiler);
