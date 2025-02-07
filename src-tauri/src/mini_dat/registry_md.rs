use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use shellbags::{parse_bag_with_timeline, ShellBagAction};
use std::{
    ffi::OsString,
    io::{Bytes, Read},
    os::windows::ffi::OsStringExt,
};
use windows::Win32::Storage::FileSystem::QueryDosDeviceW;
use windows_registry::{Type, CURRENT_USER, LOCAL_MACHINE};

use crate::utils::{get_current_username_in_sid, known_folder_in_path, rot13, string_to_pcwstr};

use super::mini_dat::{MiniDat, MiniDatEmployee, MiniDatWrapper};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShellBagView {
    pub path: String,
    pub name: String,
    pub timestamp: i64,
    pub action: ShellBagViewAction,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShellBagViewAction {
    DELETE,
    MODIFY,
    ACCESS,
    CREATE,
}

pub struct SevenZip {}
pub struct WinRar {}
pub struct UserAssist {}
pub struct Radar {}
pub struct AppCompatCache {}
pub struct Bam {}
pub struct AppSwitched {}
pub struct Shellbag {}

impl MiniDatWrapper for SevenZip {
    fn new_instance(value: String) -> MiniDat {
        MiniDat {
            value: value,
            id: "seven_zip",
        }
    }
}

impl MiniDatEmployee<MiniDat> for SevenZip {
    fn run() -> Vec<MiniDat> {
        match CURRENT_USER.open("SOFTWARE\\7-Zip\\Compression") {
            Ok(key) => {
                let compression_history = key.get_value("ArcHistory");

                if let Ok(compression_history) = compression_history {
                    if compression_history.ty() == Type::Bytes {
                        return bytes_to_string(compression_history.bytes(), false)
                            .split('\0')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(|path| SevenZip::new_instance(String::from(path)))
                            .collect();
                    }
                } else {
                    if cfg!(dev) {
                        println!("{:?}", compression_history.err().unwrap());
                    }
                }
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        vec![]
    }
}

impl MiniDatWrapper for Shellbag {
    fn new_instance(value: String) -> MiniDat {
        MiniDat {
            value: value,
            id: "shellbag",
        }
    }
}

impl MiniDatEmployee<MiniDat> for Shellbag {
    fn run() -> Vec<MiniDat> {
        read_shellbag()
            .iter()
            .map(|item| {
                Shellbag::new_instance(
                    serde_json::to_string(item).unwrap_or(String::from("undefined")),
                )
            })
            .collect()
    }
}

impl MiniDatWrapper for AppSwitched {
    fn new_instance(value: String) -> MiniDat {
        MiniDat {
            value: value,
            id: "app_switched",
        }
    }
}

impl MiniDatEmployee<MiniDat> for AppSwitched {
    fn run() -> Vec<MiniDat> {
        match CURRENT_USER.open(
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FeatureUsage\\AppSwitched",
        ) {
            Ok(key) => {
                let values = key.values();

                if let Ok(values) = values {
                    return values
                        .map(|v| AppSwitched::new_instance(known_folder_in_path(v.0)))
                        .collect();
                }
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        vec![]
    }
}

impl MiniDatWrapper for WinRar {
    fn new_instance(value: String) -> MiniDat {
        MiniDat {
            value: value,
            id: "winrar",
        }
    }
}

impl MiniDatEmployee<MiniDat> for WinRar {
    fn run() -> Vec<MiniDat> {
        match CURRENT_USER.open("SOFTWARE\\WinRar\\ArcHistory") {
            Ok(key) => {
                let values = key.values();

                if let Ok(values) = values {
                    return values
                        .map(|v| WinRar::new_instance(bytes_to_string(v.1.bytes(), true)))
                        .collect();
                }
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        vec![]
    }
}

impl MiniDatWrapper for UserAssist {
    fn new_instance(value: String) -> MiniDat {
        MiniDat {
            value: value,
            id: "user_assist",
        }
    }
}

impl MiniDatEmployee<MiniDat> for UserAssist {
    fn run() -> Vec<MiniDat> {
        match CURRENT_USER
            .open("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\UserAssist")
        {
            Ok(key) => {
                let keys = key.keys();

                if let Ok(keys) = keys {
                    let result: Vec<Option<Vec<MiniDat>>> = keys
                        .map(|str_key| {
                            match key.open(format!("{}\\Count", str_key)) {
                                Ok(key) => {
                                    let values = key.values();

                                    if let Ok(values) = values {
                                        return Some(
                                            values
                                                .map(|value| {
                                                    UserAssist::new_instance(known_folder_in_path(
                                                        rot13(&value.0),
                                                    ))
                                                })
                                                .collect(),
                                        );
                                    }
                                }

                                Err(e) => {
                                    if cfg!(dev) {
                                        println!("{e:?}");
                                    }
                                }
                            }

                            None
                        })
                        .collect();

                    let mut response: Vec<MiniDat> = vec![];

                    for m in result {
                        if m.is_some() {
                            let mut m = m.unwrap();
                            response.append(&mut m);
                        }
                    }

                    return response;
                }
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        vec![]
    }
}

impl MiniDatWrapper for Radar {
    fn new_instance(value: String) -> MiniDat {
        MiniDat {
            value: value,
            id: "radar",
        }
    }
}

impl MiniDatEmployee<MiniDat> for Radar {
    fn run() -> Vec<MiniDat> {
        let mut values = vec![];

        match LOCAL_MACHINE
            .open("SOFTWARE\\Microsoft\\RADAR\\HeapLeakDetection\\DiagnosedApplications")
        {
            Ok(key) => {
                let keys = key.keys();

                if let Ok(keys) = keys {
                    values.append(&mut keys.map(|k| Radar::new_instance(k)).collect());
                }
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        match LOCAL_MACHINE
            .open("SOFTWARE\\Microsoft\\RADAR\\HeapLeakDetection\\ReflectionApplications")
        {
            Ok(key) => {
                let keys = key.keys();

                if let Ok(keys) = keys {
                    values.append(&mut keys.map(|k| Radar::new_instance(k)).collect());
                }
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        values
    }
}

impl MiniDatWrapper for AppCompatCache {
    fn new_instance(value: String) -> MiniDat {
        MiniDat {
            value: value,
            id: "app_compat_cache",
        }
    }
}

impl MiniDatEmployee<MiniDat> for AppCompatCache {
    fn run() -> Vec<MiniDat> {
        match LOCAL_MACHINE
            .open("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\AppCompatCache")
        {
            Ok(key) => match key.get_value("AppCompatCache") {
                Ok(value) => {
                    let mut response = vec![];

                    if value.ty() == Type::Bytes {
                        let vec_bytes = bytes_to_vec_u8(value.bytes());
                        let offset_to_records =
                            i32::from_le_bytes(vec_bytes[0..4].try_into().unwrap()) as usize;
                        let mut offset = offset_to_records;

                        while offset < vec_bytes.len() {
                            if offset + 4 > vec_bytes.len() {
                                break;
                            }

                            offset += 4;

                            offset += 4;

                            if offset + 4 > vec_bytes.len() {
                                break;
                            }
                            let _ = u32::from_le_bytes(
                                vec_bytes[offset..offset + 4]
                                    .try_into()
                                    .expect("Slice with incorrect length"),
                            );
                            offset += 4;

                            if offset + 2 > vec_bytes.len() {
                                break;
                            }
                            let ce_path_size = u16::from_le_bytes(
                                vec_bytes[offset..offset + 2]
                                    .try_into()
                                    .expect("Slice with incorrect length"),
                            );
                            offset += 2;

                            let path_byte_count = ce_path_size as usize;
                            if offset + path_byte_count > vec_bytes.len() {
                                break;
                            }
                            let ce_path_bytes = &vec_bytes[offset..offset + path_byte_count];
                            offset += path_byte_count;

                            let utf16_units: Vec<u16> = ce_path_bytes
                                .chunks_exact(2)
                                .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
                                .collect();
                            let path = OsString::from_wide(&utf16_units)
                                .to_string_lossy()
                                .replace(r"\??\", "");

                            response.push(AppCompatCache::new_instance(path));

                            if offset + 8 > vec_bytes.len() {
                                break;
                            }
                            let _ = i64::from_le_bytes(
                                vec_bytes[offset..offset + 8]
                                    .try_into()
                                    .expect("Slice with incorrect length"),
                            );
                            offset += 8;

                            if offset + 4 > vec_bytes.len() {
                                break;
                            }
                            let data_size = i32::from_le_bytes(
                                vec_bytes[offset..offset + 4]
                                    .try_into()
                                    .expect("Slice with incorrect length"),
                            );
                            offset += 4;

                            let data_size_usize = data_size as usize;
                            if offset + data_size_usize > vec_bytes.len() {
                                break;
                            }
                            let ce_data = &vec_bytes[offset..offset + data_size_usize];
                            offset += data_size_usize;

                            let _ = if ce_data.len() >= 4 {
                                let last_val = i32::from_le_bytes(
                                    ce_data[ce_data.len() - 4..]
                                        .try_into()
                                        .expect("Slice with incorrect length"),
                                );
                                last_val == 1
                            } else {
                                false
                            };
                        }
                    }

                    return response;
                }

                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}");
                    }
                }
            },

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        vec![]
    }
}

impl MiniDatWrapper for Bam {
    fn new_instance(value: String) -> MiniDat {
        MiniDat {
            value: value,
            id: "bam",
        }
    }
}

impl MiniDatEmployee<MiniDat> for Bam {
    fn run() -> Vec<MiniDat> {
        let current_username_in_sid = get_current_username_in_sid();

        if let Some(current_username_in_sid) = current_username_in_sid {
            match LOCAL_MACHINE.open(format!(
                "SYSTEM\\ControlSet001\\Services\\bam\\State\\UserSettings\\{}",
                current_username_in_sid
            )) {
                Ok(key) => match key.values() {
                    Ok(values) => {
                        let mut response = vec![];

                        for value in values {
                            if value.0.starts_with(r"\Device\") {
                                response.push(Bam::new_instance(
                                    replace_device_path_with_drive_letter(&value.0),
                                ));
                            }
                        }

                        return response;
                    }

                    Err(e) => {
                        if cfg!(dev) {
                            println!("{e:?}");
                        }
                    }
                },

                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}");
                    }
                }
            }
        }

        vec![]
    }
}

pub fn read_shellbag() -> Vec<ShellBagView> {
    let mut response = parse_bag_with_timeline()
        .par_iter()
        .map(|item| ShellBagView {
            name: String::from(
                item.folder
                    .clone()
                    .split("\\")
                    .last()
                    .unwrap_or("undefined"),
            ),
            path: replace_start_path(item.folder.clone()),
            timestamp: item.numeric_time,
            action: (if item.folder.contains("?") {
                ShellBagViewAction::DELETE
            } else {
                if item.action == ShellBagAction::Access {
                    ShellBagViewAction::ACCESS
                } else if item.action == ShellBagAction::Created {
                    ShellBagViewAction::CREATE
                } else {
                    ShellBagViewAction::MODIFY
                }
            }),
        })
        .collect::<Vec<ShellBagView>>();

    remove_duplicates::<ShellBagView>(&mut response, |item1, item2| {
        item1.path == item2.path && item1.action == item2.action
    });

    response
}

fn remove_duplicates<T>(vec: &mut Vec<T>, filter: fn(&T, &T) -> bool)
where
    T: Clone,
{
    let mut unique = Vec::new();
    vec.retain(|item| {
        if unique.iter().any(|unique_item| filter(unique_item, item)) {
            false
        } else {
            unique.push(item.clone());
            true
        }
    });
}

fn replace_start_path(path: String) -> String {
    path
        .replace("\\LIBRARIES\\", "")
        .replace("\\MY_DOCUMENTS\\", "")
        .replace("\\NETWORK\\", "")
        .replace("\\RECYCLE_BIN\\", "")
        .replace("\\INTERNET_EXPLORER\\", "")
        .replace("\\UNKNOWN\\", "")
        .replace("\\MY_GAMES\\", "")
        .replace("\\MY_COMPUTER\\", "")
        .replace("\\USERS\\", "")
        .replace(":\\\\", ":\\")
}

pub fn replace_device_path_with_drive_letter(path: &str) -> String {
    let parts: Vec<&str> = path.split('\\').collect();

    if parts.len() >= 3
        && parts[0].is_empty()
        && parts[1] == "Device"
        && parts[2].starts_with("HarddiskVolume")
    {
        let device_path = format!("\\{}\\{}", parts[1], parts[2]);
        if let Some(drive_letter) = get_drive_letter(&device_path) {
            let rest_of_path = parts[3..].join("\\");
            return format!("{}\\{}", drive_letter, rest_of_path);
        }
    }

    path.to_string()
}

fn get_drive_letter(device_path: &str) -> Option<String> {
    let target_device = String::from(device_path);

    for drive in b'A'..=b'Z' {
        let drive_letter = format!("{}:", drive as char);
        let mut buffer = [0u16; 1024];

        unsafe {
            let success =
                QueryDosDeviceW(string_to_pcwstr(drive_letter.clone()), Some(&mut buffer));

            if success == 0 {
                continue;
            }

            let device = OsString::from_wide(&buffer)
                .into_string()
                .unwrap()
                .trim_end_matches('\0')
                .to_string();

            if device == target_device {
                return Some(drive_letter);
            }
        }
    }

    None
}

fn bytes_to_vec_u8(bytes: Bytes<&[u8]>) -> Vec<u8> {
    bytes.filter_map(|byte| byte.ok()).collect()
}

fn bytes_to_string(bytes: Bytes<&[u8]>, remove_null_char: bool) -> String {
    let bytes: Vec<u8> = bytes_to_vec_u8(bytes);

    let utf16_data: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    if remove_null_char {
        String::from_utf16_lossy(&utf16_data).replace("\0", "")
    } else {
        String::from_utf16_lossy(&utf16_data)
    }
}
