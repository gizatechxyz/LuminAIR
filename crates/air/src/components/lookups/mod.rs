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

/// Container for configurations of all active lookup arguments in the AIR.
///
/// Each field is optional, present only if the corresponding lookup is used.
/// The contained struct (e.g., `SinLookup`) typically holds the `LookupLayout`
/// defining the LUT's structure and value ranges.
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Lookups {
    /// Configuration for the Sine lookup argument, if active.
    pub sin: Option<SinLookup>,
    /// Configuration for the Exp2 lookup argument, if active.
    pub exp2: Option<Exp2Lookup>,
    /// Configuration for the Log2 lookup argument, if active.
    pub log2: Option<Log2Lookup>,
    /// Configuration for the RangeCheck lookup argument, if active.
    pub range_check: Option<RangeCheckLookup<1>>,
}

/// Container for interaction elements specific to each lookup type.
///
/// These elements are drawn from the Fiat-Shamir channel and are used to build
/// the LogUp arguments that connect trace values to the preprocessed lookup tables.
#[derive(Clone, Debug)]
pub struct LookupElements {
    /// Interaction elements for the Sine lookup.
    pub sin: SinLookupElements,
    /// Interaction elements for the Exp2 lookup.
    pub exp2: Exp2LookupElements,
    /// Interaction elements for the Log2 lookup.
    pub log2: Log2LookupElements,
    /// Interaction elements for the RangeCheck lookup.
    pub range_check: RangeCheckLookupElements,
}

impl LookupElements {
    /// Draws all necessary lookup-specific interaction elements from the channel.
    pub fn draw(channel: &mut impl Channel) -> Self {
        Self {
            sin: SinLookupElements::draw(channel),
            exp2: Exp2LookupElements::draw(channel),
            log2: Log2LookupElements::draw(channel),
            range_check: RangeCheckLookupElements::draw(channel),
        }
    }
}
