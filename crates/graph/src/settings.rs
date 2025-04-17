use std::fmt;

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

impl fmt::Debug for CircuitSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print the IDs of the LUT columns for readability
        f.debug_struct("CircuitSettings")
            .field(
                "lut_cols",
                &self.lut_cols.iter().map(|col| col.id()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
