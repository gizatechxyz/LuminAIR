pub mod data;
pub mod graph;
pub mod op;
pub mod utils;

#[cfg(test)]
mod tests;

/// Type alias for the STWO compiler used in LuminAIR
pub type StwoCompiler = (op::prim::PrimitiveCompiler, op::other::CopyCompiler);
