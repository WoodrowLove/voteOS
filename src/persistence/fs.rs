//! File-backed persistence for domain stores.
//!
//! Simple, explicit, inspectable JSON files.
//! One file per domain registry partition.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, de::DeserializeOwned};

/// Persistence result type.
pub type PersistResult<T> = Result<T, PersistError>;

/// Explicit persistence errors — never silent.
#[derive(Debug)]
pub enum PersistError {
    IoError(String),
    SerializationError(String),
    DeserializationError(String),
    CorruptFile(String),
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO_ERROR: {}", e),
            Self::SerializationError(e) => write!(f, "SERIALIZATION_ERROR: {}", e),
            Self::DeserializationError(e) => write!(f, "DESERIALIZATION_ERROR: {}", e),
            Self::CorruptFile(e) => write!(f, "CORRUPT_FILE: {}", e),
        }
    }
}

/// Snapshot of a DomainStore's state — what gets serialized to disk.
#[derive(Serialize, serde::Deserialize, Debug)]
pub struct StoreSnapshot<T> {
    pub prefix: String,
    pub counter: u64,
    pub records: HashMap<String, T>,
}

/// Save a store snapshot to a JSON file.
pub fn save_snapshot<T: Serialize>(
    path: &Path, snapshot: &StoreSnapshot<T>,
) -> PersistResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PersistError::IoError(format!("mkdir: {}", e)))?;
    }

    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| PersistError::SerializationError(e.to_string()))?;

    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, &json)
        .map_err(|e| PersistError::IoError(format!("write: {}", e)))?;
    fs::rename(&tmp_path, path)
        .map_err(|e| PersistError::IoError(format!("rename: {}", e)))?;

    Ok(())
}

/// Load a store snapshot from a JSON file.
/// Returns None if the file does not exist (normal for first run).
pub fn load_snapshot<T: DeserializeOwned>(
    path: &Path,
) -> PersistResult<Option<StoreSnapshot<T>>> {
    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(path)
        .map_err(|e| PersistError::IoError(format!("read: {}", e)))?;

    if json.trim().is_empty() {
        return Err(PersistError::CorruptFile("empty file".to_string()));
    }

    let snapshot: StoreSnapshot<T> = serde_json::from_str(&json)
        .map_err(|e| PersistError::DeserializationError(format!("{}: {}", path.display(), e)))?;

    Ok(Some(snapshot))
}

/// Get the default data directory for persisted state.
pub fn default_data_dir() -> PathBuf {
    PathBuf::from("data")
}
