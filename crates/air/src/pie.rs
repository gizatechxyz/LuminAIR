use serde::{Deserialize, Serialize};

use crate::{
    components::{
        add::table::AddTable, lookups::sin::table::SinLookupTable,
        max_reduce::table::MaxReduceTable, mul::table::MulTable, recip::table::RecipTable,
        sin::table::SinTable, sum_reduce::table::SumReduceTable, ClaimType, TraceError, TraceEval,
    },
    utils::AtomicMultiplicityColumn,
};

/// Represents an operator's trace table along with its claim before conversion
/// to a serialized trace format. Used to defer trace evaluation until proving.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum TableTrace {
    /// Addition operator trace table.
    Add { table: AddTable },
    /// Multiplication operator trace table.
    Mul { table: MulTable },
    /// Sum Reduce operator trace table.
    SumReduce { table: SumReduceTable },
    /// Recip operator trace table.
    Recip { table: RecipTable },
    /// Max Reduce operator trace table.
    MaxReduce { table: MaxReduceTable },
    /// Sin operator trace table.
    Sin { table: SinTable },
    /// Sin LUT trace table.
    SinLookup { table: SinLookupTable },
}

impl TableTrace {
    /// Creates a new [`TableTrace`] from an [`AddTable`]
    /// for use in the proof generation.
    pub fn from_add(table: AddTable) -> Self {
        Self::Add { table }
    }

    /// Creates a new [`TableTrace`] from a [`MulTable`]
    /// for use in the proof generation.
    pub fn from_mul(table: MulTable) -> Self {
        Self::Mul { table }
    }

    /// Creates a new [`TableTrace`] from a [`RecipTable`]
    /// for use in the proof generation.
    pub fn from_recip(table: RecipTable) -> Self {
        Self::Recip { table }
    }

    /// Creates a new [`TableTrace`] from a [`SumReduceTable`]
    /// for use in the proof generation.
    pub fn from_sum_reduce(table: SumReduceTable) -> Self {
        Self::SumReduce { table }
    }

    /// Creates a new [`TableTrace`] from a [`MaxReduceTable`]
    /// for use in the proof generation.
    pub fn from_max_reduce(table: MaxReduceTable) -> Self {
        Self::MaxReduce { table }
    }

    /// Creates a new [`TableTrace`] from a [`SinTable`]
    /// for use in the proof generation.
    pub fn from_sin(table: SinTable) -> Self {
        Self::Sin { table }
    }

    /// Creates a new [`TableTrace`] from a [`SinLookupTable`]
    /// for use in the proof generation.
    pub fn from_sin_lookup(table: SinLookupTable) -> Self {
        Self::SinLookup { table }
    }

    pub fn to_trace(&self) -> Result<(TraceEval, ClaimType), TraceError> {
        match self {
            TableTrace::Add { table } => {
                let (trace, claim) = table.trace_evaluation()?;
                Ok((trace, ClaimType::Add(claim)))
            }

            TableTrace::Mul { table } => {
                let (trace, claim) = table.trace_evaluation()?;
                Ok((trace, ClaimType::Mul(claim)))
            }

            TableTrace::SumReduce { table } => {
                let (trace, claim) = table.trace_evaluation()?;
                Ok((trace, ClaimType::SumReduce(claim)))
            }

            TableTrace::Recip { table } => {
                let (trace, claim) = table.trace_evaluation()?;
                Ok((trace, ClaimType::Recip(claim)))
            }

            TableTrace::MaxReduce { table } => {
                let (trace, claim) = table.trace_evaluation()?;
                Ok((trace, ClaimType::MaxReduce(claim)))
            }

            TableTrace::Sin { table } => {
                let (trace, claim) = table.trace_evaluation()?;
                Ok((trace, ClaimType::Sin(claim)))
            }

            TableTrace::SinLookup { table } => {
                let (trace, claim) = table.trace_evaluation()?;
                Ok((trace, ClaimType::SinLookup(claim)))
            }
        }
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
    pub sum_reduce: usize,
    pub recip: usize,
    pub max_reduce: usize,
    pub sin: usize,
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
