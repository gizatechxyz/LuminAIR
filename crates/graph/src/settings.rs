use luminair_air::components::lookups::Lookups;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CircuitSettings {
    pub lookups: Lookups,
}
