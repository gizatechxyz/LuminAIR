use luminair_air::DEFAULT_FP_SCALE;
use luminal::prelude::*;
use num_traits::Zero;
use numerair::Fixed;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct StwoData(pub(crate) Arc<Vec<Fixed<DEFAULT_FP_SCALE>>>);

impl StwoData {
    pub(crate) fn from_f32(data: &[f32]) -> Self {
        let fixed_data = data
            .iter()
            .map(|&d| Fixed::from_f64(d as f64))
            .collect::<Vec<_>>();

        StwoData(Arc::new(fixed_data))
    }

    pub(crate) fn to_f32(&self) -> Vec<f32> {
        self.0.iter().map(|&d| d.to_f64() as f32).collect()
    }

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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
