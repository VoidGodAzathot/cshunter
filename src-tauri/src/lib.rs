use usn_journal::{get_all_volumes, get_usn_journal_records};

pub mod usn_journal;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_usn_journal_records,
            get_all_volumes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
