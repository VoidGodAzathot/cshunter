use dashmap::DashMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Window};

#[derive(Clone, Serialize, Deserialize)]
struct StorageUpdate {
    name: String,
}

pub struct Storage {
    pub internal: DashMap<String, serde_json::Value>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            internal: DashMap::new(),
        }
    }
}

#[tauri::command]
pub fn set_storage(app_handle: AppHandle, name: String, value: serde_json::Value, window: Window) {
    let storage = app_handle.state::<Arc<Storage>>();
    storage.internal.insert(name.clone(), value);
    let _ = window.emit("storage_update", StorageUpdate { name });
}

#[tauri::command]
pub fn get_storage(app_handle: AppHandle, name: String) -> Option<serde_json::Value> {
    let storage = app_handle.state::<Arc<Storage>>();
    storage
        .internal
        .get(&name)
        .map(|entry| entry.value().clone())
}

#[tauri::command]
pub fn get_all_storage(app_handle: AppHandle) -> HashMap<String, serde_json::Value> {
    let storage = app_handle.state::<Arc<Storage>>();
    storage
        .internal
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().clone()))
        .collect()
}

pub fn set_storage_value(app_handle: &AppHandle, name: String, value: serde_json::Value) {
    let storage = app_handle.state::<Arc<Storage>>();
    storage.internal.insert(name.clone(), value);
}

pub fn get_storage_value<T>(app_handle: &AppHandle, name: &str) -> Option<T>
where
    T: DeserializeOwned,
{
    app_handle
        .state::<Arc<Storage>>()
        .internal
        .get(name)
        .and_then(
            |entry| match serde_json::from_value(entry.value().clone()) {
                Ok(val) => Some(val),
                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}");
                    }
                    None
                }
            },
        )
}
