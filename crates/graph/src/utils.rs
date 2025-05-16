use std::any::{Any, TypeId};

use crate::data::StwoData;
use luminair_air::{preprocessed::Range, DEFAULT_FP_SCALE};
use luminal::prelude::*;
use num_traits::Zero;
use numerair::Fixed;

/// Generic helper function to check if a given `TypeId` corresponds to the type `T`.
pub(crate) fn is<T: Any>(type_id: TypeId) -> bool {
    type_id == TypeId::of::<T>()
}

/// Safely attempts to downcast a Luminal `InputTensor` to a reference to `StwoData`.
/// Returns `None` if the tensor does not contain `StwoData`.
pub(crate) fn get_buffer_from_tensor<'a>(tensor: &'a InputTensor) -> Option<&'a StwoData> {
    tensor.borrowed().downcast_ref::<StwoData>()
}

/// Retrieves a value from `StwoData` using Luminal expressions for indexing and validity.
///
/// This function evaluates the `ind` expression to get the target index and the `val`
/// expression to check if the access is valid for the given `index` (often representing
/// the current position in a symbolic execution trace).
/// If valid, it returns the data at the computed index; otherwise, it returns `Fixed::zero()`.
/// Requires the execution `stack` for evaluating expressions.
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

/// Computes the combined value range across multiple source tensors, adding padding.
///
/// Iterates through the provided source tensors (`srcs`), extracts their `StwoData`,
/// finds the overall minimum and maximum values, and then applies padding using `buffer_range`.
/// This is used to determine the necessary range for lookup tables.
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

/// Expands a `Range` by a fixed percentage margin (currently 10%) on both ends.
///
/// This buffering helps ensure that lookup tables constructed based on observed ranges
/// during `gen_circuit_settings` can accommodate potential minor variations in values
/// encountered during actual trace generation.
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
