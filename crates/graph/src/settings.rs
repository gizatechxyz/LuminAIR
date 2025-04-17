use luminair_air::preprocessed::PreProcessedColumn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CircuitSettings {
    pub lut_cols: Vec<Box<dyn PreProcessedColumn>>,
}
