use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use serde::{de::DeserializeOwned, Serialize};

// SAVES anything to a JSON file
pub fn save_json<T: Serialize>(path: &PathBuf, data: &T) -> Result<(), String> {
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    let file = File::create(path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file, data).map_err(|e| e.to_string())?;
    Ok(())
}

// LOADS anything from a JSON file, returns None if file doesn't exist
pub fn load_json<T: DeserializeOwned>(path: &PathBuf) -> Result<Option<T>, String> {
    if !path.exists() {
        return Ok(None);
    }
    
    let file = File::open(path).map_err(|e| e.to_string())?;
    let data = serde_json::from_reader(file).map_err(|e| e.to_string())?;
    Ok(Some(data))
}