use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use crate::lookups::Lookups;
use luminair_utils::LuminairError;
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

impl CircuitSettings {
    // --- Serde Binary ---
    pub fn to_bincode(&self) -> Result<Vec<u8>, LuminairError> {
        bincode::serialize(self).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to serialize proof to bincode: {}",
                e
            ))
        })
    }

    pub fn from_bincode(data: &[u8]) -> Result<Self, LuminairError> {
        bincode::deserialize(data).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to deserialize proof from bincode: {}",
                e
            ))
        })
    }

    pub fn to_bincode_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LuminairError> {
        let data = self.to_bincode()?;
        std::fs::write(path, data).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to write bincode file: {}", e))
        })
    }

    pub fn from_bincode_file<P: AsRef<Path>>(path: P) -> Result<Self, LuminairError> {
        let data = std::fs::read(path).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to read bincode file: {}", e))
        })?;
        Self::from_bincode(&data)
    }

    // --- Serde JSON ---
    pub fn to_json(&self) -> Result<String, LuminairError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to serialize proof to JSON: {}", e))
        })
    }

    pub fn from_json(json: &str) -> Result<Self, LuminairError> {
        serde_json::from_str(json).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to deserialize proof from JSON: {}",
                e
            ))
        })
    }

    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LuminairError> {
        let file = File::create(path).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to create file: {}", e))
        })?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer_pretty(&mut writer, self).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to write proof to JSON file: {}", e))
        })?;

        writer.flush().map_err(|e| {
            LuminairError::SerializationError(format!("Failed to flush writer: {}", e))
        })?;

        Ok(())
    }

    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, LuminairError> {
        let file = File::open(path).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to open file: {}", e))
        })?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to read proof from JSON file: {}", e))
        })
    }
}
