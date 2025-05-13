use luminal::prelude::*;
use num_traits::Zero;
use numerair::Fixed;
use std::sync::Arc;

/// A wrapper for tensor data using `Fixed` point numbers, suitable for the STWO prover.
///
/// Stores data as an `Arc<Vec<Fixed>>` for efficient sharing.
#[derive(Clone, Debug)]
pub(crate) struct StwoData(pub(crate) Arc<Vec<Fixed>>);

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
    pub(crate) fn min_max(&self) -> (Fixed, Fixed) {
        self.0.iter().fold(
            (Fixed::zero(), Fixed::zero()),
            |(min_val, max_val), &val| match self.0.len() {
                0 => (Fixed::zero(), Fixed::zero()),
                _ if min_val.0 == 0 && max_val.0 == 0 => (val, val),
                _ => (
                    if val.0 < min_val.0 { val } else { min_val },
                    if val.0 > max_val.0 { val } else { max_val },
                ),
            },
        )
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
