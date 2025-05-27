use std::{collections::HashMap, fs, io::Read, sync::Arc};

use flate2::read::GzDecoder;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use base64::{engine::general_purpose::STANDARD, Engine};

use crate::storage::Storage;

#[tauri::command(async)]
pub fn import_all_data(app_handle: AppHandle, file: String) -> Result<(), String> {
    let storage = app_handle.state::<Arc<Storage>>();

    let encoded = fs::read_to_string(&file)
        .map_err(|e| format!("Failed to read file '{}': {}", file, e))?;

    let compressed = STANDARD
        .decode(&encoded)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut json_str = String::new();
    decoder
        .read_to_string(&mut json_str)
        .map_err(|e| format!("Decompression error: {}", e))?;

    let parsed: HashMap<String, Value> = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    storage.internal.clear();
    for (key, value) in parsed {
        storage.internal.insert(key, value);
    }

    Ok(())
}