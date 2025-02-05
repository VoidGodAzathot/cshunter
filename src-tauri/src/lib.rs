use std::sync::Mutex;

use analyzer::{
    create_analyzer_context, create_analyzer_context_from_url, generate_context, save_context,
};
use browser::{
    get_browser_cache_data, get_browser_download_data, get_browser_visit_data,
    get_supported_browsers,
};
use device_id::{get_device_id, get_ip_addr};
use mini_dat::{collect_mini_dat, get_mini_dat_info};
use steam::{get_steam_accounts_history, is_vac_present};
use storage::{get_all_storage, get_storage, set_storage, Storage};
use tauri::{Manager, WindowEvent};
use usn_journal::{get_all_volumes, get_usn_journal_records};
use utils::{get_parallel_files, open_explorer, open_url, run_main_window_and_close_preload};
use vmdetect::is_vm;

pub mod analyzer;
pub mod browser;
pub mod device_id;
pub mod mini_dat;
pub mod steam;
pub mod storage;
pub mod usn_journal;
pub mod utils;
pub mod vmdetect;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|w, e| {
            match e {
                WindowEvent::Destroyed {  .. } => {
                    w.app_handle().webview_windows().iter().for_each(|window| {
                        let _ = window.1.close();
                    });
                }
                _ => {}
            };
        })
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
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
            generate_context,
            run_main_window_and_close_preload,
            get_storage,
            set_storage,
            get_all_storage,
            save_context,
            open_explorer,
            open_url,
            is_vm,
            collect_mini_dat,
            get_mini_dat_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
