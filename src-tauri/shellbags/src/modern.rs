use std::{collections::BTreeMap, convert::TryInto, ptr::{addr_of_mut, null_mut}};

use chrono::{Utc, TimeZone};
use uuid::Uuid;
use windows::{Win32::{System::Registry::{RegOpenKeyW, HKEY_CLASSES_ROOT, HKEY, RegQueryValueExW, REG_BINARY, REG_VALUE_TYPE, HKEY_CURRENT_USER, HKEY_USERS}, Foundation::{WIN32_ERROR, ERROR_NO_MORE_ITEMS}}, core::PCWSTR};

use crate::{enumerate_keys, read_reg_bin_value, read_reg_sz_value, read_reg_u32_value, shellbag::{NodeSlot, ShellBagList, ShellBagPath, ShellBags, ShellFileItem, ShellFolderItem, ShellItem, ShellNetworkItem, ShellVolumeItem, WindowBagInfo}, to_pwstr};

pub unsafe fn read_shell_bags_modern() -> Result<ShellBags, WIN32_ERROR>{
    let shell_key = get_shell_key_usrclass(HKEY_CLASSES_ROOT)?;
    let bags_key = get_bags_key(shell_key)?;
    let mru_bag_key = get_mrubag_key(shell_key)?;
    let mut bags_usrclass = read_all_bags(bags_key)?;
    read_mru_bag(mru_bag_key, &mut bags_usrclass, &vec![])?;
    let shell_key = get_shell_key_ntuser(HKEY_CURRENT_USER)?;
    let bags_key = get_bags_key(shell_key)?;
    let mru_bag_key = get_mrubag_key(shell_key)?;
    let mut bags_ntuser = read_all_bags(bags_key)?;
    read_mru_bag(mru_bag_key, &mut bags_ntuser, &vec![])?;
    Ok(ShellBags {
        ntuser : bags_ntuser,
        usr_class : bags_usrclass
    })
}

pub unsafe fn read_shell_bags_user(user_id : &str) -> Result<ShellBags, WIN32_ERROR>{
    let user_key = open_key(HKEY_USERS, user_id)?;
    let user_class_key = open_key(user_key, &format!("SOFTWARE\\Classes"))?;
    let shell_key = get_shell_key_usrclass(user_class_key)?;
    let bags_key = get_bags_key(shell_key)?;
    let mru_bag_key = get_mrubag_key(shell_key)?;
    let mut bags_usrclass = read_all_bags(bags_key)?;
    read_mru_bag(mru_bag_key, &mut bags_usrclass, &vec![])?;
    let shell_key = get_shell_key_ntuser(user_key)?;
    let bags_key = get_bags_key(shell_key)?;
    let mru_bag_key = get_mrubag_key(shell_key)?;
    let mut bags_ntuser = read_all_bags(bags_key)?;
    read_mru_bag(mru_bag_key, &mut bags_ntuser, &vec![])?;
    Ok(ShellBags {
        ntuser : bags_ntuser,
        usr_class : bags_usrclass
    })
}

pub unsafe fn get_shell_key_usrclass(hkey : HKEY) -> Result<HKEY, WIN32_ERROR> {
    let mut shell_key = HKEY(null_mut());
    let shell_key_str = to_pwstr("Local Settings\\Software\\Microsoft\\Windows\\Shell");
    let opened = RegOpenKeyW(hkey, PCWSTR(shell_key_str.as_ptr()), &mut shell_key);
    if opened.is_err() {
        return Err(opened);
    }
    Ok(shell_key)
}

pub unsafe fn get_shell_key_ntuser(hkey: HKEY) -> Result<HKEY, WIN32_ERROR> {
    let mut shell_key = HKEY(null_mut());
    let shell_key_str = to_pwstr("Software\\Microsoft\\Windows\\Shell");
    let opened = RegOpenKeyW(hkey, PCWSTR(shell_key_str.as_ptr()), &mut shell_key);
    if opened.is_err() {
        return Err(opened);
    }
    Ok(shell_key)
}

pub unsafe fn open_key(hkey : HKEY, name : &str) -> Result<HKEY, WIN32_ERROR> {
    let mut bags_key = HKEY(null_mut());
    let bags_key_str = to_pwstr(name);
    let opened = RegOpenKeyW(hkey, PCWSTR(bags_key_str.as_ptr()), &mut bags_key);
    if opened.is_err() {
        return Err(opened);
    }
    Ok(bags_key)
}

pub unsafe fn get_bags_key(shell_key : HKEY) -> Result<HKEY, WIN32_ERROR> {
    open_key(shell_key, "Bags")
}

pub unsafe fn get_mrubag_key(shell_key : HKEY) -> Result<HKEY, WIN32_ERROR> {
    open_key(shell_key, "BagMRU")
}

pub unsafe fn read_all_bags(bags_key : HKEY) -> Result<ShellBagList, WIN32_ERROR> {
    let mut list = ShellBagList::new();
    let mut counter = 0;
    loop {
        let key_name = match enumerate_keys(bags_key, counter) {
            Ok(v) => v,
            Err(e) => {
                if e == ERROR_NO_MORE_ITEMS {
                    break;
                }
                return Err(e);
            }
        };
        let node_slot = match key_name.parse::<u32>() {
            Ok(v) => v,
            Err(_) => {
                counter += 1;
                continue;
            }
        };
        match read_windows_in_bag(bags_key, node_slot) {
            Ok(v) => {
                list.node_slots.insert(NodeSlot(node_slot), v);
            },
            Err(err) => {
                if err != ERROR_NO_MORE_ITEMS {
                    println!("Error: {}",err.0);
                }
                counter += 1;
                continue;
            }
        };
        counter += 1;
    }
    
    Ok(list)
}

pub unsafe fn read_windows_in_bag(bags_key : HKEY, node_slot : u32) -> Result<BTreeMap<String, WindowBagInfo>, WIN32_ERROR> {
    let mut list_of_windows = BTreeMap::new();
    let node_key = open_key(bags_key, &format!("{}",node_slot))?;
    let mut counter = 0;
    loop {
        let window_name = match enumerate_keys(node_key, counter) {
            Ok(v) => v,
            Err(e) => {
                if e == ERROR_NO_MORE_ITEMS {
                    break;
                }
                return Err(e);
            }
        };
        let mut info = WindowBagInfo::default();
        info.slot = node_slot;
        let subnode_key = open_key(node_key, &window_name)?;
        let uuid = match enumerate_keys(subnode_key, 0) {
            Ok(v) => v,
            Err(_e) => {
                counter += 1;
                continue;
            }
        };
        let uuid_key = open_key(subnode_key, &uuid)?;
        match read_reg_u32_value(uuid_key, "FFlags") {
            Ok(v) => {info.f_flags = v;},
            Err(err) => {
                println!("FFlags error: {}", err.0);
            }
        };
        match read_reg_u32_value(uuid_key, "GroupByDirection") {
            Ok(v) => {info.group_by_direction = v;},
            Err(_) => {}
        };
        match read_reg_u32_value(uuid_key, "GroupByKey:PID") {
            Ok(v) => {info.group_by_key_pid = v;},
            Err(_) => {}
        };
        match read_reg_u32_value(uuid_key, "GroupView") {
            Ok(v) => {info.group_view = v;},
            Err(_) => {}
        };
        match read_reg_u32_value(uuid_key, "IconSize") {
            Ok(v) => {info.icon_size = v;},
            Err(_) => {}
        };
        match read_reg_u32_value(uuid_key, "LogicalViewMode") {
            Ok(v) => {info.logical_view_mode = v;},
            Err(_) => {}
        };
        match read_reg_u32_value(uuid_key, "Mode") {
            Ok(v) => {info.mode = v;},
            Err(_) => {}
        };
        match read_reg_u32_value(uuid_key, "Rev") {
            Ok(v) => {info.rev = v;},
            Err(_) => {}
        };
        match read_reg_sz_value(uuid_key, "Vid") {
            Ok(v) => {
                info.vid = Uuid::parse_str(&v[1..v.len() - 1]).unwrap_or_default().as_u128();
            },
            Err(_) => {}
        };
        list_of_windows.insert(window_name, info);
        counter += 1;
    }
    
    
    //Open <xxx>\Shell\<uuid>
   

   
    Ok(list_of_windows)
}

pub unsafe fn read_mru_bag(bag_mru_key : HKEY, mut data : &mut ShellBagList, parent_route : &Vec<u32>) -> Result<(),WIN32_ERROR> {
    let mru_list = get_mru_list(bag_mru_key)?;
    let mut mru_key_list = Vec::with_capacity(mru_list.len());
    let mut counter = 0;
    loop {
        let node_slot = match enumerate_keys(bag_mru_key, counter) {
            Ok(v) => match v.parse::<u32>() {
                Ok(v) => v,
                Err(_) => {
                    counter += 1;
                    continue;
                }
            },
            Err(_e) => break
        };
        mru_key_list.push(node_slot);
        counter += 1;
    }
    
    let mut anomaly = false;
    for (a,b) in mru_key_list.iter().zip(mru_list.iter()) {
        if *a != *b {
            anomaly = true;
            break;
        }
    }
    for element in &mru_list {
        // TODO: detect anomalies
        let mut element_route = parent_route.clone();
        element_route.push(*element);
        let (node_slot, node_value) = match read_reg_bin_value(bag_mru_key, &format!("{}",(*element))) {
            Ok(v) => {
                let item = parse_node_value(&v, &element_route);
                
                if let Some(item) = item {
                    let subnode_key = match open_key(bag_mru_key, &format!("{}",*element)) {
                        Ok(v) => v,
                        Err(e) => {
                            println!("Error opening folder item: {}", e.0);
                            continue;
                        }
                    };
                    let _ = read_mru_bag(subnode_key, &mut data, &element_route);
                    let node_slot = match read_reg_u32_value(subnode_key, "NodeSlot") {
                        Ok(v) => Some(v),
                        Err(_err) => {
                            None
                        }
                    };
                    (node_slot, item)
                }else{
                    continue;
                }
            },
            Err(e) => {
                println!("Error parsing item: {}", e.0);
                continue;
            }
        };
        data.list.insert(ShellBagPath(element_route), (node_slot, node_value));
    }
    if anomaly {
        data.mru_anomalies = Some((mru_list, mru_key_list));
    }
    Ok(())
}

pub fn parse_node_value(node_value : &Vec<u8>, element_route : &Vec<u32>) -> Option<ShellItem> {
    if node_value.len() < 2 {
        return None;
    }
    let entry_size = u16::from_le_bytes(node_value[0..2].try_into().unwrap_or_else(|_| [0,0]));
    if entry_size < 20 {
        return None;
    }
    let entry_type : u8 = node_value[2];
    let is_file = (entry_type & 0x70) == 0x30;
    if is_file {
        return Some(
            {
                // File
                let mut item = ShellFileItem::default();
                item.fflags = u16::from_le_bytes(node_value[12..14].try_into().unwrap_or_else(|_| [0,0]));
                item.file_size = u32::from_le_bytes(node_value[4..8].try_into().unwrap_or_else(|_| [0,0,0,0]));
                match dosdate(&node_value[8..12]) {
                    Some(v) => {
                        item.m_time = v;
                    },
                    None => {}
                };
                let ext_offset = (node_value[entry_size as usize -2] as u16 | ((node_value[entry_size as usize -1] as u16) << 8)) as usize;
                if ext_offset == 0 || ext_offset > entry_size as usize {
                    return None;
                }
                item.short_name = String::from_utf8_lossy(&node_value[14..ext_offset - 1]).to_string();
                if let Some(pos) = item.short_name.find('\0') {
                    item.short_name.truncate(pos);
                }
    
                let ext_size = node_value[ext_offset] as u16 | ((node_value[ext_offset + 1] as u16) << 8);
                if ext_size > entry_size {
                    println!("Error in ext size for path: {:?}", element_route);
                    return None;
                }
                item.ext_size = ext_size as u32;
    
                let ext_version = node_value[ext_offset + 2] as u16 | ((node_value[ext_offset + 3] as u16) << 8);
                item.ext_version = ext_version as u32;
    
                let mut offset = 4 + ext_offset;
                if ext_version >= 0x03 {
                    let check = u32::from_le_bytes(node_value[offset..offset + 4].try_into().unwrap_or_default()) as u64;
                    if check != 0xbeef0004 {
                        println!("Error parsing file entry detecting 0xbeef0004");
                        return None;
                    }
                    offset += 4;
                    match dosdate(&node_value[offset..offset + 4]) {
                        Some(v) => {
                            item.c_time = v;
                        },
                        None => {}
                    };
                    offset += 4;
                    match dosdate(&node_value[offset..offset + 4]) {
                        Some(v) => {
                            item.a_time = v;
                        },
                        None => {}
                    };
                    offset += 6; // 2 from unknown
                }
                
                if ext_version >= 0x07 {
                    offset += 18;
                }
                if ext_version >= 0x03 {
                    offset += 2; //Name size
                }
                if ext_version >= 0x09 {
                    offset += 4;
                }
                if ext_version >= 0x08 {
                    offset += 4;
                }
    
                if ext_version >= 0x03 {
                    let str_vec : Vec<u16> = node_value[offset..].chunks(2).map(|v| {
                        if v.len() == 2 {
                            (v[1] as u16) << 8 | v[0] as u16
                        }else if v.len() == 1 {
                            v[0] as u16
                        }else {
                            0
                        }
                    }).collect();
                    let pos = str_vec.iter().position(|&v| v == 0).unwrap_or_default();
                    if pos > 0 {
                        item.long_name = String::from_utf16_lossy(&str_vec[0..pos]);
                    }
                }
                ShellItem::File(item)
            }
        )
    }
    let is_network = (entry_type & 0x70) == 0x40;
    if is_network {
        return Some(
            {
                let mut item = ShellNetworkItem::default();
                if entry_type & 0x0F == 0x0D {
                    item.guid = Some(Uuid::from_slice_le(&node_value[3..19]).unwrap_or_default().as_u128());
                }else {
                    let flags = node_value[4];
                    item.flags = flags as u32;
                    let mut offset = 5;
                    let pos = node_value[offset..].iter().position(|&v| v == 0).unwrap_or_default();
                    if pos > 0 {
                        item.location = String::from_utf8_lossy(&node_value[offset.. offset + pos]).to_string();
                        offset += pos + 1;
                    }
                    if (flags & 0x80) > 0 {
                        let str_vec = &node_value[offset..];
                        let pos = str_vec.iter().position(|&v| v == 0).unwrap_or_default();
                        if pos > 0 {
                            item.description = String::from_utf8_lossy(&str_vec[0..pos]).to_string();
                            offset += pos + 1;
                        }
                    }
                    if (flags & 0x40) > 0 {
                        let str_vec = &node_value[offset..];
                        let pos = str_vec.iter().position(|&v| v == 0).unwrap_or_default();
                        if pos > 0 {
                            item.comment = String::from_utf8_lossy(&str_vec[0..pos]).to_string();
                        }
                    }
                }
                ShellItem::Network(item)
            }
        );
    }
    Some(match entry_type {
        0x1F => {
            // FOLDER
            let mut item = ShellFolderItem::default();
            item.id = node_value[3];
            item.guid = Uuid::from_slice_le(&node_value[4..20]).unwrap_or_default().as_u128();
            item.name = match node_value[3] {
                0x00 => "INTERNET_EXPLORER",
                0x42 => "LIBRARIES",
                0x44 => "USERS",
                0x48 => "MY_DOCUMENTS",
                0x50 => "MY_COMPUTER",
                0x58 => "NETWORK",
                0x60 => "RECYCLE_BIN",
                0x68 => "INTERNET_EXPLORER",
                0x70 => "UNKNOWN",
                0x80 => "MY_GAMES",
                _ => ""
            }.into();
            // Name must be retrieved 
            ShellItem::Folder(item)
        },
        0x2f => {
            //VOLUME
            let mut item = ShellVolumeItem::default();
            item.name = String::from_utf8_lossy(&node_value[3..6]).to_string();
            ShellItem::Volume(item)
        },
        _ => ShellItem::Unknown(entry_type as u32)
    })


}

pub fn dosdate(data : &[u8]) -> Option<i64> {
    if data.len() != 4 {
        return None;
    }
    let dt = u16::from_le_bytes(data[0..2].try_into().unwrap_or_else(|_| [0,0])) as u64;
    //let dt = ((data[1] as u64) << 8) | data[0] as u64;
    let day = dt & 0b0000000000011111;
    let month = (dt & 0b0000000111100000) >> 5;
    let year = (dt & 0b1111111000000000) >> 9;
    let year = year + 1980;
    let dd = u16::from_le_bytes(data[2..4].try_into().unwrap_or_else(|_| [0,0])) as u64;
    //let dd = ((data[3] as u64) << 8) | data[2] as u64;
    let sec = (dd & 0b0000000000011111) * 2;
    let minute = (dd & 0b0000011111100000) >> 5;
    let hour = (dd & 0b1111100000000000) >> 11;
    if day == 0 || month == 0 {
        return None;
    }
    /* 
    let dt = NaiveDate::from_ymd(year as i32, month as u32, day as u32).and_hms(hour as u32, minute as u32, sec as u32);
    let dt = Utc.from_utc_datetime(&dt);
    
    let dt = Utc.from_local_datetime(&dt);
    let dt = match dt {
        chrono::LocalResult::None => return None,
        chrono::LocalResult::Single(v) => v,
        chrono::LocalResult::Ambiguous(v1, _v2) => v1,
    };*/
    let dt = Utc.with_ymd_and_hms(year as i32, month as u32, day as u32, hour as u32, minute as u32, sec as u32);
    //println!("Dosdate: {}, {}, {}-{}-{} {}:{}:{}", dt.timestamp(), dd, year, month, day, hour, minute, sec);
    Some(dt.unwrap().timestamp())
}

pub unsafe fn get_mru_list(bag_mru_key : HKEY) -> Result<Vec<u32>,WIN32_ERROR> {
    let mru_list_str = to_pwstr("MRUListEx");
    let mut capacity : u32 = 10_000;
    let mut readed_data =vec![0; capacity as usize];
    let mut data_type : REG_VALUE_TYPE = REG_BINARY;
    let reserved : *const u32 = std::ptr::null();
    let readed = RegQueryValueExW(bag_mru_key, PCWSTR(mru_list_str.as_ptr()),Some(reserved as _), Some(addr_of_mut!(data_type)),Some(readed_data.as_mut_ptr()), Some(addr_of_mut!(capacity)));
    if readed.is_err() {
        return Err(readed);
    }
    Ok(readed_data[0..capacity as usize - 4].chunks(4).map(|v| u32::from_ne_bytes(v.try_into().unwrap())).collect())
}