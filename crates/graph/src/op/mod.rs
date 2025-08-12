use std::fmt::Debug;

use luminair_air::{components::TraceColumn, pie::NodeInfo};
use luminal::prelude::*;

pub(crate) mod other;
pub(crate) mod prim;

pub(crate) trait LuminairOperator<
    C: TraceColumn + Debug + 'static, // The specific column structure for this op's trace
    T: Debug + 'static,             // The table type to store trace entries (e.g., AddTraceTable)
    L: Debug + 'static,             // Auxiliary lookup data/helper (e.g., SinLookup)
>: Operator
{
    fn process_trace(
        &mut self,
        inp: Vec<(InputTensor, ShapeTracker)>,
        table: &mut T,
        node_info: &NodeInfo,
        lookup: &mut L,
    ) -> Vec<Tensor>;
}

pub(crate) trait HasProcessTrace<
    C: TraceColumn + Debug + 'static, // The specific column structure for this op's trace
    T: Debug + 'static,               // The table type to store trace entries
    L: Debug + 'static,               // Auxiliary lookup data/helper
>
{
    fn has_process_trace(&self) -> bool {
        false
    }

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
    fn process(&mut self, inp: Vec<(InputTensor, ShapeTracker)>) -> Vec<Tensor> {
        self.0.process(inp)
    }
}

impl<C: TraceColumn + Debug + 'static, T: Debug + 'static, L: Debug + 'static>
    HasProcessTrace<C, T, L> for LuminairWrapper<C, T, L>
{
    fn has_process_trace(&self) -> bool {
        true
    }

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
    fn has_process_trace(&self) -> bool {
        if let Some(wrapper) = (**self).as_any().downcast_ref::<LuminairWrapper<C, T, L>>() {
            wrapper.has_process_trace()
        } else {
            false
        }
    }

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

pub(crate) trait IntoOperator<
    C: TraceColumn + Debug + 'static, // The specific column structure for the op's trace
    T: Debug + 'static,               // The table type for trace entries
    L: Debug + 'static,               // Auxiliary lookup data/helper
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
    fn into_operator(self) -> Box<dyn Operator> {
        Box::new(LuminairWrapper(Box::new(self)))
    }
}
