use luminair_air::DEFAULT_FP_SCALE;
use luminal::prelude::*;
use num_traits::Zero;
use numerair::Fixed;
use std::sync::Arc;

/// A wrapper for tensor data using `Fixed` point numbers, suitable for the STWO prover.
///
/// Stores data as an `Arc<Vec<Fixed>>` for efficient sharing.
#[derive(Clone, Debug)]
pub(crate) struct StwoData(pub(crate) Arc<Vec<Fixed<DEFAULT_FP_SCALE>>>);

impl StwoData {
    /// Creates a new `StwoData` instance by converting a slice of `f32` values to `Fixed` point.
    pub(crate) fn from_f32(data: &[f32]) -> Self {
        let fixed_data = data
            .iter()
            .map(|&d| Fixed::from_f64(d as f64))
            .collect::<Vec<_>>();

        StwoData(Arc::new(fixed_data))
    }

    /// Converts the internal `Fixed` point data back to a vector of `f32` values.
    pub(crate) fn to_f32(&self) -> Vec<f32> {
        self.0.iter().map(|&d| d.to_f64() as f32).collect()
    }

    /// Finds the minimum and maximum `Fixed` point values within the data.
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

/// Enables `StwoData` to be used as a generic data container within Luminal's tensor operations.
impl Data for StwoData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
