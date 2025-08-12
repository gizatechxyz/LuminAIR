use serde::{Deserialize, Serialize};
use sin::{SinLookup, SinLookupElements};
use stwo_prover::core::channel::Channel;

use crate::components::lookups::{
    exp2::{Exp2Lookup, Exp2LookupElements},
    log2::{Log2Lookup, Log2LookupElements},
    range_check::{RangeCheckLookup, RangeCheckLookupElements},
};

pub mod exp2;
pub mod log2;
pub mod range_check;
pub mod sin;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Lookups {
    pub sin: Option<SinLookup>,
    pub exp2: Option<Exp2Lookup>,
    pub log2: Option<Log2Lookup>,
    pub range_check: Option<RangeCheckLookup<1>>,
}

#[derive(Clone, Debug)]
pub struct LookupElements {
    pub sin: SinLookupElements,
    pub exp2: Exp2LookupElements,
    pub log2: Log2LookupElements,
    pub range_check: RangeCheckLookupElements,
}

impl LookupElements {
    pub fn draw(channel: &mut impl Channel) -> Self {
        Self {
            sin: SinLookupElements::draw(channel),
            exp2: Exp2LookupElements::draw(channel),
            log2: Log2LookupElements::draw(channel),
            range_check: RangeCheckLookupElements::draw(channel),
        }
    }
}
