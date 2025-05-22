use crate::components::lookups::Lookups;
use serde::{Deserialize, Serialize};

/// Configuration settings for a LuminAIR circuit.
///
/// Holds information derived from the graph structure that is necessary for
/// trace generation and proving, such as lookup table configurations.
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct CircuitSettings {
    /// Lookup table configurations required by the circuit.
    pub lookups: Lookups,
}
