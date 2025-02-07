use std::{collections::BTreeMap, convert::TryInto, ptr::addr_of_mut};

use chrono::{TimeZone, Utc};
use shellbag::{ShellBagPath, ShellBags, ShellItem};
use serde::{Deserialize, Serialize};
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{
            ERROR_INVALID_DATA, ERROR_MORE_DATA, ERROR_NOT_SUPPORTED, ERROR_NO_MORE_ITEMS,
            FILETIME, WIN32_ERROR,
        },
        System::{
            Registry::{
                RegEnumKeyExW, RegQueryValueExW, HKEY, HKEY_USERS, REG_DWORD, REG_SZ,
                REG_VALUE_TYPE,
            },
            SystemInformation::{GetVersionExA, OSVERSIONINFOA},
        },
    },
};

mod modern;
use modern::read_shell_bags_modern;
pub mod shellbag;

pub mod err;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShellBagTimeline {
    pub action : ShellBagAction,
    pub time : String,
    pub numeric_time : i64,
    pub folder : String,
    pub user : String
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShellBagAction {
    Modified,
    Access,
    Created
}

pub unsafe fn read_shell_bags_user(user_id: &str) -> Result<ShellBags, WIN32_ERROR> {
    use modern::read_shell_bags_user;
    if is_win_xp() {
        read_shell_bags_xp()
    } else {
        read_shell_bags_user(user_id)
    }
}

pub unsafe fn read_all_shell_bags() -> Result<BTreeMap<String, ShellBags>, WIN32_ERROR> {
    let mut bag_list: BTreeMap<String, ShellBags> = BTreeMap::new();
    if is_win_xp() {
        //TODO:
    } else {
        let mut counter = 0;
        loop {
            let key_name = match enumerate_keys(HKEY_USERS, counter) {
                Ok(v) => v,
                Err(e) => {
                    if e == ERROR_NO_MORE_ITEMS {
                        break;
                    }
                    return Err(e);
                }
            };
            counter += 1;
            if !key_name.starts_with("S-") {
                continue;
            }
            let shell_item = match read_shell_bags_user(&key_name) {
                Ok(v) => v,
                Err(_) => continue,
            };
            bag_list.insert(key_name, shell_item);
        }
    }
    Ok(bag_list)
}

pub unsafe fn read_shell_bags() -> Result<ShellBags, WIN32_ERROR> {
    if is_win_xp() {
        read_shell_bags_xp()
    } else {
        read_shell_bags_modern()
    }
}

pub unsafe fn read_shell_bags_xp() -> Result<ShellBags, WIN32_ERROR> {
    Err(ERROR_NOT_SUPPORTED)
}

pub fn parse_bag_with_timeline() -> Vec<ShellBagTimeline> {
    unsafe {
        let empty_path = String::new();
        match read_all_shell_bags() {
            Ok(list) => {
                let mut timeline : BTreeMap<i64, Vec<ShellBagTimeline>> = BTreeMap::new();
                
                for (user, bags) in &list {
                    let mut path_association : BTreeMap<&ShellBagPath, String> = BTreeMap::new();
                    for (path, bag) in &bags.ntuser.list {
                        let parent_path = if path.0.len() > 0 {
                            let mut pth = path.0.clone();
                            pth.pop();
                            ShellBagPath(pth)
                        }else {
                            ShellBagPath(Vec::new())
                        };
                        let parent_path = match path_association.get(&parent_path) {
                            Some(path_name) => path_name,
                            None => &empty_path
                        };
                        match &bag.1 {
                            ShellItem::Folder(v) => {
                                path_association.insert(path, format!("{}\\{}",parent_path,v.name));
                            },
                            ShellItem::Volume(v) => {
                                path_association.insert(path, format!("{}\\{}",parent_path,v.name));
                            },
                            ShellItem::File(v) => {
                                path_association.insert(path, format!("{}\\{}",parent_path,v.long_name));
                            },
                            ShellItem::Network(v) =>{
                                path_association.insert(path, format!("{}\\{}",parent_path,v.location));
                            },
                            ShellItem::Unknown(_) => {
                                path_association.insert(path, format!("{}\\?",parent_path));
                            },
                        }
                    }
                    for (path, bag) in &bags.ntuser.list {
                        let parent_path = match path_association.get(path) {
                            Some(path_name) => path_name,
                            None => &empty_path
                        };
                        match &bag.1 {
                            ShellItem::File(v) => {
                                let mut time_vec = match timeline.remove(&v.a_time) {
                                    Some(v) => v,
                                    None => Vec::with_capacity(4)
                                };
                                let time = Utc.timestamp_opt(v.a_time, 0 ).unwrap().to_string();
                                time_vec.push(ShellBagTimeline { action: ShellBagAction::Access, numeric_time : v.a_time, folder: parent_path.clone(), user: user[..].to_string(), time});
                                timeline.insert(v.a_time, time_vec);

                                let mut time_vec = match timeline.remove(&v.m_time) {
                                    Some(v) => v,
                                    None => Vec::with_capacity(4)
                                };
                                let time = Utc.timestamp_opt(v.m_time, 0).unwrap().to_string();
                                time_vec.push(ShellBagTimeline { action: ShellBagAction::Modified, numeric_time : v.m_time, folder: parent_path.clone(), user: user[..].to_string(), time });
                                timeline.insert(v.m_time, time_vec);

                                let mut time_vec = match timeline.remove(&v.c_time) {
                                    Some(v) => v,
                                    None => Vec::with_capacity(4)
                                };
                                let time = Utc.timestamp_opt(v.c_time,0 ).unwrap().to_string();
                                time_vec.push(ShellBagTimeline { action: ShellBagAction::Created, numeric_time : v.c_time, folder: parent_path.clone(), user: user[..].to_string(), time });
                                timeline.insert(v.c_time, time_vec);
                            },
                            _ => {}
                        }
                    }
                    for (user, bags) in &list {
                        let mut path_association : BTreeMap<&ShellBagPath, String> = BTreeMap::new();
                        for (path, bag) in &bags.usr_class.list {
                            let parent_path = if path.0.len() > 0 {
                                let mut pth = path.0.clone();
                                pth.pop();
                                ShellBagPath(pth)
                            }else {
                                ShellBagPath(Vec::new())
                            };
                            let parent_path = match path_association.get(&parent_path) {
                                Some(path_name) => path_name,
                                None => &empty_path
                            };
                            match &bag.1 {
                                ShellItem::Folder(v) => {
                                    path_association.insert(path, format!("{}\\{}",parent_path,v.name));
                                },
                                ShellItem::Volume(v) => {
                                    path_association.insert(path, format!("{}\\{}",parent_path,v.name));
                                },
                                ShellItem::File(v) => {
                                    path_association.insert(path, format!("{}\\{}",parent_path,v.long_name));
                                },
                                ShellItem::Network(v) =>{
                                    path_association.insert(path, format!("{}\\{}",parent_path,v.location));
                                },
                                ShellItem::Unknown(_) => {
                                    path_association.insert(path, format!("{}\\?",parent_path));
                                },
                            }
                        }
                        for (path, bag) in &bags.usr_class.list {
                            let parent_path = match path_association.get(path) {
                                Some(path_name) => path_name,
                                None => &empty_path
                            };
                            match &bag.1 {
                                ShellItem::File(v) => {
                                    let mut time_vec = match timeline.remove(&v.a_time) {
                                        Some(v) => v,
                                        None => Vec::with_capacity(4)
                                    };
                                    let time = Utc.timestamp_opt(v.a_time, 0).unwrap().to_string();
                                    time_vec.push(ShellBagTimeline { action: ShellBagAction::Access, numeric_time : v.a_time, folder: parent_path.clone(), user: user[..].to_string(), time});
                                    timeline.insert(v.a_time, time_vec);
    
                                    let mut time_vec = match timeline.remove(&v.m_time) {
                                        Some(v) => v,
                                        None => Vec::with_capacity(4)
                                    };
                                    let time = Utc.timestamp_opt(v.m_time, 0).unwrap().to_string();
                                    time_vec.push(ShellBagTimeline { action: ShellBagAction::Modified, numeric_time : v.m_time, folder: parent_path.clone(), user: user[..].to_string(), time });
                                    timeline.insert(v.m_time, time_vec);
    
                                    let mut time_vec = match timeline.remove(&v.c_time) {
                                        Some(v) => v,
                                        None => Vec::with_capacity(4)
                                    };
                                    let time = Utc.timestamp_opt(v.c_time, 0).unwrap().to_string();
                                    time_vec.push(ShellBagTimeline { action: ShellBagAction::Created, numeric_time : v.c_time, folder: parent_path.clone(), user: user[..].to_string(), time });
                                    timeline.insert(v.c_time, time_vec);
                                },
                                _ => {}
                            }
                        }
                    }
                }
                let mut timeline_vec : Vec<ShellBagTimeline> = Vec::with_capacity(4096);
                for (_time, events) in timeline {
                    for event in events {
                        timeline_vec.push(event);
                    }
                }
                return timeline_vec;
            },
            Err(e) => {
                println!("ERROR: {:?}",e);
            }
        }
    }

    vec![]
}

unsafe fn is_win_xp() -> bool {
    let mut info = OSVERSIONINFOA::default();
    if !GetVersionExA(&mut info).is_ok() {
        return false;
    }
    info.dwMajorVersion <= 5
}

pub fn to_pwstr(val: &str) -> Vec<u16> {
    let mut val = val.encode_utf16().collect::<Vec<u16>>();
    val.push(0);
    val
}

pub fn from_pwstr(val: &[u16]) -> String {
    String::from_utf16_lossy(val)
}

pub unsafe fn read_reg_u32_value(hkey: HKEY, name: &str) -> Result<u32, WIN32_ERROR> {
    let value_name = to_pwstr(name);
    let mut capacity: u32 = 10_000;
    let mut readed_data = [0; 10_000];
    let mut data_type: REG_VALUE_TYPE = REG_DWORD;
    let reserved: *const u32 = std::ptr::null();
    let readed = RegQueryValueExW(
        hkey,
        PCWSTR(value_name.as_ptr()),
        Some(reserved as _),
        Some(addr_of_mut!(data_type)),
        Some(readed_data.as_mut_ptr()),
        Some(addr_of_mut!(capacity)),
    );
    if readed.is_err() {
        return Err(readed);
    }
    if capacity == 4 {
        let data: [u8; 4] = match readed_data[0..4].try_into() {
            Ok(v) => v,
            Err(_) => return Err(ERROR_INVALID_DATA),
        };
        return Ok(u32::from_ne_bytes(data));
    }
    Err(ERROR_INVALID_DATA)
}

pub unsafe fn read_reg_sz_value(hkey: HKEY, name: &str) -> Result<String, WIN32_ERROR> {
    let value_name = to_pwstr(name);
    let mut capacity: u32 = 10_000;
    let mut readed_data = [0; 10_000];
    let mut data_type: REG_VALUE_TYPE = REG_SZ;
    let reserved: *const u32 = std::ptr::null();
    let readed = RegQueryValueExW(
        hkey,
        PCWSTR(value_name.as_ptr()),
        Some(reserved as _),
        Some(addr_of_mut!(data_type)),
        Some(readed_data.as_mut_ptr()),
        Some(addr_of_mut!(capacity)),
    );
    if readed.is_err() {
        return Err(readed);
    }
    if capacity == 0 {
        return Ok(String::new());
    }
    let mut u16_vec: Vec<u16> = readed_data[0..capacity as usize]
        .chunks(2)
        .map(|v| (v[1] as u16) << 8 | v[0] as u16)
        .collect();
    let _ = u16_vec.pop(); //Ends with 00
    return Ok(String::from_utf16_lossy(&u16_vec));
}

pub fn vec_with_capacity(capacity: usize) -> Vec<u8> {
    vec![0; capacity as usize]
}

pub unsafe fn read_reg_bin_value(hkey: HKEY, name: &str) -> Result<Vec<u8>, WIN32_ERROR> {
    let value_name = to_pwstr(name);
    loop {
        let mut capacity: u32 = 10_000;
        let mut readed_data = vec_with_capacity(capacity as usize);
        let mut data_type: REG_VALUE_TYPE = REG_SZ;
        let reserved: *const u32 = std::ptr::null();
        let readed = RegQueryValueExW(
            hkey,
            PCWSTR(value_name.as_ptr()),
            Some(reserved as _),
            Some(addr_of_mut!(data_type)),
            Some(readed_data.as_mut_ptr()),
            Some(addr_of_mut!(capacity)),
        );
        if readed == ERROR_MORE_DATA {
            continue;
        } else {
            if readed.is_err() {
                return Err(readed);
            }
        }
        readed_data.resize(capacity as usize, 0);
        return Ok(readed_data);
    }
}

pub unsafe fn enumerate_keys(hkey: HKEY, pos: u32) -> Result<String, WIN32_ERROR> {
    let reserved: *const u32 = std::ptr::null();
    let mut key_name_capacity: u32 = 1024;
    let mut key_name_buff = [0; 1024];

    let mut key_class_capacity: u32 = 1024;
    let mut key_class_buff = [0; 1024];

    let mut last_written: FILETIME = FILETIME::default();

    let enumerated = RegEnumKeyExW(
        hkey,
        pos,
        Some(PWSTR(key_name_buff.as_mut_ptr())),
        addr_of_mut!(key_name_capacity),
        Some(reserved as _),
        Some(PWSTR(key_class_buff.as_mut_ptr())),
        Some(&mut key_class_capacity),
        Some(addr_of_mut!(last_written)),
    );
    if enumerated.is_err() {
        return Err(enumerated);
    }
    Ok(from_pwstr(&key_name_buff[0..key_name_capacity as usize]))
}
