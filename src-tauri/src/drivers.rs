use drivers_info::{collect_drivers_info, DriverInfo};

pub mod drivers_info;

#[tauri::command]
pub fn get_drivers_info() -> Vec<DriverInfo> {
    collect_drivers_info()
}