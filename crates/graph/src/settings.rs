use luminair_air::preprocessed::PreProcessedColumn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CircuitSettings {
    pub lut_cols: Vec<Box<dyn PreProcessedColumn>>,
}

impl Clone for CircuitSettings {
    fn clone(&self) -> Self {
        CircuitSettings {
            lut_cols: self.lut_cols.iter().map(|col| col.clone_box()).collect(),
        }
    }
}
