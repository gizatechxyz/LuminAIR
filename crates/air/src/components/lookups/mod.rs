use serde::{Deserialize, Serialize};
use sin::{SinLookup, SinLookupElements};
use stwo::core::channel::Channel;

use crate::components::lookups::{
    exp2::{Exp2Lookup, Exp2LookupElements},
    log2::{Log2Lookup, Log2LookupElements},
    range_check::{RangeCheckLookup, RangeCheckLookupElements},
};

pub mod exp2;
pub mod log2;
pub mod range_check;
pub mod sin;

/// Collection of all lookup table structures
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Lookups {
    /// Optional sine lookup table configuration
    pub sin: Option<SinLookup>,
    /// Optional exponential base-2 lookup table configuration
    pub exp2: Option<Exp2Lookup>,
    /// Optional logarithm base-2 lookup table configuration
    pub log2: Option<Log2Lookup>,
    /// Optional range check lookup table configuration
    pub range_check: Option<RangeCheckLookup<1>>,
}

/// Collection of all lookup table interaction elements
#[derive(Clone, Debug)]
pub struct LookupElements {
    /// Interaction elements for sine lookup table
    pub sin: SinLookupElements,
    /// Interaction elements for exponential base-2 lookup table
    pub exp2: Exp2LookupElements,
    /// Interaction elements for logarithm base-2 lookup table
    pub log2: Log2LookupElements,
    /// Interaction elements for range check lookup table
    pub range_check: RangeCheckLookupElements,
}

impl LookupElements {
    /// Draws lookup elements from the given channel
    pub fn draw(channel: &mut impl Channel) -> Self {
        Self {
            sin: SinLookupElements::draw(channel),
            exp2: Exp2LookupElements::draw(channel),
            log2: Log2LookupElements::draw(channel),
            range_check: RangeCheckLookupElements::draw(channel),
        }
    }
}
