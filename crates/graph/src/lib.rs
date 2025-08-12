pub mod data;
pub mod graph;
pub mod op;
pub mod utils;

#[cfg(test)]
mod tests;

pub type StwoCompiler = (op::prim::PrimitiveCompiler, op::other::CopyCompiler);
