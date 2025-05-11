use serde::{Deserialize, Serialize};

use crate::{
    components::{
        add::table::AddTable, max_reduce::table::MaxReduceTable, mul::table::MulTable,
        recip::table::RecipTable, sin::table::SinTable, sum_reduce::table::SumReduceTable,
        ClaimType,
    },
    utils::AtomicMultiplicityColumn,
};

/// Represents an operator's trace table along with its claim before conversion
/// to a serialized trace format. Used to defer trace evaluation until proving.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TableTrace {
    Add { table: AddTable },
    Mul { table: MulTable },
    Recip { table: RecipTable },
    Sin { table: SinTable },
    SumReduce { table: SumReduceTable },
    MaxReduce { table: MaxReduceTable },
}

impl TableTrace {
    pub fn from_add(table: AddTable) -> Self {
        Self::Add { table }
    }
    pub fn from_mul(table: MulTable) -> Self {
        Self::Mul { table }
    }
    pub fn from_recip(table: RecipTable) -> Self {
        Self::Recip { table }
    }
    pub fn from_sin(table: SinTable) -> Self {
        Self::Sin { table }
    }
    pub fn from_sum_reduce(table: SumReduceTable) -> Self {
        Self::SumReduce { table }
    }
    pub fn from_max_reduce(table: MaxReduceTable) -> Self {
        Self::MaxReduce { table }
    }
}

/// Container for traces and execution resources of a computational graph.
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairPie {
    pub table_traces: Vec<TableTrace>,
    pub execution_resources: ExecutionResources,
}

/// Struct for all LUT multiplicities
#[derive(Serialize, Deserialize, Debug)]
pub struct LUTMultiplicities {
    pub sin: AtomicMultiplicityColumn,
}

/// Represents a single trace with its evaluation, claim, and node information.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Trace {
    pub claim: ClaimType,
}

impl Trace {
    pub fn new(claim: ClaimType) -> Self {
        Self { claim }
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
    pub add: usize,
    pub mul: usize,
    pub recip: usize,
    pub sin: usize,
    pub sum_reduce: usize,
    pub max_reduce: usize,
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
