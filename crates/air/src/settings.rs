use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use crate::lookups::Lookups;
use luminair_utils::{JsonDeserialization, JsonSerialization, LuminairError};
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

// Implementation for CircuitSettings
impl JsonSerialization for CircuitSettings {
    fn to_json(&self) -> Result<String, LuminairError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to serialize circuit settings to JSON: {}",
                e
            ))
        })
    }

    fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LuminairError> {
        let file = File::create(path).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to create file: {}", e))
        })?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer_pretty(&mut writer, self).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to write circuit settings to JSON file: {}",
                e
            ))
        })?;

        writer.flush().map_err(|e| {
            LuminairError::SerializationError(format!("Failed to flush writer: {}", e))
        })?;

        Ok(())
    }
}

impl JsonDeserialization for CircuitSettings {
    fn from_json(json: &str) -> Result<Self, LuminairError> {
        serde_json::from_str(json).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to deserialize circuit settings from JSON: {}",
                e
            ))
        })
    }

    fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, LuminairError> {
        let file = File::open(path).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to open file: {}", e))
        })?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to read circuit settings from JSON file: {}",
                e
            ))
        })
    }
}
