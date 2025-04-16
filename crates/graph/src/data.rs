use luminal::prelude::*;
use num_traits::Zero;
use numerair::Fixed;
use std::sync::Arc;

/// Represents tensor data in a form compatible with Stwo.
#[derive(Clone, Debug)]
pub(crate) struct StwoData(pub(crate) Arc<Vec<Fixed>>);

impl StwoData {
    /// Creates a new `StwoData` instance from a slice of `f32` values.
    pub(crate) fn from_f32(data: &[f32]) -> Self {
        let mut fixed_data = Vec::with_capacity(data.len());
        for d in data {
            fixed_data.push(Fixed::from_f64(*d as f64));
        }

        StwoData(Arc::new(fixed_data))
    }

    /// Converts the fixed point data back to a vector of `f32` values.
    pub(crate) fn to_f32(&self) -> Vec<f32> {
        let mut float_data = Vec::with_capacity(self.0.len());

        for &d in self.0.iter() {
            float_data.push(d.to_f64() as f32);
        }

        float_data
    }

    /// Returns both minimum and maximum values in the data
    pub(crate) fn min_max(&self) -> (Fixed, Fixed) {
        if self.0.is_empty() {
            return (Fixed::zero(), Fixed::zero());
        }

        let mut min_val = self.0[0];
        let mut max_val = self.0[0];

        for &val in self.0.iter().skip(1) {
            if val.0 < min_val.0 {
                min_val = val;
            }
            if val.0 > max_val.0 {
                max_val = val;
            }
        }

        (min_val, max_val)
    }
}

/// Implementation of the `Data` trait for `StwoData`, allowing it to be used
/// within the Luminal framework's tensor system.
impl Data for StwoData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
