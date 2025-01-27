use jwalk::WalkDir;
use rand::{distr::Alphanumeric, Rng};
use rayon::iter::{ParallelBridge, ParallelIterator};
use windows::core::PCWSTR;

pub fn string_to_pcwstr(str: String) -> PCWSTR {
    PCWSTR(
        str.encode_utf16()
            .chain(Some(0))
            .collect::<Vec<u16>>()
            .as_mut_ptr(),
    )
}

pub fn random_name() -> String {
    let s: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    s
}

#[tauri::command(async)]
pub fn get_parallel_files(start_path: String) -> Vec<String> {
    WalkDir::new(start_path)
        .into_iter()
        .par_bridge()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect()
}
