use std::fmt::Debug;

use luminair_air::{components::TraceColumn, pie::NodeInfo};
use luminal::prelude::*;

pub(crate) mod other;
pub(crate) mod prim;

/// Defines an operator that generates execution traces for proof generation.
///
/// Extends the Luminal's `Operator` trait with the ability to produce a trace and claim
/// compatible with the Stwo prover, specific to a given `TraceColumn` type `C` and table type `T`.
pub(crate) trait LuminairOperator<
    C: TraceColumn + Debug + 'static,
    T: Debug + 'static,
    L: Debug + 'static,
>: Operator
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
        lookup: &mut L,
    ) -> Vec<Tensor>;
}

/// Trait to check and invoke trace generation capabilities of an operator.
///
/// Provides methods to determine if an operator supports trace generation and
/// to execute it if available, defaulting to no support.
pub(crate) trait HasProcessTrace<
    C: TraceColumn + Debug + 'static,
    T: Debug + 'static,
    L: Debug + 'static,
>
{
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
        _lookup: &mut L,
    ) -> Option<Vec<Tensor>> {
        None
    }
}

/// Wraps a `LuminairOperator` to integrate with the Luminal operator system.
///
/// Bridges operators that generate traces with the standard Luminal's `Operator` trait.
#[derive(Debug)]
struct LuminairWrapper<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static>(
    Box<dyn LuminairOperator<C, T, L>>,
);

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static> Operator
    for LuminairWrapper<C, T, L>
{
    /// Delegates processing to the wrapped `LuminairOperator`.
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        self.0.process(inp)
    }
}

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static>
    HasProcessTrace<C, T, L> for LuminairWrapper<C, T, L>
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
        lookup: &mut L,
    ) -> Option<Vec<Tensor>> {
        Some(self.0.process_trace(inp, table, node_info, lookup))
    }
}

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static>
    HasProcessTrace<C, T, L> for Box<dyn Operator>
{
    /// Checks if the boxed operator is a `LuminairWrapper` supporting tracing.
    fn has_process_trace(&self) -> bool {
        if let Some(wrapper) = (**self).as_any().downcast_ref::<LuminairWrapper<C, T, L>>() {
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
        lookup: &mut L,
    ) -> Option<Vec<Tensor>> {
        if let Some(wrapper) = (**self)
            .as_any_mut()
            .downcast_mut::<LuminairWrapper<C, T, L>>()
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
pub(crate) trait IntoOperator<
    C: TraceColumn + Debug + 'static,
    T: Debug + 'static,
    L: Debug + 'static,
>
{
    fn into_operator(self) -> Box<dyn Operator>;
}

impl<O, C, T, L> IntoOperator<C, T, L> for O
where
    O: LuminairOperator<C, T, L> + 'static,
    C: TraceColumn + Debug + 'static,
    T: Debug + 'static,
    L: Debug + 'static,
{
    /// Wraps the operator in a `LuminairWrapper` and boxes it.
    fn into_operator(self) -> Box<dyn Operator> {
        Box::new(LuminairWrapper(Box::new(self)))
    }
}
