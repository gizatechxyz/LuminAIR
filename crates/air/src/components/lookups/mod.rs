use serde::{Deserialize, Serialize};
use sin::{SinLookup, SinLookupElements};
use stwo_prover::core::channel::Channel;

pub mod sin;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Lookups {
    pub sin: Option<SinLookup>,
}

#[derive(Clone, Debug)]
pub struct LookupElements {
    pub sin: SinLookupElements,
}

impl LookupElements {
    pub fn draw(channel: &mut impl Channel) -> Self {
        Self {
            sin: SinLookupElements::draw(channel),
        }
    }
}
