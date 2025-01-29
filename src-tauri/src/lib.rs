use std::sync::Mutex;

use analyzer::{create_analyzer_context, create_analyzer_context_from_url, generate_context_from_folder, run_analyzer};
use browser::{
    get_browser_cache_data, get_browser_download_data, get_browser_visit_data,
    get_supported_browsers,
};
use device_id::{get_device_id, get_ip_addr};
use steam::{get_steam_accounts_history, is_vac_present};
use storage::{get_all_storage, get_storage, set_storage, Storage};
use tauri::Manager;
use usn_journal::{get_all_volumes, get_usn_journal_records};
use utils::{get_parallel_files, run_main_window_and_close_preload};

pub mod analyzer;
pub mod browser;
pub mod device_id;
pub mod steam;
pub mod tests;
pub mod usn_journal;
pub mod utils;
pub mod emit;
pub mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(Mutex::new(Storage::default()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_usn_journal_records,
            get_all_volumes,
            get_device_id,
            get_ip_addr,
            get_parallel_files,
            get_steam_accounts_history,
            is_vac_present,
            get_supported_browsers,
            get_browser_cache_data,
            get_browser_download_data,
            get_browser_visit_data,
            create_analyzer_context,
            create_analyzer_context_from_url,
            generate_context_from_folder,
            run_analyzer,
            run_main_window_and_close_preload,
            get_storage,
            set_storage,
            get_all_storage
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
