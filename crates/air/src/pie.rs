use serde::{Deserialize, Serialize};

use crate::components::{
    add::table::AddTable, lessthan::table::LessThanTable, mul::table::MulTable,
    add::table::AddTable, max_reduce::table::MaxReduceTable, mul::table::MulTable,
    recip::table::RecipTable, sum_reduce::table::SumReduceTable, ClaimType, TraceError, TraceEval,
use crate::{
    components::{
        add::table::AddTraceTable, lookups::sin::table::SinLookupTraceTable,
        max_reduce::table::MaxReduceTraceTable, mul::table::MulTraceTable,
        recip::table::RecipTraceTable, sin::table::SinTraceTable,
        sum_reduce::table::SumReduceTraceTable,
    },
    utils::AtomicMultiplicityColumn,
};

/// Enum wrapping the trace table generated for a specific AIR component.
///
/// This allows collecting raw trace data from different operations (Add, Mul, Sin, etc.)
/// produced during the graph execution (`gen_trace` phase) into a heterogeneous list (`Vec<TraceTable>`)
/// before being processed by the prover.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TableTrace {
    /// Addition operator trace table.
    Add { table: AddTable },
    /// Multiplication operator trace table.
    Mul { table: MulTable },
    /// LessThan operator trace table.
    LessThan { table: LessThanTable },
    /// Sum Reduce operator trace table.
    SumReduce { table: SumReduceTable },
    /// Recip operator trace table.
    Recip { table: RecipTable },
    /// Max Reduce operator trace table.
    MaxReduce { table: MaxReduceTable },
pub enum TraceTable {
    /// Trace table for Add operations.
    Add { table: AddTraceTable },
    /// Trace table for Mul operations.
    Mul { table: MulTraceTable },
    /// Trace table for Recip operations.
    Recip { table: RecipTraceTable },
    /// Trace table for Sin operations.
    Sin { table: SinTraceTable },
    /// Trace table for Sin lookup operations.
    SinLookup { table: SinLookupTraceTable },
    /// Trace table for SumReduce operations.
    SumReduce { table: SumReduceTraceTable },
    /// Trace table for MaxReduce operations.
    MaxReduce { table: MaxReduceTraceTable },
}

impl TraceTable {
    /// Creates a `TraceTable::Add` variant.
    pub fn from_add(table: AddTraceTable) -> Self {
        Self::Add { table }
    }
    /// Creates a `TraceTable::Mul` variant.
    pub fn from_mul(table: MulTraceTable) -> Self {
        Self::Mul { table }
    }

    /// Creates a new [`TableTrace`] from a [`LessThanTable`]
    /// for use in the proof generation.
    pub fn from_lessthan(table: LessThanTable) -> Self {
        Self::LessThan { table }
    }

    /// Creates a new [`TableTrace`] from a [`RecipTable`]
    /// for use in the proof generation.
    pub fn from_recip(table: RecipTable) -> Self {
    /// Creates a `TraceTable::Recip` variant.
    pub fn from_recip(table: RecipTraceTable) -> Self {
        Self::Recip { table }
    }
    /// Creates a `TraceTable::Sin` variant.
    pub fn from_sin(table: SinTraceTable) -> Self {
        Self::Sin { table }
    }
    /// Creates a `TraceTable::SinLookup` variant.
    pub fn from_sin_lookup(table: SinLookupTraceTable) -> Self {
        Self::SinLookup { table }
    }
    /// Creates a `TraceTable::SumReduce` variant.
    pub fn from_sum_reduce(table: SumReduceTraceTable) -> Self {
        Self::SumReduce { table }
    }
    /// Creates a `TraceTable::MaxReduce` variant.
    pub fn from_max_reduce(table: MaxReduceTraceTable) -> Self {
        Self::MaxReduce { table }
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

            TableTrace::LessThan { table } => {
                let (trace, claim) = table.trace_evaluation()?;
                Ok((trace, ClaimType::LessThan(claim)))
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
        }
    }
}

/// Primary container for the PIE generated during trace execution.
///
/// This structure holds all the computed trace data (`trace_tables`) and essential metadata
/// (`execution_resources`) required by the STWO prover to generate a STARK proof.
/// It is the output of the `LuminairGraph::gen_trace` method.
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairPie {
    /// A collection of trace tables, one entry for each AIR component instance used.
    pub trace_tables: Vec<TraceTable>,
    /// Metadata about the execution, such as trace dimensions and operation counts.
    pub execution_resources: ExecutionResources,
}

/// Struct for all LUT multiplicities
#[derive(Serialize, Deserialize, Debug)]
pub struct LUTMultiplicities {
    pub sin: AtomicMultiplicityColumn,
}



/// Holds resource usage metadata gathered during graph execution.
///
/// This includes the maximum trace log-size required across all components
/// and counts of different operation types.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionResources {
    /// Counts of each AIR component operation executed.
    pub op_counter: OpCounter,
    /// The maximum log2 size needed for any trace segment (determines STARK domain size).
    pub max_log_size: u32,
}

/// Counts the occurrences of each specific AIR operation type during graph execution.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OpCounter {
    pub add: Option<usize>,
    pub mul: Option<usize>,
    pub lessthan: Option<usize>,
    pub sum_reduce: Option<usize>,
    pub recip: Option<usize>,
    pub max_reduce: Option<usize>,
    /// Number of Add operations.
    pub add: usize,
    /// Number of Mul operations.
    pub mul: usize,
    /// Number of Recip operations.
    pub recip: usize,
    /// Number of Sin operations.
    pub sin: usize,
    /// Number of SumReduce operations.
    pub sum_reduce: usize,
    /// Number of MaxReduce operations.
    pub max_reduce: usize,
}

/// Metadata about a specific input to a graph node.
/// Indicates if a node input is an initializer (i.e., from initial input).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputInfo {
    /// True if the input originates from a graph input or a constant (not an intermediate value).
    pub is_initializer: bool,
    /// The unique ID of the node providing this input.
    pub id: u32,
}

/// Metadata about the output of a graph node.
/// Indicates if a node output is a final graph output or intermediate.
#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct OutputInfo {
    /// True if this node's output is marked as a final output of the computation graph.
    pub is_final_output: bool,
}

/// Comprehensive metadata about a node in the computation graph during trace generation.
///
/// Passed to `LuminairOperator::process_trace` to provide context for building trace rows,
/// particularly for calculating multiplicities in interaction arguments.
#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct NodeInfo {
    /// Information about each input connection to the node.
    pub inputs: Vec<InputInfo>,
    /// Information about the node's output.
    pub output: OutputInfo,
    /// The number of nodes that consume the output of this node.
    pub num_consumers: u32,
    /// The unique ID of this node.
    pub id: u32,
}
