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
        let fixed_data = data.iter()
            .map(|&d| Fixed::from_f64(d as f64))
            .collect::<Vec<_>>();

        StwoData(Arc::new(fixed_data))
    }

    /// Converts the fixed point data back to a vector of `f32` values.
    pub(crate) fn to_f32(&self) -> Vec<f32> {
        self.0.iter()
            .map(|&d| d.to_f64() as f32)
            .collect()
    }

    /// Returns both minimum and maximum values in the data
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
