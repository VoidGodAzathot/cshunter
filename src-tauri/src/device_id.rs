use device_id::DeviceId;
use net_id::NetId;

pub mod device_id;
pub mod net_id;
pub mod shuffle;
pub mod tests;

#[tauri::command]
pub fn get_device_id() -> String {
    match DeviceId::generate() {
        Ok(device_id) => {
            return device_id.to_string();
        },

        Err(_) => { }
    };

    String::from("undefined")
}

#[tauri::command(async)]
pub async fn get_ip_addr() -> Option<String> {
    NetId::get_ip_addr().await
}