use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use crate::LuminairProof;
use luminair_utils::LuminairError;
pub use luminair_utils::{JsonDeserialization, JsonSerialization};
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleHasher;

impl JsonSerialization for LuminairProof<Blake2sMerkleHasher> {
    fn to_json(&self) -> Result<String, LuminairError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to serialize proof to JSON: {}", e))
        })
    }

    fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LuminairError> {
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
}

impl JsonDeserialization for LuminairProof<Blake2sMerkleHasher> {
    fn from_json(json: &str) -> Result<Self, LuminairError> {
        serde_json::from_str(json).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to deserialize proof from JSON: {}",
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
            LuminairError::SerializationError(format!("Failed to read proof from JSON file: {}", e))
        })
    }
}
