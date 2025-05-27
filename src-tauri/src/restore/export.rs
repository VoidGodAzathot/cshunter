use std::{collections::HashMap, io::Write, sync::Arc};

use flate2::{write::GzEncoder, Compression};
use tauri::{AppHandle, Manager};
use base64::{engine::general_purpose::STANDARD, Engine};

use crate::storage::Storage;

#[tauri::command(async)]
pub fn export_all_data(app_handle: AppHandle) -> Result<String, String> {
    let storage = app_handle.state::<Arc<Storage>>();

    let map: HashMap<String, serde_json::Value> = storage
        .internal
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().clone()))
        .collect();

    let json_str = serde_json::to_string(&map)
        .map_err(|e| format!("Error while serialization: {}", e))?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(json_str.as_bytes())
        .map_err(|e| format!("Compression error: {}", e))?;
    let compressed = encoder
        .finish()
        .map_err(|e| format!("Finalizing compression error: {}", e))?;

    Ok(STANDARD.encode(&compressed))
}