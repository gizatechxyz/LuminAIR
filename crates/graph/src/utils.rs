use std::any::{Any, TypeId};

use crate::data::StwoData;
use luminair_air::{preprocessed::Range, DEFAULT_FP_SCALE};
use luminal::prelude::*;
use num_traits::Zero;
use numerair::Fixed;

pub(crate) fn is<T: Any>(type_id: TypeId) -> bool {
    type_id == TypeId::of::<T>()
}

pub(crate) fn get_buffer_from_tensor<'a>(tensor: &'a InputTensor) -> Option<&'a StwoData> {
    tensor.borrowed().downcast_ref::<StwoData>()
}

pub(crate) fn get_index(
    data: &StwoData,
    (ind, val): &(Expression, Expression),
    stack: &mut Vec<i64>,
    index: usize,
) -> Fixed<DEFAULT_FP_SCALE> {
    if val.exec_single_var_stack(index, stack) != 0 {
        let i = ind.exec_single_var_stack(index, stack);
        data.0[i]
    } else {
        Fixed::zero()
    }
}

pub(crate) fn compute_padded_range_from_srcs(srcs: &Vec<(InputTensor<'_>, ShapeTracker)>) -> Range {
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

    buffer_range(Range(min, max))
}

fn buffer_range(range: Range) -> Range {
    // TODO (@raphaelDkhn): make it parametizeable maybe.
    const RANGE_MARGIN: f64 = 0.10;

    let min = range.0.to_f64();
    let max = range.1.to_f64();
    let span = max - min;

    let delta = span * RANGE_MARGIN;
    let low = Fixed::from_f64(min - delta);
    let high = Fixed::from_f64(max + delta);

    Range(low, high)
}
