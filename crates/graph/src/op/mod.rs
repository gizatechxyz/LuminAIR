use std::fmt::Debug;

use luminair_air::{components::TraceColumn, pie::NodeInfo};
use luminal::prelude::*;

pub(crate) mod other;
pub(crate) mod prim;

/// Trait for LuminAIR operators that can process traces
/// 
/// Extends the base Operator trait with trace processing capabilities
/// for STARK proving and verification
pub(crate) trait LuminairOperator<
    C: TraceColumn + Debug + 'static, // The specific column structure for this op's trace
    T: Debug + 'static,             // The table type to store trace entries (e.g., AddTraceTable)
    L: Debug + 'static,             // Auxiliary lookup data/helper (e.g., SinLookup)
>: Operator
{
    /// Processes the operation and generates trace data for proving
    /// 
    /// Takes input tensors and generates both output tensors and trace table entries
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut T,
        node_info: &NodeInfo,
        lookup: &mut L,
    ) -> Vec<Tensor>;
}

/// Trait for checking if an operator supports trace processing
/// 
/// Provides a way to check and call trace processing methods on operators
/// that may or may not support them
pub(crate) trait HasProcessTrace<
    C: TraceColumn + Debug + 'static, // The specific column structure for this op's trace
    T: Debug + 'static,               // The table type to store trace entries
    L: Debug + 'static,               // Auxiliary lookup data/helper
>
{
    /// Returns true if this operator supports trace processing
    fn has_process_trace(&self) -> bool {
        false
    }

    /// Attempts to call the trace processing method if supported
    /// 
    /// Returns Some(result) if trace processing is supported, None otherwise
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

/// Wrapper struct that implements Operator for LuminairOperator types
/// 
/// Allows LuminairOperator implementations to be used as regular Operator types
/// while maintaining access to trace processing capabilities
struct LuminairWrapper<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static>(
    Box<dyn LuminairOperator<C, T, L>>,
);

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static> core::fmt::Debug
    for LuminairWrapper<C, T, L>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static> Operator
    for LuminairWrapper<C, T, L>
{
    /// Processes input tensors using the wrapped LuminairOperator
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        self.0.process(inp)
    }
}

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static>
    HasProcessTrace<C, T, L> for LuminairWrapper<C, T, L>
{
    /// Always returns true for LuminairWrapper
    fn has_process_trace(&self) -> bool {
        true
    }

    /// Delegates trace processing to the wrapped LuminairOperator
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
    /// Checks if the boxed operator supports trace processing for the given types
    fn has_process_trace(&self) -> bool {
        if let Some(wrapper) = (**self).as_any().downcast_ref::<LuminairWrapper<C, T, L>>() {
            wrapper.has_process_trace()
        } else {
            false
        }
    }

    /// Attempts to call trace processing on the boxed operator if supported
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

/// Trait for converting LuminairOperator types into boxed Operator types
/// 
/// Provides a convenient way to wrap LuminairOperator implementations
/// for use in the broader luminal framework
pub(crate) trait IntoOperator<
    C: TraceColumn + Debug + 'static, // The specific column structure for the op's trace
    T: Debug + 'static,               // The table type for trace entries
    L: Debug + 'static,               // Auxiliary lookup data/helper
>
{
    /// Converts self into a boxed Operator
    fn into_operator(self) -> Box<dyn Operator>;
}

impl<O, C, T, L> IntoOperator<C, T, L> for O
where
    O: LuminairOperator<C, T, L> + 'static,
    C: TraceColumn + Debug + 'static,
    T: Debug + 'static,
    L: Debug + 'static,
{
    /// Wraps the LuminairOperator in a LuminairWrapper and boxes it
    fn into_operator(self) -> Box<dyn Operator> {
        Box::new(LuminairWrapper(Box::new(self)))
    }
}
