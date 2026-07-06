use std::path::PathBuf;
use serde::{de::DeserializeOwned, Serialize};
use crate::helper::fshelper;

pub fn save_json<T: Serialize>(path: &PathBuf, data: &T) -> Result<(), String> {
    fshelper::ensure_parent_dir(path)?;
    let serialized = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fshelper::write_string_to_file(path, &serialized)?;
    Ok(())
}

pub fn load_json<T: DeserializeOwned>(path: &PathBuf) -> Result<Option<T>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fshelper::read_file_to_string(path)?;
    let data = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(Some(data))
}