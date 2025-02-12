use dump::{dump_modules_strings_from_process, dump_strings_from_process, ModuleStrings, Strings};
use process::Process;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tauri::AppHandle;

use crate::{
    emitter::global_emit,
    storage::{get_storage_as, set_storage_i},
    utils::filter_is_present,
};

pub mod dump;
pub mod process;
pub mod tests;

#[tauri::command(async)]
pub fn collect_modules_strings_from_cs2(app_handle: AppHandle) {
    let process = Process::find_by_name("cs2");
    if process.is_some() {
        global_emit("task_status_update", "start dumping strings from modules");
        let data = dump_modules_strings_from_process(process.unwrap());
        let summary: usize = data.par_iter().map(|item| item.values.len()).sum();
        match serde_json::to_value(&data) {
            Ok(val) => {
                set_storage_i(&app_handle, String::from("cs2_modules_strings"), val);
                set_storage_i(
                    &app_handle,
                    String::from("cs2_modules_strings_len"),
                    serde_json::to_value(summary).unwrap(),
                );
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }
    }
}

#[tauri::command(async)]
pub fn collect_strings_from_cs2(app_handle: AppHandle) {
    let process = Process::find_by_name("cs2");
    if process.is_some() {
        global_emit("task_status_update", "start dumping strings from regions");
        let data = dump_strings_from_process(process.unwrap());
        let summary: usize = data.par_iter().map(|item| item.values.len()).sum();
        match serde_json::to_value(&data) {
            Ok(val) => {
                set_storage_i(&app_handle, String::from("cs2_strings"), val);
                set_storage_i(
                    &app_handle,
                    String::from("cs2_strings_len"),
                    serde_json::to_value(summary).unwrap(),
                );
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }
    }
}

#[tauri::command(async)]
pub fn find_strings(app_handle: AppHandle, filter: String) -> Vec<String> {
    let modules_strings = get_storage_as::<Vec<ModuleStrings>>(&app_handle, "cs2_modules_strings");
    let strings = get_storage_as::<Vec<Strings>>(&app_handle, "cs2_strings");

    if modules_strings.is_some() && strings.is_some() {
        let modules_strings = modules_strings.unwrap();
        let strings = strings.unwrap();
        let mut response = vec![];

        response.extend(
            modules_strings
                .par_iter()
                .flat_map(|item| {
                    item.values
                        .par_iter()
                        .filter(|value| filter_is_present(&filter, value))
                        .cloned()
                })
                .collect::<Vec<String>>(),
        );

        response.extend(
            strings
                .par_iter()
                .flat_map(|item| {
                    item.values
                        .par_iter()
                        .filter(|value| filter_is_present(&filter, value))
                        .cloned()
                })
                .collect::<Vec<String>>(),
        );

        return response;
    }
    vec![]
}
