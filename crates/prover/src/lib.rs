use ::serde::{Deserialize, Serialize};
use luminair_air::{LuminairClaim, LuminairInteractionClaim};
use luminair_utils::LuminairError;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use stwo::core::{prover::StarkProof, vcs::ops::MerkleHasher};

pub mod prover;

/// Complete LuminAIR proof containing claim, interaction claim, and STARK proof
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairProof<H: MerkleHasher> {
    pub claim: LuminairClaim,
    pub interaction_claim: LuminairInteractionClaim,
    pub proof: StarkProof<H>,
}

impl LuminairProof<Blake2sMerkleHasher> {
    // --- Serde Binary ---
    /// Serializes the proof to bincode format
    pub fn to_bincode(&self) -> Result<Vec<u8>, LuminairError> {
        bincode::serialize(self).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to serialize proof to bincode: {}",
                e
            ))
        })
    }

    /// Deserializes a proof from bincode format
    pub fn from_bincode(data: &[u8]) -> Result<Self, LuminairError> {
        bincode::deserialize(data).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to deserialize proof from bincode: {}",
                e
            ))
        })
    }

    /// Writes the proof to a bincode file
    pub fn to_bincode_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LuminairError> {
        let data = self.to_bincode()?;
        std::fs::write(path, data).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to write bincode file: {}", e))
        })
    }

    /// Reads a proof from a bincode file
    pub fn from_bincode_file<P: AsRef<Path>>(path: P) -> Result<Self, LuminairError> {
        let data = std::fs::read(path).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to read bincode file: {}", e))
        })?;
        Self::from_bincode(&data)
    }

    // --- Serde JSON ---
    /// Serializes the proof to JSON format
    pub fn to_json(&self) -> Result<String, LuminairError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            LuminairError::SerializationError(format!("Failed to serialize proof to JSON: {}", e))
        })
    }

    /// Deserializes a proof from JSON format
    pub fn from_json(json: &str) -> Result<Self, LuminairError> {
        serde_json::from_str(json).map_err(|e| {
            LuminairError::SerializationError(format!(
                "Failed to deserialize proof from JSON: {}",
                e
            ))
        })
    }

    /// Writes the proof to a JSON file
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

    /// Reads a proof from a JSON file
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
