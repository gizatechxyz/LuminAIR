use std::sync::Arc;

use luminair_air::{
    components::{
        add::{self, table::AddColumn},
        mul::{self, table::MulColumn},
        Claim, TraceEval,
    },
    pie::NodeInfo,
    utils::calculate_log_size,
};
use luminal::prelude::*;

use crate::{data::StwoData, utils::is};

use super::{IntoOperator, LuminairOperator};

/// Compiles primitive operations into provable forms for LuminAIR.
///
/// Replaces standard Luminal operators with LuminAIR-specific implementations
/// that support trace generation.
#[derive(Default)]
pub struct PrimitiveCompiler {}

impl PrimitiveCompiler {
    /// Creates a new `PrimitiveCompiler` instance.
    pub fn new() -> Self {
        Self {}
    }
}

// ====== BINARY ======

/// Implements element-wise addition for LuminAIR.
#[derive(Debug, Clone, Default, PartialEq)]
struct LuminairAdd {}

impl LuminairAdd {
    /// Creates a new `LuminairAdd` instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl LuminairOperator<AddColumn> for LuminairAdd {
    /// Processes two input tensors, generating a trace, claim, and output tensor.
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        node_info: &NodeInfo,
    ) -> (TraceEval, Claim<AddColumn>, Vec<Tensor>) {
        // Get data
        let (lhs_tensor, _) = &inp[0];
        let (rhs_tensor, _) = &inp[1];

        let get_data = |tensor: &InputTensor| {
            if let Some(data) = tensor.borrowed().downcast_ref::<Vec<f32>>() {
                StwoData::from_f32(data)
            } else if let Some(data) = tensor.borrowed().downcast_ref::<StwoData>() {
                StwoData(Arc::clone(&data.0))
            } else {
                panic!("Unsupported input type for Add");
            }
        };

        let lhs = get_data(lhs_tensor);
        let rhs = get_data(rhs_tensor);

        // Calculate required trace size based on tensor dimensions
        let max_size = lhs.0.len().max(rhs.0.len());
        let log_size = calculate_log_size(max_size);

        // Generate trace and get result tensor
        let (main_trace, claim, output) =
            add::table::trace_evaluation(log_size, &lhs.0, &rhs.0, node_info);

        (
            main_trace,
            claim,
            vec![Tensor::new(StwoData(Arc::new(output)))],
        )
    }
}

impl Operator for LuminairAdd {
    /// This method is not used as `process_trace` handles all computation for this operator.
    fn process(&mut self, _inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        unimplemented!()
    }
}

/// Implements element-wise mul for LuminAIR.
#[derive(Debug, Clone, Default, PartialEq)]
struct LuminairMul {}

impl LuminairMul {
    /// Creates a new `LuminairMul` instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl LuminairOperator<MulColumn> for LuminairMul {
    /// Processes two input tensors, generating a trace, claim, and output tensor.
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        node_info: &NodeInfo,
    ) -> (TraceEval, Claim<MulColumn>, Vec<Tensor>) {
        // Get data
        let (lhs_tensor, _) = &inp[0];
        let (rhs_tensor, _) = &inp[1];

        let get_data = |tensor: &InputTensor| {
            if let Some(data) = tensor.borrowed().downcast_ref::<Vec<f32>>() {
                StwoData::from_f32(data)
            } else if let Some(data) = tensor.borrowed().downcast_ref::<StwoData>() {
                StwoData(Arc::clone(&data.0))
            } else {
                panic!("Unsupported input type for Mul");
            }
        };

        let lhs = get_data(lhs_tensor);
        let rhs = get_data(rhs_tensor);

        // Calculate required trace size based on tensor dimensions
        let max_size = lhs.0.len().max(rhs.0.len());
        let log_size = calculate_log_size(max_size);

        // Generate trace and get result tensor
        let (main_trace, claim, output) =
            mul::table::trace_evaluation(log_size, &lhs.0, &rhs.0, node_info);

        (
            main_trace,
            claim,
            vec![Tensor::new(StwoData(Arc::new(output)))],
        )
    }
}

impl Operator for LuminairMul {
    /// This method is not used as `process_trace` handles all computation for this operator.
    fn process(&mut self, _inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        unimplemented!()
    }
}

impl Compiler for PrimitiveCompiler {
    type Output = ();

    /// Compiles a graph by replacing Luminal operators with LuminAIR equivalents.
    fn compile<T: ToIdsMut>(&self, graph: &mut Graph, _ids: T) -> Self::Output {
        for id in graph.node_indices().collect::<Vec<_>>() {
            let op = graph.node_weight(id).unwrap().as_any().type_id();
            let op_ref = graph.graph.node_weight_mut(id).unwrap();

            if is::<luminal::op::Add>(op) {
                *op_ref = LuminairAdd::new().into_operator()
            } else if is::<luminal::op::Mul>(op) {
                *op_ref = LuminairMul::new().into_operator()
            } else if is::<luminal::op::Contiguous>(op) {
                *op_ref = Box::new(Contiguous)
            } else if is::<luminal::op::Function>(op) {
                continue;
            } else {
                panic!("Operator not implemented yet!")
            }
        }
    }
}
