use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use crate::lookups::Lookups;
use luminair_utils::LuminairError;
use serde::{Deserialize, Serialize};

/// Configuration settings for LuminAIR circuit generation and proving
/// 
/// Contains all the necessary parameters and lookup table configurations
/// needed to generate and verify STARK proofs
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct CircuitSettings {
    /// Lookup table configurations for non-linear operations
    pub lookups: Lookups,
}

impl CircuitSettings {
    // --- Serde Binary ---
    /// Serializes the circuit settings to bincode format
    /// 
    /// Returns a byte vector containing the serialized settings
    pub fn to_bincode(&self) -> Result<Vec<u8>, LuminairError> {
        bincode::serialize(self).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to serialize proof to bincode: {}",
                e
            ))
        })
    }

    /// Deserializes circuit settings from bincode format
    /// 
    /// Takes a byte slice and returns the deserialized CircuitSettings
    pub fn from_bincode(data: &[u8]) -> Result<Self, LuminairError> {
        bincode::deserialize(data).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to deserialize proof from bincode: {}",
                e
            ))
        })
    }

    /// Writes the circuit settings to a bincode file
    /// 
    /// Serializes the settings and writes them to the specified file path
    pub fn to_bincode_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LuminairError> {
        let data = self.to_bincode()?;
        std::fs::write(path, data).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to write bincode file: {}", e))
        })
    }

    /// Reads circuit settings from a bincode file
    /// 
    /// Reads the file at the specified path and deserializes the settings
    pub fn from_bincode_file<P: AsRef<Path>>(path: P) -> Result<Self, LuminairError> {
        let data = std::fs::read(path).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to read bincode file: {}", e))
        })?;
        Self::from_bincode(&data)
    }

    // --- Serde JSON ---
    /// Serializes the circuit settings to JSON format
    /// 
    /// Returns a pretty-printed JSON string representation of the settings
    pub fn to_json(&self) -> Result<String, LuminairError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to serialize proof to JSON: {}", e))
        })
    }

    /// Deserializes circuit settings from JSON format
    /// 
    /// Takes a JSON string and returns the deserialized CircuitSettings
    pub fn from_json(json: &str) -> Result<Self, LuminairError> {
        serde_json::from_str(json).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to deserialize proof from JSON: {}",
                e
            ))
        })
    }

    /// Writes the circuit settings to a JSON file
    /// 
    /// Serializes the settings to pretty-printed JSON and writes them to the specified file path
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

    /// Reads circuit settings from a JSON file
    /// 
    /// Reads the file at the specified path and deserializes the JSON settings
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
