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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TraceTable {
    Add { table: AddTraceTable },
    Mul { table: MulTraceTable },
    Recip { table: RecipTraceTable },
    Sin { table: SinTraceTable },
    SinLookup { table: SinLookupTraceTable },
    SumReduce { table: SumReduceTraceTable },
    MaxReduce { table: MaxReduceTraceTable },
    Sqrt { table: SqrtTraceTable },
    Rem { table: RemTraceTable },
    Exp2 { table: Exp2TraceTable },
    Exp2Lookup { table: Exp2LookupTraceTable },
    Log2 { table: Log2TraceTable },
    Log2Lookup { table: Log2LookupTraceTable },
    LessThan { table: LessThanTraceTable },
    RangeCheckLookup { table: RangeCheckLookupTraceTable },
    Inputs { table: InputsTraceTable },
    Contiguous { table: ContiguousTraceTable },
}

impl TraceTable {
    pub fn from_add(table: AddTraceTable) -> Self {
        Self::Add { table }
    }
    pub fn from_mul(table: MulTraceTable) -> Self {
        Self::Mul { table }
    }
    pub fn from_recip(table: RecipTraceTable) -> Self {
        Self::Recip { table }
    }
    pub fn from_sin(table: SinTraceTable) -> Self {
        Self::Sin { table }
    }
    pub fn from_sin_lookup(table: SinLookupTraceTable) -> Self {
        Self::SinLookup { table }
    }
    pub fn from_sum_reduce(table: SumReduceTraceTable) -> Self {
        Self::SumReduce { table }
    }
    pub fn from_max_reduce(table: MaxReduceTraceTable) -> Self {
        Self::MaxReduce { table }
    }
    pub fn from_sqrt(table: SqrtTraceTable) -> Self {
        Self::Sqrt { table }
    }
    pub fn from_rem(table: RemTraceTable) -> Self {
        Self::Rem { table }
    }
    pub fn from_exp2(table: Exp2TraceTable) -> Self {
        Self::Exp2 { table }
    }
    pub fn from_exp2_lookup(table: Exp2LookupTraceTable) -> Self {
        Self::Exp2Lookup { table }
    }
    pub fn from_log2(table: Log2TraceTable) -> Self {
        Self::Log2 { table }
    }
    pub fn from_log2_lookup(table: Log2LookupTraceTable) -> Self {
        Self::Log2Lookup { table }
    }
    pub fn from_less_than(table: LessThanTraceTable) -> Self {
        Self::LessThan { table }
    }
    pub fn from_range_check_lookup(table: RangeCheckLookupTraceTable) -> Self {
        Self::RangeCheckLookup { table }
    }
    pub fn from_inputs(table: InputsTraceTable) -> Self {
        Self::Inputs { table }
    }
    pub fn from_contiguous(table: ContiguousTraceTable) -> Self {
        Self::Contiguous { table }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairPie {
    pub trace_tables: Vec<TraceTable>,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub execution_resources: ExecutionResources,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LUTMultiplicities {
    pub sin: AtomicMultiplicityColumn,
    pub exp2: AtomicMultiplicityColumn,
    pub log2: AtomicMultiplicityColumn,
    pub range_check: AtomicMultiplicityColumn,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionResources {
    pub op_counter: OpCounter,
    pub max_log_size: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OpCounter {
    pub add: usize,
    pub mul: usize,
    pub recip: usize,
    pub sin: usize,
    pub sum_reduce: usize,
    pub max_reduce: usize,
    pub sqrt: usize,
    pub rem: usize,
    pub exp2: usize,
    pub log2: usize,
    pub less_than: usize,
    pub inputs: usize,
    pub contiguous: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputInfo {
    pub id: u32,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct OutputInfo {
    pub is_final_output: bool,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq)]
pub struct NodeInfo {
    pub inputs: Vec<InputInfo>,
    pub output: OutputInfo,
    pub num_consumers: u32,
    pub id: u32,
}
