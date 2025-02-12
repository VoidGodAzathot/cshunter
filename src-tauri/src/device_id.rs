use device_id::DeviceId;
use net_id::NetId;

pub mod device_id;
pub mod net_id;
pub mod shuffle;

#[tauri::command]
pub fn get_device_id() -> Option<String> {
    match DeviceId::generate() {
        Ok(device_id) => {
            return Some(device_id.to_string());
        }

        Err(_) => {}
    };

    None
}

#[tauri::command(async)]
pub async fn get_ip_addr() -> Option<String> {
    NetId::get_ip_addr().await
}
