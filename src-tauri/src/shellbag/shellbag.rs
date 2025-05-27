use std::io::Read;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use windows_registry::{Type, CURRENT_USER};

use crate::mini_dat::registry_md::bytes_to_vec_u8;
use super::shellbag_dat::{BagMRU, ShellBagDat};

pub fn collect_shell_bag() -> Vec<ShellBagDat> {
    let items: Vec<ShellBagDat> = read_bag_mru(String::new(), String::new())
        .par_iter()
        .flat_map(|bag_mru| {
            let mut vec = Vec::new();
            if let Some(bag) = bag_mru_to_shell_bag(bag_mru) {
                vec.push(bag);
                vec.extend(collect_sub_shell_bag(bag_mru));
            }
            vec
        })
        .filter(|item| item.path.len() > 3)
        .collect();

    let final_items: Vec<ShellBagDat> = items
        .par_iter()
        .filter(|item| {
            let base = &item.path;
            let base_with_sep = if base.ends_with('\\') {
                base.clone()
            } else {
                format!("{}\\", base)
            };

            !items.iter().any(|other| {
                other.path.len() > base.len() && other.path.starts_with(&base_with_sep)
            })
        })
        .cloned()
        .collect();

    final_items
}

pub fn bag_mru_to_shell_bag(bag_mru: &BagMRU) -> Option<ShellBagDat> {
    bag_mru.full_name.as_ref().map(|path| {
        ShellBagDat {
            path: path
                .replacen("\\\\", "", 1)
                .replace(":\\\\", ":\\"),
        }
    })
}

pub fn collect_sub_shell_bag(bag_mru: &BagMRU) -> Vec<ShellBagDat> {
    bag_mru
        .sub
        .par_iter()
        .flat_map(|bag_mru| {
            let mut vec = vec![];
            if let Some(bag) = bag_mru_to_shell_bag(bag_mru) {
                vec.push(bag);
                vec.extend(collect_sub_shell_bag(bag_mru));
            }
            vec
        })
        .collect()
}

fn read_bag_mru(start_path: String, before_full_name: String) -> Vec<BagMRU> {
    let registry_path = format!(
        "SOFTWARE\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\BagMRU\\{}",
        start_path
    );

    let key = match CURRENT_USER.open(&registry_path) {
        Ok(key) => key,
        Err(e) => {
            if cfg!(debug_assertions) {
                eprintln!("Registry open error: {e:?}");
            }
            return vec![];
        }
    };

    match key.values() {
        Ok(values) => values
            .into_iter()
            .filter_map(|(name, value)| {
                if name.parse::<u32>().is_err() || value.ty() != Type::Bytes {
                    return None;
                }

                let entry_bytes = value.bytes();

                let new_path = format!("{}\\{}", start_path, name);
                let mut bag_mru = BagMRU {
                    path: new_path.clone(),
                    entry: bytes_to_vec_u8(entry_bytes),
                    sub: vec![],
                    full_name: None,
                    short_name: None,
                };

                decode_bag_mru_entry_name(&mut bag_mru);

                let full_name = format!(
                    "{}\\{}",
                    before_full_name,
                    bag_mru.short_name.clone().unwrap_or_default()
                );
                bag_mru.full_name = Some(full_name);
                bag_mru.sub = read_bag_mru(new_path, bag_mru.full_name.clone().unwrap_or_default());

                Some(bag_mru)
            })
            .collect(),
        Err(e) => {
            if cfg!(debug_assertions) {
                eprintln!("Registry read error: {e:?}");
            }
            vec![]
        }
    }
}

pub fn decode_bag_mru_entry_name(bag_mru: &mut BagMRU) {
    if bag_mru.entry.len() < 3 {
        return;
    }

    let entry_size_bytes = &bag_mru.entry[0..2];
    let entry_size = match entry_size_bytes.try_into() {
        Ok(bytes) => u16::from_le_bytes(bytes) as usize,
        Err(_) => return,
    };

    let entry_type = bag_mru.entry[2];

    match entry_type {
        0x2f => {
            if bag_mru.entry.len() >= 6 {
                bag_mru.short_name = Some(String::from_utf8_lossy(&bag_mru.entry[3..6]).to_string());
            }
        }
        0x31 => {
            if bag_mru.entry.len() >= entry_size {
                let ext_offset = {
                    let lo = *bag_mru.entry.get(entry_size - 2).unwrap_or(&0) as u16;
                    let hi = (*bag_mru.entry.get(entry_size - 1).unwrap_or(&0) as u16) << 8;
                    (lo | hi) as usize
                };

                if ext_offset > 14 && ext_offset <= bag_mru.entry.len() {
                    let short_name = String::from_utf8_lossy(&bag_mru.entry[14..ext_offset])
                        .to_string()
                        .replace('\0', "");
                    bag_mru.short_name = Some(short_name);
                }
            }
        }
        _ => {}
    }
}