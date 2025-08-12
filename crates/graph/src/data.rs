use luminair_air::DEFAULT_FP_SCALE;
use luminal::prelude::*;
use num_traits::Zero;
use numerair::Fixed;
use std::sync::Arc;

/// Data structure for STWO operations using fixed-point arithmetic
/// 
/// Wraps a vector of fixed-point values with a default scale for STARK proving
#[derive(Clone, Debug)]
pub(crate) struct StwoData(pub(crate) Arc<Vec<Fixed<DEFAULT_FP_SCALE>>>);

impl StwoData {
    /// Creates a new StwoData from a slice of f32 values
    /// 
    /// Converts each f32 value to fixed-point representation with the default scale
    pub(crate) fn from_f32(data: &[f32]) -> Self {
        let fixed_data = data
            .iter()
            .map(|&d| Fixed::from_f64(d as f64))
            .collect::<Vec<_>>();

        StwoData(Arc::new(fixed_data))
    }

    /// Converts the fixed-point data back to f32 values
    /// 
    /// Returns a vector of f32 values converted from the internal fixed-point representation
    pub(crate) fn to_f32(&self) -> Vec<f32> {
        self.0.iter().map(|&d| d.to_f64() as f32).collect()
    }

    /// Finds the minimum and maximum values in the data
    /// 
    /// Returns a tuple of (min, max) fixed-point values, or (0, 0) if empty
    pub(crate) fn min_max(&self) -> (Fixed<DEFAULT_FP_SCALE>, Fixed<DEFAULT_FP_SCALE>) {
        if self.0.is_empty() {
            return (Fixed::zero(), Fixed::zero());
        }

        let first = self.0[0];
        self.0
            .iter()
            .skip(1)
            .fold((first, first), |(min_val, max_val), &val| {
                (
                    if val.0 < min_val.0 { val } else { min_val },
                    if val.0 > max_val.0 { val } else { max_val },
                )
            })
    }
}

impl Data for StwoData {
    /// Returns a reference to the underlying data as Any
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Returns a mutable reference to the underlying data as Any
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
