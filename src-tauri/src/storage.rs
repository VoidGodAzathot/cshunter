use std::{collections::HashMap, sync::Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Window};

#[derive(Serialize, Deserialize)]
struct StorageItem {
    name: String,
    value: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
struct StorageUpdate {
    name: String,
}

#[tauri::command]
pub fn set_storage(app_handle: AppHandle, name: String, value: serde_json::Value, window: Window) {
    let binding = app_handle.state::<Mutex<Storage>>();
    let mut storage = binding.lock().unwrap();
    storage.internal.insert(name.clone(), value);
    let _ = window.emit("storage_update", StorageUpdate { name: name });
}

#[tauri::command]
pub fn get_storage(app_handle: AppHandle, name: String) -> Option<serde_json::Value> {
    let binding = app_handle.state::<Mutex<Storage>>();
    let storage = binding.lock().unwrap();
    storage.internal.get(&name).cloned()
}

#[tauri::command]
pub fn get_all_storage(app_handle: AppHandle) -> HashMap<String, serde_json::Value> {
    let binding = app_handle.state::<Mutex<Storage>>();
    let storage = binding.lock().unwrap();
    storage.internal.clone()
}

pub struct Storage {
    internal: HashMap<String, serde_json::Value>,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            internal: HashMap::new(),
        }
    }
}
