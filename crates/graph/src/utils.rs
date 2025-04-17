use std::any::{Any, TypeId};

use crate::data::StwoData;
use luminair_air::preprocessed::Range;
use luminal::prelude::*;
use num_traits::Zero;
use numerair::Fixed;

/// Checks if a `TypeId` matches the type `T`.
pub(crate) fn is<T: Any>(type_id: TypeId) -> bool {
    type_id == TypeId::of::<T>()
}

/// Extracts the StwoData reference from an InputTensor.
pub(crate) fn get_buffer_from_tensor<'a>(tensor: &'a InputTensor) -> Option<&'a StwoData> {
    tensor.borrowed().downcast_ref::<StwoData>()
}

/// Retrieves a value from data based on index expressions.
///
/// Evaluates index expressions to determine which element to access.
/// If the validity expression evaluates to non-zero, returns the element at the calculated index.
/// Otherwise, returns zero.
pub(crate) fn get_index(
    data: &StwoData,
    (ind, val): &(Expression, Expression),
    stack: &mut Vec<i64>,
    index: usize,
) -> Fixed {
    if val.exec_single_var_stack(index, stack) != 0 {
        let i = ind.exec_single_var_stack(index, stack);
        data.0[i]
    } else {
        Fixed::zero()
    }
}

/// Computes the range (minimum and maximum) of all values in the given input tensors.
pub(crate) fn compute_range_from_srcs(srcs: &Vec<(InputTensor<'_>, ShapeTracker)>) -> Range {
    let mut min = Fixed(i64::MAX);
    let mut max = Fixed(i64::MIN);

    for (tensor, _) in srcs {
        if let Some(buffer) = get_buffer_from_tensor(tensor) {
            let (src_min, src_max) = buffer.min_max();

            if src_min.0 < min.0 {
                min = src_min;
            }
            if src_max.0 > max.0 {
                max = src_max;
            }
        }
    }

    Range(min, max)
}
