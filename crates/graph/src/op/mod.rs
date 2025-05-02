use std::fmt::Debug;

use luminair_air::{
    components::{lookups::Layout, TraceColumn},
    pie::NodeInfo, utils::AtomicMultiplicityColumn,
};
use luminal::prelude::*;
use stwo_prover::core::fields::m31::BaseField;

pub(crate) mod other;
pub(crate) mod prim;

pub(crate) struct Lookup {
    pub(crate) layout: Layout,
    pub(crate) multiplicities: AtomicMultiplicityColumn,
}

/// Defines an operator that generates execution traces for proof generation.
///
/// Extends the Luminal's `Operator` trait with the ability to produce a trace and claim
/// compatible with the Stwo prover, specific to a given `TraceColumn` type `C` and table type `T`.
pub(crate) trait LuminairOperator<C: TraceColumn + Debug + 'static, T: Debug + 'static>:
    Operator
{
    /// Processes inputs to generate a trace, claim, and output tensor.
    ///
    /// Takes input tensors, a mutable reference to a table, and node info,
    /// and produces a trace evaluation, claim, and output tensors.
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut T,
        node_info: &NodeInfo,
        lookup: &mut Option<Lookup>,
    ) -> Vec<Tensor>;
}

/// Trait to check and invoke trace generation capabilities of an operator.
///
/// Provides methods to determine if an operator supports trace generation and
/// to execute it if available, defaulting to no support.
pub(crate) trait HasProcessTrace<C: TraceColumn + Debug + 'static, T: Debug + 'static> {
    /// Returns `true` if the operator supports trace generation, `false` otherwise.
    fn has_process_trace(&self) -> bool {
        false
    }

    /// Calls `process_trace` if supported, returning the result or `None`.
    fn call_process_trace(
        &mut self,
        _inp: Vec<(InputTensor, ShapeTracker)>,
        _table: &mut T,
        _node_info: &NodeInfo,
        _lookup: &mut Option<Lookup>,
    ) -> Option<Vec<Tensor>> {
        None
    }
}

/// Wraps a `LuminairOperator` to integrate with the Luminal operator system.
///
/// Bridges operators that generate traces with the standard Luminal's `Operator` trait.
#[derive(Debug)]
struct LuminairWrapper<C: TraceColumn + Debug + 'static, T: Debug + 'static>(
    Box<dyn LuminairOperator<C, T>>,
);

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static> Operator for LuminairWrapper<C, T> {
    /// Delegates processing to the wrapped `LuminairOperator`.
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        self.0.process(inp)
    }
}

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static> HasProcessTrace<C, T>
    for LuminairWrapper<C, T>
{
    /// Indicates that this wrapper supports trace generation.
    fn has_process_trace(&self) -> bool {
        true
    }

    /// Invokes the wrapped operator's `process_trace` method.
    fn call_process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut T,
        node_info: &NodeInfo,
        lookup: &mut Option<Lookup>,
    ) -> Option<Vec<Tensor>> {
        Some(self.0.process_trace(inp, table, node_info, lookup))
    }
}

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static> HasProcessTrace<C, T>
    for Box<dyn Operator>
{
    /// Checks if the boxed operator is a `LuminairWrapper` supporting tracing.
    fn has_process_trace(&self) -> bool {
        if let Some(wrapper) = (**self).as_any().downcast_ref::<LuminairWrapper<C, T>>() {
            wrapper.has_process_trace()
        } else {
            false
        }
    }

    /// Calls `process_trace` on the boxed operator if it's a `LuminairWrapper`.
    fn call_process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut T,
        node_info: &NodeInfo,
        lookup: &mut Option<Lookup>,
    ) -> Option<Vec<Tensor>> {
        if let Some(wrapper) = (**self)
            .as_any_mut()
            .downcast_mut::<LuminairWrapper<C, T>>()
        {
            wrapper.call_process_trace(inp, table, node_info, lookup)
        } else {
            None
        }
    }
}

/// Converts a type into a boxed `Operator` for use in the graph.
///
/// Facilitates wrapping `LuminairOperator` implementations into the Luminal system.
pub(crate) trait IntoOperator<C: TraceColumn + Debug + 'static, T: Debug + 'static> {
    fn into_operator(self) -> Box<dyn Operator>;
}

impl<O, C, T> IntoOperator<C, T> for O
where
    O: LuminairOperator<C, T> + 'static,
    C: TraceColumn + Debug + 'static,
    T: Debug + 'static,
{
    /// Wraps the operator in a `LuminairWrapper` and boxes it.
    fn into_operator(self) -> Box<dyn Operator> {
        Box::new(LuminairWrapper(Box::new(self)))
    }
}
