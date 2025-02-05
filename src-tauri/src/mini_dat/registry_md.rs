use std::io::{Bytes, Read};
use windows_registry::{Type, CURRENT_USER, LOCAL_MACHINE};

use crate::utils::{known_folder_in_path, rot13};

use super::mini_dat::{MiniDat, MiniDatEmployee, MiniDatWrapper};

pub struct SevenZip {}
pub struct WinRar {}
pub struct UserAssist {}
pub struct Radar {}

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
                    let result: Vec<Option<Vec<MiniDat>>> = keys.map(|str_key| {
                        match key.open(format!("{}\\Count", str_key)) {
                            Ok(key) => {
                                let values = key.values();

                                if let Ok(values) = values {
                                    return Some(values.map(|value| UserAssist::new_instance(known_folder_in_path(rot13(&value.0)))).collect());
                                }
                            },

                            Err(e) => {
                                if cfg!(dev) {
                                    println!("{e:?}");
                                }
                            }
                        }

                        None
                    }).collect();

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

        match LOCAL_MACHINE.open("SOFTWARE\\Microsoft\\RADAR\\HeapLeakDetection\\DiagnosedApplications") {
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

        match LOCAL_MACHINE.open("SOFTWARE\\Microsoft\\RADAR\\HeapLeakDetection\\ReflectionApplications") {
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

fn bytes_to_string(bytes: Bytes<&[u8]>, remove_null_char: bool) -> String {
    let bytes: Vec<u8> = bytes.filter_map(|byte| byte.ok()).collect();

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
