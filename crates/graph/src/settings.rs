use std::fmt;

use luminair_air::{components::lookups::Layout, preprocessed::PreProcessedColumn};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CircuitSettings {
    pub lookup_layouts: LookupLayouts,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LookupLayouts {
    pub sin: Option<Layout>,
}
