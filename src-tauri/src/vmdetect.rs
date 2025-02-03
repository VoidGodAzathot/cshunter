use vmdetect::VMDetector;

pub mod vmdetect;

#[tauri::command]
pub fn is_vm() -> bool {
    let vmdetector = VMDetector::new();
    unsafe { vmdetector.is_vm() }
}