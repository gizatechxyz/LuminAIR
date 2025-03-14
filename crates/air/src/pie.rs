use serde::{Deserialize, Serialize};

use crate::{components::ClaimType, serde::SerializableTrace};

/// Container for traces and execution resources of a computational graph.
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairPie {
    pub traces: Vec<Trace>,
    pub execution_resources: ExecutionResources,
}

/// Represents a single trace with its evaluation, claim, and node information.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Trace {
    pub eval: SerializableTrace,
    pub claim: ClaimType,
}

impl Trace {
    pub fn new(eval: SerializableTrace, claim: ClaimType) -> Self {
        Self { eval, claim }
    }
}

/// Holds resource usage data for the execution.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionResources {
    pub op_counter: OpCounter,
    pub max_log_size: u32,
}

/// Counts occurrences of specific operations.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OpCounter {
    pub add: Option<usize>,
    pub mul: Option<usize>,
    pub lessthan: Option<usize>,
}

/// Indicates if a node input is an initializer (i.e., from initial input).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputInfo {
    pub is_initializer: bool,
    pub id: u32,
}

/// Indicates if a node output is a final graph output or intermediate.
#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct OutputInfo {
    pub is_final_output: bool,
}

/// Contains input, output, and consumer information for a node.
#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct NodeInfo {
    pub inputs: Vec<InputInfo>,
    pub output: OutputInfo,
    pub num_consumers: u32,
    pub id: u32,
}
