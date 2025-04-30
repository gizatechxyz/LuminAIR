use std::fmt;

use luminair_air::preprocessed::{LUTLayout, PreProcessedColumn};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CircuitSettings {
    pub lut_cols: Vec<Box<dyn PreProcessedColumn>>,
    pub lookup_tables: LUTs,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct LUTs {
    pub sin: Option<LUTLayout>,
}

impl Clone for CircuitSettings {
    fn clone(&self) -> Self {
        CircuitSettings {
            lut_cols: self.lut_cols.iter().map(|col| col.clone_box()).collect(),
            lookup_tables: self.lookup_tables.clone(),
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
