use serde::{Deserialize, Serialize};
use sin::{SinLookup, SinLookupElements};
use exp2::Exp2Lookup;
use stwo_prover::core::channel::Channel;

pub mod sin;
pub mod exp2;

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
    pub exp2: exp2::Exp2LookupElements,
}

impl LookupElements {
    /// Draws all necessary lookup-specific interaction elements from the channel.
    pub fn draw(channel: &mut impl Channel) -> Self {
        Self {
            sin: SinLookupElements::draw(channel),
            exp2: exp2::Exp2LookupElements::draw(channel),
        }
    }
}