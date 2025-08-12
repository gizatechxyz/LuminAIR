use serde::{Deserialize, Serialize};

use crate::{
    components::{
        add::table::AddTraceTable,
        contiguous::table::ContiguousTraceTable,
        exp2::table::Exp2TraceTable,
        inputs::table::InputsTraceTable,
        less_than::table::LessThanTraceTable,
        log2::table::Log2TraceTable,
        lookups::{
            exp2::table::Exp2LookupTraceTable, log2::table::Log2LookupTraceTable, range_check::table::RangeCheckLookupTraceTable,
            sin::table::SinLookupTraceTable,
        },
        max_reduce::table::MaxReduceTraceTable,
        mul::table::MulTraceTable,
        recip::table::RecipTraceTable,
        rem::table::RemTraceTable,
        sin::table::SinTraceTable,
        sqrt::table::SqrtTraceTable,
        sum_reduce::table::SumReduceTraceTable,
    },
    utils::AtomicMultiplicityColumn,
};

/// Enumeration of all possible trace table types in LuminAIR
/// 
/// Each variant contains the specific trace table for a particular operation type,
/// allowing unified handling of different trace structures
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TraceTable {
    /// Addition operation trace table
    Add { table: AddTraceTable },
    /// Multiplication operation trace table
    Mul { table: MulTraceTable },
    /// Reciprocal operation trace table
    Recip { table: RecipTraceTable },
    /// Sine operation trace table
    Sin { table: SinTraceTable },
    /// Sine lookup table trace
    SinLookup { table: SinLookupTraceTable },
    /// Sum reduction operation trace table
    SumReduce { table: SumReduceTraceTable },
    /// Maximum reduction operation trace table
    MaxReduce { table: MaxReduceTraceTable },
    /// Square root operation trace table
    Sqrt { table: SqrtTraceTable },
    /// Remainder operation trace table
    Rem { table: RemTraceTable },
    /// Exponential base-2 operation trace table
    Exp2 { table: Exp2TraceTable },
    /// Exponential base-2 lookup table trace
    Exp2Lookup { table: Exp2LookupTraceTable },
    /// Logarithm base-2 operation trace table
    Log2 { table: Log2TraceTable },
    /// Logarithm base-2 lookup table trace
    Log2Lookup { table: Log2LookupTraceTable },
    /// Less-than comparison operation trace table
    LessThan { table: LessThanTraceTable },
    /// Range check lookup table trace
    RangeCheckLookup { table: RangeCheckLookupTraceTable },
    /// Input tensor trace table
    Inputs { table: InputsTraceTable },
    /// Contiguous operation trace table
    Contiguous { table: ContiguousTraceTable },
}

impl TraceTable {
    /// Creates a TraceTable from an AddTraceTable
    pub fn from_add(table: AddTraceTable) -> Self {
        Self::Add { table }
    }
    /// Creates a TraceTable from a MulTraceTable
    pub fn from_mul(table: MulTraceTable) -> Self {
        Self::Mul { table }
    }
    /// Creates a TraceTable from a RecipTraceTable
    pub fn from_recip(table: RecipTraceTable) -> Self {
        Self::Recip { table }
    }
    /// Creates a TraceTable from a SinTraceTable
    pub fn from_sin(table: SinTraceTable) -> Self {
        Self::Sin { table }
    }
    /// Creates a TraceTable from a SinLookupTraceTable
    pub fn from_sin_lookup(table: SinLookupTraceTable) -> Self {
        Self::SinLookup { table }
    }
    /// Creates a TraceTable from a SumReduceTraceTable
    pub fn from_sum_reduce(table: SumReduceTraceTable) -> Self {
        Self::SumReduce { table }
    }
    /// Creates a TraceTable from a MaxReduceTraceTable
    pub fn from_max_reduce(table: MaxReduceTraceTable) -> Self {
        Self::MaxReduce { table }
    }
    /// Creates a TraceTable from a SqrtTraceTable
    pub fn from_sqrt(table: SqrtTraceTable) -> Self {
        Self::Sqrt { table }
    }
    /// Creates a TraceTable from a RemTraceTable
    pub fn from_rem(table: RemTraceTable) -> Self {
        Self::Rem { table }
    }
    /// Creates a TraceTable from an Exp2TraceTable
    pub fn from_exp2(table: Exp2TraceTable) -> Self {
        Self::Exp2 { table }
    }
    /// Creates a TraceTable from an Exp2LookupTraceTable
    pub fn from_exp2_lookup(table: Exp2LookupTraceTable) -> Self {
        Self::Exp2Lookup { table }
    }
    /// Creates a TraceTable from a Log2TraceTable
    pub fn from_log2(table: Log2TraceTable) -> Self {
        Self::Log2 { table }
    }
    /// Creates a TraceTable from a Log2LookupTraceTable
    pub fn from_log2_lookup(table: Log2LookupTraceTable) -> Self {
        Self::Log2Lookup { table }
    }
    /// Creates a TraceTable from a LessThanTraceTable
    pub fn from_less_than(table: LessThanTraceTable) -> Self {
        Self::LessThan { table }
    }
    /// Creates a TraceTable from a RangeCheckLookupTraceTable
    pub fn from_range_check_lookup(table: RangeCheckLookupTraceTable) -> Self {
        Self::RangeCheckLookup { table }
    }
    /// Creates a TraceTable from an InputsTraceTable
    pub fn from_inputs(table: InputsTraceTable) -> Self {
        Self::Inputs { table }
    }
    /// Creates a TraceTable from a ContiguousTraceTable
    pub fn from_contiguous(table: ContiguousTraceTable) -> Self {
        Self::Contiguous { table }
    }
}

/// Main structure containing all trace tables and metadata for a LuminAIR computation
/// 
/// This represents the complete execution trace that will be used for STARK proving
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairPie {
    /// Collection of all trace tables for different operations
    pub trace_tables: Vec<TraceTable>,
    /// Metadata about the computation execution
    pub metadata: Metadata,
}

/// Metadata about the LuminAIR computation execution
#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    /// Resources consumed during execution
    pub execution_resources: ExecutionResources,
}

/// Multiplicities for lookup tables used in the computation
/// 
/// Tracks how many times each lookup table entry is accessed
#[derive(Serialize, Deserialize, Debug)]
pub struct LUTMultiplicities {
    /// Multiplicities for sine lookup table
    pub sin: AtomicMultiplicityColumn,
    /// Multiplicities for exponential base-2 lookup table
    pub exp2: AtomicMultiplicityColumn,
    /// Multiplicities for logarithm base-2 lookup table
    pub log2: AtomicMultiplicityColumn,
    /// Multiplicities for range check lookup table
    pub range_check: AtomicMultiplicityColumn,
}

/// Resources consumed during computation execution
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionResources {
    /// Count of each operation type performed
    pub op_counter: OpCounter,
    /// Maximum log size across all trace tables
    pub max_log_size: u32,
}

/// Counter for different operation types performed during computation
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OpCounter {
    /// Number of addition operations
    pub add: usize,
    /// Number of multiplication operations
    pub mul: usize,
    /// Number of reciprocal operations
    pub recip: usize,
    /// Number of sine operations
    pub sin: usize,
    /// Number of sum reduction operations
    pub sum_reduce: usize,
    /// Number of maximum reduction operations
    pub max_reduce: usize,
    /// Number of square root operations
    pub sqrt: usize,
    /// Number of remainder operations
    pub rem: usize,
    /// Number of exponential base-2 operations
    pub exp2: usize,
    /// Number of logarithm base-2 operations
    pub log2: usize,
    /// Number of less-than comparison operations
    pub less_than: usize,
    /// Number of input tensor operations
    pub inputs: usize,
    /// Number of contiguous operations
    pub contiguous: usize,
}

/// Information about an input tensor in the computation graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputInfo {
    /// Unique identifier for the input
    pub id: u32,
}

/// Information about an output tensor in the computation graph
#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct OutputInfo {
    /// Whether this output is a final output of the computation
    pub is_final_output: bool,
}

/// Information about a node in the computation graph
#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct NodeInfo {
    /// Information about input tensors to this node
    pub inputs: Vec<InputInfo>,
    /// Information about the output tensor from this node
    pub output: OutputInfo,
    /// Number of nodes that consume this node's output
    pub num_consumers: u32,
    /// Unique identifier for this node
    pub id: u32,
}
