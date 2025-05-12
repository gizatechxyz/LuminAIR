use std::fmt::Debug;

use luminair_air::{components::TraceColumn, pie::NodeInfo};
use luminal::prelude::*;

pub(crate) mod other;
pub(crate) mod prim;

/// Defines an operator specifically designed for LuminAIR, capable of generating execution traces.
///
/// This trait extends Luminal's `Operator` trait. An implementation must provide
/// a `process_trace` method that, given input tensors, a mutable trace table (`T`),
/// node information, and a mutable lookup helper (`L`), produces output tensors and populates the table.
/// The `C` generic parameter represents the specific `TraceColumn` type for this operator.
pub(crate) trait LuminairOperator<
    C: TraceColumn + Debug + 'static, // The specific column structure for this op's trace
    T: Debug + 'static,             // The table type to store trace entries (e.g., AddTraceTable)
    L: Debug + 'static,             // Auxiliary lookup data/helper (e.g., SinLookup)
>: Operator
{
    /// Processes input tensors to produce output tensors and populate the trace table.
    ///
    /// This method is responsible for the core logic of the operator when generating
    /// a trace for the STWO prover. It records relevant data into the `table`.
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut T,
        node_info: &NodeInfo,
        lookup: &mut L,
    ) -> Vec<Tensor>;
}

/// A trait to dynamically check if an operator supports trace generation and to invoke it.
///
/// This allows the graph execution logic to determine if a generic `Box<dyn Operator>`
/// can be used to generate a specific kind of trace (defined by `C`, `T`, `L`)
/// and to call its `process_trace` method if so.
pub(crate) trait HasProcessTrace<
    C: TraceColumn + Debug + 'static, // The specific column structure for this op's trace
    T: Debug + 'static,             // The table type to store trace entries
    L: Debug + 'static,             // Auxiliary lookup data/helper
>
{
    /// Returns `true` if the operator implements `LuminairOperator` for the given `C`, `T`, `L`.
    fn has_process_trace(&self) -> bool {
        false
    }

    /// Calls the `process_trace` method of the underlying `LuminairOperator`.
    ///
    /// Returns `Some(Vec<Tensor>)` if the operator supports and successfully executes trace generation,
    /// otherwise `None`.
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

/// Wraps a `LuminairOperator` to make it compatible with Luminal's `Operator` trait
/// while also exposing trace generation capabilities via `HasProcessTrace`.
///
/// The generic parameters `C`, `T`, and `L` correspond to the `TraceColumn` type,
/// the trace table type, and the lookup helper type of the wrapped operator, respectively.
#[derive(Debug)]
struct LuminairWrapper<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static>(
    Box<dyn LuminairOperator<C, T, L>>,
);

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static> Operator
    for LuminairWrapper<C, T, L>
{
    /// Delegates the standard `process` call to the wrapped `LuminairOperator`.
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        self.0.process(inp)
    }
}

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static>
    HasProcessTrace<C, T, L> for LuminairWrapper<C, T, L>
{
    /// Confirms that this wrapper, by definition, supports trace generation.
    fn has_process_trace(&self) -> bool {
        true
    }

    /// Invokes the `process_trace` method of the wrapped `LuminairOperator`.
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
    /// Checks if the dynamically-typed `Box<dyn Operator>` is a `LuminairWrapper`
    /// that supports the specific trace generation types (`C`, `T`, `L`).
    fn has_process_trace(&self) -> bool {
        if let Some(wrapper) = (**self).as_any().downcast_ref::<LuminairWrapper<C, T, L>>() {
            wrapper.has_process_trace()
        } else {
            false
        }
    }

    /// Dynamically calls `process_trace` on the `Box<dyn Operator>` if it is a
    /// `LuminairWrapper` matching the trace types (`C`, `T`, `L`).
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

/// A utility trait to convert a `LuminairOperator` into a `Box<dyn Operator>`.
///
/// This simplifies the creation of graph nodes from custom LuminAIR operators by automatically
/// wrapping them in `LuminairWrapper`. The `C`, `T`, and `L` parameters specify the
/// trace generation signature of the operator being converted.
pub(crate) trait IntoOperator<
    C: TraceColumn + Debug + 'static, // The specific column structure for the op's trace
    T: Debug + 'static,             // The table type for trace entries
    L: Debug + 'static,             // Auxiliary lookup data/helper
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
    /// Wraps the `LuminairOperator` (self) in `LuminairWrapper` and then boxes it.
    fn into_operator(self) -> Box<dyn Operator> {
        Box::new(LuminairWrapper(Box::new(self)))
    }
}
