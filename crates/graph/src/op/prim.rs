use luminair_air::{
    components::{
        add::table::{AddColumn, AddTraceTable, AddTraceTableRow},
        lookups::sin::SinLookup,
        max_reduce::table::{MaxReduceColumn, MaxReduceTraceTable, MaxReduceTraceTableRow},
        mul::table::{MulColumn, MulTraceTable, MulTraceTableRow},
        recip::table::{RecipColumn, RecipTraceTable, RecipTraceTableRow},
        sin::table::{SinColumn, SinTraceTable, SinTraceTableRow},
        sqrt::table::{SqrtColumn, SqrtTraceTable, SqrtTraceTableRow},
        sum_reduce::table::{SumReduceColumn, SumReduceTraceTable, SumReduceTraceTableRow},
    },
    pie::NodeInfo,
    DEFAULT_FP_SCALE,
};
use luminal::{
    op::{Function as LFunction, *},
    prelude::{petgraph::visit::EdgeRef, *},
};
use num_traits::{identities::Zero, One};
use numerair::Fixed;
use std::{ops::Deref, sync::Arc};
use stwo_prover::core::fields::m31::{BaseField, M31};

use crate::{
    data::StwoData,
    utils::{get_buffer_from_tensor, get_index, is, expansion_factor},
};

use super::{IntoOperator, LuminairOperator};

// ================== COPY ==================

/// Operator to convert tensor data from standard `Vec<f32>` to `StwoData` (fixed-point).
/// No-op if the input tensor is already `StwoData`.
#[derive(Clone, Debug)]
pub struct CopyToStwo {}
impl CopyToStwo {
    /// Creates a new `CopyToStwo` operator instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl Operator for CopyToStwo {
    fn process(&mut self, mut inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        if inp[0].0.borrowed().is::<StwoData>() {
            // Already in StwoData format, no conversion needed
            return vec![inp.pop().unwrap().0.cloned()];
        }

        // Convert Vec<f32> to StwoData
        let cpu_data = inp[0].0.borrowed().downcast_ref::<Vec<f32>>().unwrap();
        vec![Tensor::new(StwoData::from_f32(cpu_data))]
    }
}

/// Operator to convert tensor data from `StwoData` (fixed-point) back to `Vec<f32>`.
/// No-op if the input tensor is already `Vec<f32>`.
#[derive(Clone, Debug)]
pub struct CopyFromStwo {}
impl CopyFromStwo {
    /// Creates a new `CopyFromStwo` operator instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl Operator for CopyFromStwo {
    fn process(&mut self, mut inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        if inp[0].0.borrowed().is::<Vec<f32>>() {
            // Already in Vec<f32> format, no conversion needed
            return vec![inp.pop().unwrap().0.cloned()];
        }

        // Convert StwoData to Vec<f32>
        let data = inp[0].0.borrowed().downcast_ref::<StwoData>().unwrap();
        vec![Tensor::new(data.to_f32())]
    }
}

// ================== CONSTANT ================

/// Represents a constant value within the LuminAIR graph, stored as `StwoData`.
///
/// Currently supports only float constants; dynamic expressions are not yet implemented.
#[derive(Debug, Clone, PartialEq)]
pub struct LuminairConstant {
    /// The constant value.
    pub value: ConstantValue,
}

impl LuminairConstant {
    /// Creates a new `LuminairConstant` operator holding the specified value.
    pub fn new(value: ConstantValue) -> Self {
        Self { value }
    }
}

impl Operator for LuminairConstant {
    fn process(&mut self, _inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        // Create a new tensor with the constant value
        let value = match &self.value {
            ConstantValue::Float(f) => *f,
            ConstantValue::Expression(_expr) => {
                panic!("Dynamic expressions not yet supported")
            }
        };

        // Create and return a single element with the constant value
        let mut data = Vec::with_capacity(1);
        data.push(Fixed::<DEFAULT_FP_SCALE>::from_f64(value as f64));
        vec![Tensor::new(StwoData(Arc::new(data)))]
    }
}

// ================== UNARY ==================

/// LuminAIR operator for element-wise reciprocal (`1 / x`).
///
/// Implements both the standard `Operator` trait for graph execution and the
/// `LuminairOperator` trait to generate trace entries for `RecipTraceTable`.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct LuminairRecip {}

impl LuminairRecip {
    /// Creates a new `LuminairRecip` operator instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl LuminairRecip {
    fn compute(
        &self,
        inp: &[(InputTensor, ShapeTracker)],
        trace_mode: bool,
    ) -> (
        Vec<Fixed<DEFAULT_FP_SCALE>>,
        Option<
            Vec<(
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
            )>,
        >,
    ) {
        let input = get_buffer_from_tensor(&inp[0].0).unwrap();
        let expr = (inp[0].1.index_expression(), inp[0].1.valid_expression());

        let mut stack: Vec<i64> = vec![];
        let output_size = inp[0].1.n_elements().to_usize().unwrap();
        let mut out_data = vec![Fixed::<DEFAULT_FP_SCALE>::zero(); output_size];

        // Only allocate for intermediate values if in trace mode
        let mut intermediate_values = if trace_mode {
            Some(Vec::with_capacity(output_size))
        } else {
            None
        };

        for (idx, out) in out_data.iter_mut().enumerate() {
            let input_val = get_index(input, &expr, &mut stack, idx);
            let (out_val, rem_val) = input_val.recip();
            *out = out_val;

            // Only collect intermediate values if in trace mode
            if let Some(values) = &mut intermediate_values {
                values.push((input_val, out_val, rem_val));
            }
        }

        (out_data, intermediate_values)
    }
}

impl LuminairOperator<RecipColumn, RecipTraceTable, ()> for LuminairRecip {
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut RecipTraceTable,
        node_info: &NodeInfo,
        _lookup: &mut (),
    ) -> Vec<Tensor> {
        let (out_data, intermediate_values) = self.compute(&inp, true);
        let intermediate_values = intermediate_values.unwrap();

        let node_id: BaseField = node_info.id.into();
        let input_id: BaseField = node_info.inputs[0].id.into();
        let output_size = inp[0].1.n_elements().to_usize().unwrap();

        let factor = expansion_factor(&inp[0].1);
        let input_mult = if node_info.inputs[0].is_initializer {
            BaseField::zero()
        } else {
            -BaseField::from_u32_unchecked(factor)
        };
        let out_mult = if node_info.output.is_final_output {
            BaseField::zero()
        } else {
            BaseField::one() * BaseField::from_u32_unchecked(node_info.num_consumers)
        };

        for (idx, (input_val, out_val, rem_val)) in intermediate_values.into_iter().enumerate() {
            let is_last_idx: u32 = if idx == (output_size - 1) { 1 } else { 0 };

            table.add_row(RecipTraceTableRow {
                node_id,
                input_id,
                idx: idx.into(),
                is_last_idx: (is_last_idx).into(),
                next_idx: (idx + 1).into(),
                next_node_id: node_id,
                next_input_id: input_id,
                input: input_val.to_m31(),
                out: out_val.to_m31(),
                rem: rem_val.to_m31(),
                scale: M31::from_u32_unchecked(1 << DEFAULT_FP_SCALE),
                input_mult,
                out_mult,
            });
        }

        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

impl Operator for LuminairRecip {
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        let (out_data, _) = self.compute(&inp, false);
        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

/// LuminAIR operator for element-wise sine (`sin(x)`).
///
/// Implements both the standard `Operator` trait for graph execution and the
/// `LuminairOperator` trait to generate trace entries for `SinTraceTable`.
/// This operator interacts with the `SinLookup` component during trace generation
/// to record input value occurrences for the lookup argument.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct LuminairSin {}

impl LuminairSin {
    /// Creates a new `LuminairSin` operator instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl LuminairSin {
    fn compute(
        &self,
        inp: &[(InputTensor, ShapeTracker)],
        trace_mode: bool,
    ) -> (
        Vec<Fixed<DEFAULT_FP_SCALE>>,
        Option<Vec<(Fixed<DEFAULT_FP_SCALE>, Fixed<DEFAULT_FP_SCALE>)>>,
    ) {
        let input = get_buffer_from_tensor(&inp[0].0).unwrap();
        let expr = (inp[0].1.index_expression(), inp[0].1.valid_expression());

        let mut stack: Vec<i64> = vec![];
        let output_size = inp[0].1.n_elements().to_usize().unwrap();
        let mut out_data = vec![Fixed::<DEFAULT_FP_SCALE>::zero(); output_size];

        // Only allocate for intermediate values if in trace mode
        let mut intermediate_values = if trace_mode {
            Some(Vec::with_capacity(output_size))
        } else {
            None
        };

        for (idx, out) in out_data.iter_mut().enumerate() {
            let input_val = get_index(input, &expr, &mut stack, idx);
            let out_val = Fixed::<DEFAULT_FP_SCALE>::from_f64(input_val.to_f64().sin());
            *out = out_val;

            // Only collect intermediate values if in trace mode
            if let Some(values) = &mut intermediate_values {
                values.push((input_val, out_val));
            }
        }

        (out_data, intermediate_values)
    }
}

impl LuminairOperator<SinColumn, SinTraceTable, SinLookup> for LuminairSin {
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut SinTraceTable,
        node_info: &NodeInfo,
        lookup: &mut SinLookup,
    ) -> Vec<Tensor> {
        let (out_data, intermediate_values) = self.compute(&inp, true);
        let intermediate_values = intermediate_values.unwrap();

        let node_id: BaseField = node_info.id.into();
        let input_id: BaseField = node_info.inputs[0].id.into();
        let output_size = inp[0].1.n_elements().to_usize().unwrap();

        let factor = expansion_factor(&inp[0].1);
        let input_mult = if node_info.inputs[0].is_initializer {
            BaseField::zero()
        } else {
            -BaseField::from_u32_unchecked(factor)
        };
        let out_mult = if node_info.output.is_final_output {
            BaseField::zero()
        } else {
            BaseField::one() * BaseField::from_u32_unchecked(node_info.num_consumers)
        };

        for (idx, (input_val, out_val)) in intermediate_values.into_iter().enumerate() {
            let is_last_idx: u32 = if idx == (output_size - 1) { 1 } else { 0 };

            table.add_row(SinTraceTableRow {
                node_id,
                input_id,
                idx: idx.into(),
                is_last_idx: (is_last_idx).into(),
                next_idx: (idx + 1).into(),
                next_node_id: node_id,
                next_input_id: input_id,
                input: input_val.to_m31(),
                out: out_val.to_m31(),
                input_mult,
                out_mult,
                lookup_mult: M31::one(),
            });

            // Update multiplicities of the lookup.
            // Allows you to track the occurrence of a specific Sin operation.
            let mult_address = lookup
                .layout
                .find_index(input_val.0)
                .expect("Value should fit in range.");
            lookup.multiplicities.increase_at(mult_address);
        }

        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

impl Operator for LuminairSin {
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        let (out_data, _) = self.compute(&inp, false);
        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

/// LuminAIR operator for element-wise sqrt.
///
/// Implements both the standard `Operator` trait for graph execution and the
/// `LuminairOperator` trait to generate trace entries for `SqrtTraceTable`.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct LuminairSqrt {}

impl LuminairSqrt {
    /// Creates a new `LuminairSqrt` operator instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl LuminairSqrt {
    fn compute(
        &self,
        inp: &[(InputTensor, ShapeTracker)],
        trace_mode: bool,
    ) -> (
        Vec<Fixed<DEFAULT_FP_SCALE>>,
        Option<
            Vec<(
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
            )>,
        >,
    ) {
        let input = get_buffer_from_tensor(&inp[0].0).unwrap();
        let expr = (inp[0].1.index_expression(), inp[0].1.valid_expression());

        let mut stack: Vec<i64> = vec![];
        let output_size = inp[0].1.n_elements().to_usize().unwrap();
        let mut out_data = vec![Fixed::<DEFAULT_FP_SCALE>::zero(); output_size];

        // Only allocate for intermediate values if in trace mode
        let mut intermediate_values = if trace_mode {
            Some(Vec::with_capacity(output_size))
        } else {
            None
        };

        for (idx, out) in out_data.iter_mut().enumerate() {
            let input_val = get_index(input, &expr, &mut stack, idx);
            let (out_val, rem_val) = input_val.sqrt();
            *out = out_val;

            // Only collect intermediate values if in trace mode
            if let Some(values) = &mut intermediate_values {
                values.push((input_val, out_val, rem_val));
            }
        }

        (out_data, intermediate_values)
    }
}

impl LuminairOperator<SqrtColumn, SqrtTraceTable, ()> for LuminairSqrt {
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut SqrtTraceTable,
        node_info: &NodeInfo,
        _lookup: &mut (),
    ) -> Vec<Tensor> {
        let (out_data, intermediate_values) = self.compute(&inp, true);
        let intermediate_values = intermediate_values.unwrap();

        let node_id: BaseField = node_info.id.into();
        let input_id: BaseField = node_info.inputs[0].id.into();
        let output_size = inp[0].1.n_elements().to_usize().unwrap();

        let factor = expansion_factor(&inp[0].1);
        let input_mult = if node_info.inputs[0].is_initializer {
            BaseField::zero()
        } else {
            -BaseField::from_u32_unchecked(factor)
        };
        let out_mult = if node_info.output.is_final_output {
            BaseField::zero()
        } else {
            BaseField::one() * BaseField::from_u32_unchecked(node_info.num_consumers)
        };

        for (idx, (input_val, out_val, rem_val)) in intermediate_values.into_iter().enumerate() {
            let is_last_idx: u32 = if idx == (output_size - 1) { 1 } else { 0 };

            table.add_row(SqrtTraceTableRow {
                node_id,
                input_id,
                idx: idx.into(),
                is_last_idx: (is_last_idx).into(),
                next_idx: (idx + 1).into(),
                next_node_id: node_id,
                next_input_id: input_id,
                input: input_val.to_m31(),
                out: out_val.to_m31(),
                rem: rem_val.to_m31(),
                scale: M31::from_u32_unchecked(1 << DEFAULT_FP_SCALE),
                input_mult,
                out_mult,
            });
        }

        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

impl Operator for LuminairSqrt {
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        let (out_data, _) = self.compute(&inp, false);
        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

// ================== BINARY ==================

/// LuminAIR operator for element-wise addition (`a + b`).
///
/// Implements both the standard `Operator` trait for graph execution and the
/// `LuminairOperator` trait to generate trace entries for `AddTraceTable`.
#[derive(Debug, Clone, Default, PartialEq)]
struct LuminairAdd {}

impl LuminairAdd {
    /// Creates a new `LuminairAdd` operator instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl LuminairAdd {
    fn compute(
        &self,
        inp: &[(InputTensor, ShapeTracker)],
        trace_mode: bool,
    ) -> (
        Vec<Fixed<DEFAULT_FP_SCALE>>,
        Option<
            Vec<(
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
            )>,
        >,
    ) {
        let (lhs, rhs) = (
            get_buffer_from_tensor(&inp[0].0).unwrap(),
            get_buffer_from_tensor(&inp[1].0).unwrap(),
        );
        let lexpr = (inp[0].1.index_expression(), inp[0].1.valid_expression());
        let rexpr = (inp[1].1.index_expression(), inp[1].1.valid_expression());

        let mut stack: Vec<i64> = vec![];
        let output_size = inp[0].1.n_elements().to_usize().unwrap();
        let mut out_data = vec![Fixed::<DEFAULT_FP_SCALE>::zero(); output_size];

        // Only allocate for intermediate values if in trace mode
        let mut intermediate_values = if trace_mode {
            Some(Vec::with_capacity(output_size))
        } else {
            None
        };

        for (idx, out) in out_data.iter_mut().enumerate() {
            let lhs_val = get_index(lhs, &lexpr, &mut stack, idx);
            let rhs_val = get_index(rhs, &rexpr, &mut stack, idx);
            let out_val = lhs_val + rhs_val;
            *out = out_val;
            // Only collect intermediate values if in trace mode
            if let Some(values) = &mut intermediate_values {
                values.push((lhs_val, rhs_val, out_val));
            }
        }

        (out_data, intermediate_values)
    }
}

impl LuminairOperator<AddColumn, AddTraceTable, ()> for LuminairAdd {
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut AddTraceTable,
        node_info: &NodeInfo,
        _lookup: &mut (),
    ) -> Vec<Tensor> {
        let (out_data, intermediate_values) = self.compute(&inp, true);
        let intermediate_values = intermediate_values.unwrap();

        let output_size = inp[0].1.n_elements().to_usize().unwrap();
        let node_id: BaseField = node_info.id.into();
        let lhs_id: BaseField = node_info.inputs[0].id.into();
        let rhs_id: BaseField = node_info.inputs[1].id.into();

        let lhs_factor = expansion_factor(&inp[0].1);
        let rhs_factor = expansion_factor(&inp[1].1);
        let lhs_mult = if node_info.inputs[0].is_initializer {
            BaseField::zero()
        } else {
            -BaseField::from_u32_unchecked(lhs_factor)
        };
        let rhs_mult = if node_info.inputs[1].is_initializer {
            BaseField::zero()
        } else {
            -BaseField::from_u32_unchecked(rhs_factor)
        };
        let out_mult = if node_info.output.is_final_output {
            BaseField::zero()
        } else {
            BaseField::one() * BaseField::from_u32_unchecked(node_info.num_consumers)
        };

        for (idx, (lhs_val, rhs_val, out_val)) in intermediate_values.into_iter().enumerate() {
            let is_last_idx: u32 = if idx == (output_size - 1) { 1 } else { 0 };

            table.add_row(AddTraceTableRow {
                node_id,
                lhs_id,
                rhs_id,
                idx: idx.into(),
                is_last_idx: (is_last_idx).into(),
                next_idx: (idx + 1).into(),
                next_node_id: node_id,
                next_lhs_id: lhs_id,
                next_rhs_id: rhs_id,
                lhs: lhs_val.to_m31(),
                rhs: rhs_val.to_m31(),
                out: out_val.to_m31(),
                lhs_mult,
                rhs_mult,
                out_mult,
            })
        }

        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

impl Operator for LuminairAdd {
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        let (out_data, _) = self.compute(&inp, false);
        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

/// LuminAIR operator for element-wise multiplication (`a * b`).
///
/// Implements both the standard `Operator` trait for graph execution and the
/// `LuminairOperator` trait to generate trace entries for `MulTraceTable`.
#[derive(Debug, Clone, Default, PartialEq)]
struct LuminairMul {}

impl LuminairMul {
    /// Creates a new `LuminairMul` operator instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl LuminairMul {
    fn compute(
        &self,
        inp: &[(InputTensor, ShapeTracker)],
        trace_mode: bool,
    ) -> (
        Vec<Fixed<DEFAULT_FP_SCALE>>,
        Option<
            Vec<(
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
            )>,
        >,
    ) {
        let (lhs, rhs) = (
            get_buffer_from_tensor(&inp[0].0).unwrap(),
            get_buffer_from_tensor(&inp[1].0).unwrap(),
        );
        let lexpr = (inp[0].1.index_expression(), inp[0].1.valid_expression());
        let rexpr = (inp[1].1.index_expression(), inp[1].1.valid_expression());

        let mut stack: Vec<i64> = vec![];
        let output_size = inp[0].1.n_elements().to_usize().unwrap();
        let mut out_data = vec![Fixed::<DEFAULT_FP_SCALE>::zero(); output_size];

        // Only allocate for intermediate values if in trace mode
        let mut intermediate_values = if trace_mode {
            Some(Vec::with_capacity(output_size))
        } else {
            None
        };

        for (idx, out) in out_data.iter_mut().enumerate() {
            let lhs_val = get_index(lhs, &lexpr, &mut stack, idx);
            let rhs_val = get_index(rhs, &rexpr, &mut stack, idx);
            let (out_val, rem_val) = lhs_val * rhs_val;
            *out = out_val;

            // Only collect intermediate values if in trace mode
            if let Some(values) = &mut intermediate_values {
                values.push((lhs_val, rhs_val, out_val, rem_val));
            }
        }

        (out_data, intermediate_values)
    }
}

impl LuminairOperator<MulColumn, MulTraceTable, ()> for LuminairMul {
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut MulTraceTable,
        node_info: &NodeInfo,
        _lookup: &mut (),
    ) -> Vec<Tensor> {
        let (out_data, intermediate_values) = self.compute(&inp, true);
        let intermediate_values = intermediate_values.unwrap();

        let output_size = inp[0].1.n_elements().to_usize().unwrap();
        let node_id: BaseField = node_info.id.into();
        let lhs_id: BaseField = node_info.inputs[0].id.into();
        let rhs_id: BaseField = node_info.inputs[1].id.into();

        let lhs_factor = expansion_factor(&inp[0].1);
        let rhs_factor = expansion_factor(&inp[1].1);
        let lhs_mult = if node_info.inputs[0].is_initializer {
            BaseField::zero()
        } else {
            -BaseField::from_u32_unchecked(lhs_factor)
        };
        let rhs_mult = if node_info.inputs[1].is_initializer {
            BaseField::zero()
        } else {
            -BaseField::from_u32_unchecked(rhs_factor)
        };
        let out_mult = if node_info.output.is_final_output {
            BaseField::zero()
        } else {
            BaseField::one() * BaseField::from_u32_unchecked(node_info.num_consumers)
        };

        for (idx, (lhs_val, rhs_val, out_val, rem_val)) in
            intermediate_values.into_iter().enumerate()
        {
            let is_last_idx: u32 = if idx == (output_size - 1) { 1 } else { 0 };

            table.add_row(MulTraceTableRow {
                node_id,
                lhs_id,
                rhs_id,
                idx: idx.into(),
                is_last_idx: (is_last_idx).into(),
                next_idx: (idx + 1).into(),
                next_node_id: node_id,
                next_lhs_id: lhs_id,
                next_rhs_id: rhs_id,
                lhs: lhs_val.to_m31(),
                rhs: rhs_val.to_m31(),
                out: out_val.to_m31(),
                rem: rem_val.to_m31(),
                lhs_mult,
                rhs_mult,
                out_mult,
            })
        }

        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

impl Operator for LuminairMul {
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        let (out_data, _) = self.compute(&inp, false);
        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

// ================== REDUCE ==================

/// LuminAIR operator for sum reduction along a specified dimension.
///
/// Implements both the standard `Operator` trait for graph execution and the
/// `LuminairOperator` trait to generate trace entries for `SumReduceTraceTable`,
/// capturing the accumulation process step-by-step.
#[derive(Debug, Clone, Default, PartialEq)]
struct LuminairSumReduce(pub usize);

impl LuminairSumReduce {
    /// Creates a new `LuminairSumReduce` operator instance for the given reduction dimension.
    pub fn new(value: usize) -> Self {
        Self(value)
    }
}

impl LuminairSumReduce {
    fn compute(
        &self,
        inp: &[(InputTensor, ShapeTracker)],
        trace_mode: bool,
    ) -> (
        Vec<Fixed<DEFAULT_FP_SCALE>>,
        Option<
            Vec<(
                usize,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                BaseField,
            )>,
        >,
    ) {
        let sh = inp[0].1.shape_usize();
        let front_size = sh.iter().take(self.0).product::<usize>().max(1);
        let back_size = sh.iter().skip(self.0 + 1).product::<usize>().max(1);
        let dim_size = sh[self.0];

        let output_size = front_size * back_size;
        let mut out_data = vec![Fixed::<DEFAULT_FP_SCALE>::zero(); output_size];
        let input = get_buffer_from_tensor(&inp[0].0).unwrap();
        let expr = (inp[0].1.index_expression(), inp[0].1.valid_expression());
        let mut stack: Vec<i64> = vec![];

        // Only allocate for intermediate values if in trace mode
        let mut intermediate_values = if trace_mode {
            Some(Vec::with_capacity(output_size))
        } else {
            None
        };

        for i in 0..front_size {
            for j in 0..back_size {
                let mut acc = Fixed::<DEFAULT_FP_SCALE>::zero(); // Initialize accumulator for each (i, j)
                for k in 0..dim_size {
                    let orig_index = i * dim_size * back_size + k * back_size + j;
                    let input_val = get_index(input, &expr, &mut stack, orig_index);
                    let next_acc = acc + input_val; // Compute next accumulator
                    let idx = i * back_size + j; // Index for out_data

                    // Set out_data only in the last reduction step
                    let (out_val, is_last_step) = if k == dim_size - 1 {
                        out_data[idx] = next_acc;
                        (next_acc, BaseField::one())
                    } else {
                        (Fixed::<DEFAULT_FP_SCALE>::zero(), BaseField::zero()) // Placeholder for incomplete reductions
                    };

                    // Record intermediate values if in trace mode
                    if let Some(values) = &mut intermediate_values {
                        values.push((idx, input_val, out_val, acc, next_acc, is_last_step));
                    }
                    // Update running sum
                    acc = next_acc;
                }
            }
        }

        (out_data, intermediate_values)
    }
}

impl LuminairOperator<SumReduceColumn, SumReduceTraceTable, ()> for LuminairSumReduce {
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut SumReduceTraceTable,
        node_info: &NodeInfo,
        _lookup: &mut (),
    ) -> Vec<Tensor> {
        let (out_data, intermediate_values) = self.compute(&inp, true);
        let intermediate_values = intermediate_values.unwrap();

        let node_id: BaseField = node_info.id.into();
        let input_id: BaseField = node_info.inputs[0].id.into();
        let output_size = out_data.len();

        let factor = expansion_factor(&inp[0].1);
        let input_mult = if node_info.inputs[0].is_initializer {
            BaseField::zero()
        } else {
            -BaseField::from_u32_unchecked(factor)
        };
        let out_mult = if node_info.output.is_final_output {
            BaseField::zero()
        } else {
            BaseField::one() * BaseField::from_u32_unchecked(node_info.num_consumers)
        };

        for entry in intermediate_values {
            let (idx, input_val, out_val, acc, next_acc, is_last_step) = entry;

            let out_mult = out_mult * is_last_step;

            let is_last_idx: u32 = if idx == (output_size - 1) { 1 } else { 0 };

            table.add_row(SumReduceTraceTableRow {
                node_id,
                input_id,
                idx: idx.into(),
                is_last_idx: (is_last_idx).into(),
                next_node_id: node_id,
                next_input_id: input_id,
                next_idx: (idx + 1).into(),
                input: input_val.to_m31(),
                out: out_val.to_m31(),
                acc: acc.to_m31(),
                next_acc: next_acc.to_m31(),
                is_last_step,
                input_mult,
                out_mult,
            });
        }

        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

impl Operator for LuminairSumReduce {
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        let (out_data, _) = self.compute(&inp, false);
        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

/// LuminAIR operator for max reduction along a specified dimension.
///
/// Implements both the standard `Operator` trait for graph execution and the
/// `LuminairOperator` trait to generate trace entries for `MaxReduceTraceTable`,
/// capturing the comparison and update process step-by-step.
#[derive(Debug, Clone, Default, PartialEq)]
struct LuminairMaxReduce(pub usize);

impl LuminairMaxReduce {
    /// Creates a new `LuminairMaxReduce` operator instance for the given reduction dimension.
    pub fn new(value: usize) -> Self {
        Self(value)
    }
}

impl LuminairMaxReduce {
    fn compute(
        &self,
        inp: &[(InputTensor, ShapeTracker)],
        trace_mode: bool,
    ) -> (
        Vec<Fixed<DEFAULT_FP_SCALE>>,
        Option<
            Vec<(
                usize,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                Fixed<DEFAULT_FP_SCALE>,
                BaseField,
                BaseField,
            )>,
        >,
    ) {
        let sh = inp[0].1.shape_usize();
        let front_size = sh.iter().take(self.0).product::<usize>().max(1);
        let back_size = sh.iter().skip(self.0 + 1).product::<usize>().max(1);
        let dim_size = sh[self.0];

        let output_size = front_size * back_size;
        let mut out_data = vec![Fixed::<DEFAULT_FP_SCALE>::zero(); output_size];
        let input = get_buffer_from_tensor(&inp[0].0).unwrap();
        let expr = (inp[0].1.index_expression(), inp[0].1.valid_expression());
        let mut stack: Vec<i64> = vec![];

        // Only allocate for intermediate values if in trace mode
        let mut intermediate_values = if trace_mode {
            Some(Vec::with_capacity(output_size))
        } else {
            None
        };

        for i in 0..front_size {
            for j in 0..back_size {
                // Initialize with the first element instead of negative infinity
                let orig_first_index = i * dim_size * back_size + 0 * back_size + j;
                let mut max_val = get_index(input, &expr, &mut stack, orig_first_index);

                for k in 0..dim_size {
                    let orig_index = i * dim_size * back_size + k * back_size + j;
                    let input_val = get_index(input, &expr, &mut stack, orig_index);

                    // Determine if this value is the new max
                    let is_max = if input_val.to_f64() > max_val.to_f64() {
                        BaseField::one()
                    } else {
                        BaseField::zero()
                    };

                    // Update max_val if needed
                    let next_max_val = if is_max == BaseField::one() {
                        input_val
                    } else {
                        max_val
                    };

                    // Set out_data only in the last reduction step
                    let (out_val, is_last_step) = if k == dim_size - 1 {
                        out_data[i * back_size + j] = next_max_val;
                        (next_max_val, BaseField::one())
                    } else {
                        (Fixed::<DEFAULT_FP_SCALE>::zero(), BaseField::zero()) // Placeholder for incomplete reductions
                    };

                    let idx = i * back_size + j; // Index for out_data

                    // Record intermediate values if in trace mode
                    if let Some(values) = &mut intermediate_values {
                        values.push((
                            idx,
                            input_val,
                            out_val,
                            max_val,
                            next_max_val,
                            is_max,
                            is_last_step,
                        ));
                    }

                    // Update running maximum
                    max_val = next_max_val;
                }
            }
        }

        (out_data, intermediate_values)
    }
}

impl LuminairOperator<MaxReduceColumn, MaxReduceTraceTable, ()> for LuminairMaxReduce {
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut MaxReduceTraceTable,
        node_info: &NodeInfo,
        _lookup: &mut (),
    ) -> Vec<Tensor> {
        let (out_data, intermediate_values) = self.compute(&inp, true);
        let intermediate_values = intermediate_values.unwrap();

        let node_id: BaseField = node_info.id.into();
        let input_id: BaseField = node_info.inputs[0].id.into();
        let output_size = out_data.len();

        let factor = expansion_factor(&inp[0].1);
        let input_mult = if node_info.inputs[0].is_initializer {
            BaseField::zero()
        } else {
            -BaseField::from_u32_unchecked(factor)
        };

        let out_mult = if node_info.output.is_final_output {
            BaseField::zero()
        } else {
            BaseField::one() * BaseField::from_u32_unchecked(node_info.num_consumers)
        };

        for entry in intermediate_values {
            let (idx, input_val, out_val, max_val, next_max_val, is_max, is_last_step_flag) = entry;

            let out_mult = out_mult * is_last_step_flag;

            let is_last_idx: u32 = if idx == (output_size - 1) { 1 } else { 0 };

            table.add_row(MaxReduceTraceTableRow {
                node_id,
                input_id,
                idx: idx.into(),
                is_last_idx: (is_last_idx).into(),
                next_node_id: node_id,
                next_input_id: input_id,
                next_idx: (idx + 1).into(),
                input: input_val.to_m31(),
                out: out_val.to_m31(),
                max_val: max_val.to_m31(),
                next_max_val: next_max_val.to_m31(),
                is_max,
                is_last_step: is_last_step_flag,
                input_mult,
                out_mult,
            });
        }

        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

impl Operator for LuminairMaxReduce {
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        let (out_data, _) = self.compute(&inp, false);
        vec![Tensor::new(StwoData(Arc::new(out_data)))]
    }
}

// ================== COMPILER ==================

/// A Luminal `Compiler` pass that adapts a standard computation graph for LuminAIR.
///
/// This compiler performs two main tasks:
/// 1. **Inserts Copy Operators:** Adds `CopyToStwo` and `CopyFromStwo` nodes at the boundaries
///    between standard data formats (like `Vec<f32>` used by `Function` nodes or for output)
///    and the `StwoData` format used internally by LuminAIR operators.
/// 2. **Replaces Primitive Operators:** Substitutes standard Luminal operators (e.g., `luminal::op::Add`)
///    with their LuminAIR counterparts (e.g., `LuminairAdd`) that implement trace generation.
#[derive(Default)]
pub struct PrimitiveCompiler();

impl Compiler for PrimitiveCompiler {
    type Output = ();

    /// Executes the compilation pass on the graph.
    /// Modifies the graph in-place to insert copy operators and replace primitives.
    fn compile<T: ToIdsMut>(&self, graph: &mut Graph, mut ids: T) -> Self::Output {
        // Go through the graph and insert copy ops.
        // Copy Function nodes (data input/output)
        for function_node in graph
            .node_indices()
            .filter(|n| {
                graph.node_weight(*n).unwrap().as_any().is::<Function>()
                    && graph.edges(*n).count() != 0
            })
            .collect::<Vec<_>>()
        {
            // Create CopyToStwo to convert Vec<f32> data to StwoData after function outputs
            let copy_node = graph
                .add_op(CopyToStwo::new())
                .input(function_node, 0, ShapeTracker::new(()))
                .finish();

            // Switch outgoing edges from input to copy_node
            for (edge_id, weight, dest) in graph
                .edges_directed(function_node, petgraph::Direction::Outgoing)
                .map(|e| (e.id(), *e.weight(), e.target()))
                .filter(|(_, _, trg)| *trg != copy_node)
                .collect::<Vec<_>>()
            {
                graph.add_edge(copy_node, dest, weight);
                graph.remove_edge(edge_id);
            }

            // Handle no_delete and to_retrieve for the function node
            if graph.no_delete.remove(&function_node) {
                graph.no_delete.insert(copy_node);
            }
            if let Some(v) = graph.to_retrieve.get(&function_node) {
                graph.to_retrieve.insert(copy_node, *v);
            }

            // Insert copy from Stwo for function inputs
            for (source, edge, edge_weight) in graph
                .edges_directed(function_node, petgraph::Direction::Incoming)
                .map(|e| (e.source(), e.id(), *e.weight()))
                .collect::<Vec<_>>()
            {
                let copy_from_node = graph
                    .add_op(CopyFromStwo::new())
                    .input(source, 0, ShapeTracker::new(()))
                    .finish();
                graph.add_edge(copy_from_node, function_node, edge_weight);
                graph.remove_edge(edge);
            }
        }

        // Add CopyFromStwo for retrieved outputs
        for (output_node, (_, output_shape)) in graph
            .to_retrieve
            .iter()
            .map(|(a, b)| (*a, *b))
            // Filter to non-functions
            .filter(|(n, _)| !graph.node_weight(*n).unwrap().as_any().is::<LFunction>())
            .collect::<Vec<_>>()
        {
            if graph
                .node_weight(output_node)
                .unwrap()
                .as_any()
                .is::<CopyToStwo>()
            {
                // This output is already a copy to, instead of adding a copy from, let's remap back to the source
                let src = graph
                    .neighbors_directed(output_node, petgraph::Direction::Incoming)
                    .next()
                    .unwrap();
                graph.no_delete.remove(&output_node);
                graph.no_delete.insert(src);
                let w = graph.to_retrieve.remove(&output_node).unwrap();
                graph.to_retrieve.insert(src, w);
            } else {
                // Create copy node
                let copy_node = graph
                    .add_op(CopyFromStwo::new())
                    .input(output_node, 0, output_shape)
                    .finish();

                remap(output_node, copy_node, &mut ids, graph);
            }
        }

        // Replace Luminal's ops with LuminAIR ops
        for id in graph.node_indices().collect::<Vec<_>>() {
            let op = graph.node_weight(id).unwrap().as_any().type_id();
            let op_ref = graph.graph.node_weight_mut(id).unwrap();

            if let Some(c) = op_ref.as_any().downcast_ref::<luminal::op::Constant>() {
                *op_ref = Box::new(LuminairConstant::new(c.0.clone()));
            } else if is::<luminal::op::Add>(op) {
                *op_ref = LuminairAdd::new().into_operator()
            } else if is::<luminal::op::Mul>(op) {
                *op_ref = LuminairMul::new().into_operator()
            } else if is::<luminal::op::Recip>(op) {
                *op_ref = LuminairRecip::new().into_operator()
            } else if is::<luminal::op::Sin>(op) {
                *op_ref = LuminairSin::new().into_operator()
            } else if is::<luminal::op::SumReduce>(op) {
                let dim_index =
                    if let Some(sum_reduce) = op_ref.deref().as_any().downcast_ref::<SumReduce>() {
                        sum_reduce.0 // Access the usize field (the 0 in SumReduce(0))
                    } else {
                        0
                    };
                *op_ref = LuminairSumReduce::new(dim_index).into_operator()
            } else if is::<luminal::op::MaxReduce>(op) {
                let dim_index =
                    if let Some(max_reduce) = op_ref.deref().as_any().downcast_ref::<MaxReduce>() {
                        max_reduce.0 // Access the usize field
                    } else {
                        0
                    };
                *op_ref = LuminairMaxReduce::new(dim_index).into_operator()
            } else if is::<luminal::op::Sqrt>(op) {
                *op_ref = LuminairSqrt::new().into_operator()
            }
        }
    }
}
