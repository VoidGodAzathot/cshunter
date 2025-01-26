use device_id::{get_device_id, get_ip_addr};
use usn_journal::{get_all_volumes, get_usn_journal_records};

pub mod usn_journal;
pub mod utils;
pub mod device_id;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_usn_journal_records,
            get_all_volumes,
            get_device_id,
            get_ip_addr
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
