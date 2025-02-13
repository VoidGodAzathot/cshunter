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
            vec.push(bag_mru_to_shell_bag(bag_mru));
            vec.extend(collect_sub_shell_bag(bag_mru));
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

pub fn bag_mru_to_shell_bag(bag_mru: &BagMRU) -> ShellBagDat {
    let path = bag_mru.full_name.clone();
    ShellBagDat {
        path: path
            .unwrap()
            .replacen("\\\\", "", 1)
            .replace(":\\\\", ":\\"),
    }
}

pub fn collect_sub_shell_bag(bag_mru: &BagMRU) -> Vec<ShellBagDat> {
    bag_mru
        .sub
        .par_iter()
        .flat_map(|bag_mru| {
            let mut vec = vec![];
            vec.push(bag_mru_to_shell_bag(bag_mru));
            vec.extend(collect_sub_shell_bag(bag_mru));
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
            if cfg!(dev) {
                println!("{e:?}");
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
                let new_path = format!("{}\\{}", start_path, name);
                let mut bag_mru = BagMRU {
                    path: new_path.clone(),
                    entry: bytes_to_vec_u8(value.bytes()),
                    sub: vec![],
                    full_name: None,
                    short_name: None,
                };
                decode_bag_mru_entry_name(&mut bag_mru);
                bag_mru.full_name = Some(format!(
                    "{}\\{}",
                    before_full_name,
                    bag_mru.short_name.clone().unwrap_or(String::new())
                ));
                bag_mru.sub = read_bag_mru(new_path, bag_mru.full_name.clone().unwrap());
                Some(bag_mru)
            })
            .collect(),

        Err(e) => {
            if cfg!(dev) {
                println!("{e:?}");
            }
            return vec![];
        }
    }
}

pub fn decode_bag_mru_entry_name(bag_mru: &mut BagMRU) {
    let entry_size = u16::from_le_bytes(bag_mru.entry[0..2].try_into().unwrap_or([0, 0]));
    let entry_type = bag_mru.entry[2];

    if entry_type == 0x2f {
        // volume
        bag_mru.short_name = Some(String::from_utf8_lossy(&bag_mru.entry[3..6]).to_string());
    } else if entry_type == 0x31 {
        // file
        let ext_offset: usize = (bag_mru.entry[entry_size as usize - 2] as u16
            | ((bag_mru.entry[entry_size as usize - 1] as u16) << 8))
            as usize;

        if !(ext_offset == 0 || ext_offset > entry_size as usize) {
            let short_name = String::from_utf8_lossy(&bag_mru.entry[14..ext_offset])
                .to_string()
                .replace('\0', "");

            bag_mru.short_name = Some(short_name);
        }
    }
}
