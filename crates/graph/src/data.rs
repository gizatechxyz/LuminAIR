use luminal::prelude::*;
use numerair::Fixed;
use std::sync::Arc;

/// Data structure for STWO operations using fixed-point arithmetic
/// 
/// Wraps a vector of fixed-point values with a runtime scale for STARK proving
#[derive(Clone, Debug)]
pub(crate) struct StwoData {
    pub(crate) data: Arc<Vec<Fixed>>,
    pub(crate) scale: u32,
}

impl StwoData {
    /// Creates a new StwoData from a slice of f32 values
    /// 
    /// Converts each f32 value to fixed-point representation with the specified scale
    pub(crate) fn from_f32(data: &[f32], scale: u32) -> Self {
        let fixed_data = data
            .iter()
            .map(|&d| Fixed::from_f64(d as f64, scale))
            .collect::<Vec<_>>();

        StwoData {
            data: Arc::new(fixed_data),
            scale,
        }
    }

    /// Converts the fixed-point data back to f32 values
    /// 
    /// Returns a vector of f32 values converted from the internal fixed-point representation
    pub(crate) fn to_f32(&self) -> Vec<f32> {
        self.data.iter().map(|&d| d.to_f64() as f32).collect()
    }

    /// Finds the minimum and maximum values in the data
    /// 
    /// Returns a tuple of (min, max) fixed-point values, or (0, 0) if empty
    pub(crate) fn min_max(&self) -> (Fixed, Fixed) {
        if self.data.is_empty() {
            return (Fixed::zero(self.scale), Fixed::zero(self.scale));
        }

        let first = self.data[0];
        self.data
            .iter()
            .skip(1)
            .fold((first, first), |(min_val, max_val), &val| {
                (
                    if val.value < min_val.value { val } else { min_val },
                    if val.value > max_val.value { val } else { max_val },
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
