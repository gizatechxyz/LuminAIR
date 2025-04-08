use std::any::{Any, TypeId};

use crate::data::StwoData;
use luminal::prelude::*;
use num_traits::Zero;
use numerair::Fixed;
use stwo_prover::core::fields::m31::BaseField;

/// Checks if a `TypeId` matches the type `T`.
pub(crate) fn is<T: Any>(type_id: TypeId) -> bool {
    type_id == TypeId::of::<T>()
}

/// Extracts the StwoData reference from an InputTensor.
pub(crate) fn get_buffer_from_tensor<'a>(tensor: &'a InputTensor) -> &'a StwoData {
    &tensor.borrowed().downcast_ref::<StwoData>().unwrap()
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

/// Trait to extend Fixed with exp2 functionality
pub trait FixedExp2 {
    /// Calculate 2^x for Fixed type, returning result and remainder
    fn exp2(&self) -> (Self, BaseField) where Self: Sized;
}

impl FixedExp2 for Fixed {
    fn exp2(&self) -> (Self, BaseField) {
        // Convert fixed-point to f64 for calculation
        let x = self.to_f64();
        
        // Calculate 2^x
        let result = 2.0_f64.powf(x);
        
        // Convert back to fixed point
        let fixed_result = Fixed::from_f64(result);
        
        // Calculate remainder (simplified - in a real implementation this would be more precise)
        // For now we're using a dummy remainder. In a production system this would represent 
        // the precise remainder after fixed-point conversion
        let remainder = BaseField::from_u32_unchecked(0);
        
        (fixed_result, remainder)
    }
}

// Since BaseField (M31) doesn't have to_m31() method (it's already an M31 value),
// we'll add this extension trait to make it work with our code
pub trait BaseFieldExt {
    fn to_m31(&self) -> BaseField;
}

impl BaseFieldExt for BaseField {
    fn to_m31(&self) -> BaseField {
        // Simply return the input as it's already a BaseField
        *self 
    }
}
