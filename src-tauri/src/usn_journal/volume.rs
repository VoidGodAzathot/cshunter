use std::ptr::null_mut;

use serde::{Deserialize, Serialize};
use windows::Win32::{
    Foundation::{BOOL, HANDLE, MAX_PATH},
    Storage::FileSystem::{
        CreateFileW, GetDiskFreeSpaceExW, GetLogicalDriveStringsW, GetVolumeInformationW, FILE_FLAG_BACKUP_SEMANTICS, FILE_GENERIC_READ, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING
    }, System::SystemInformation::GetWindowsDirectoryW,
};

use crate::utils::string_to_pcwstr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Flag {
    SYSTEM,
    NTFS,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Volume {
    pub path: String,
    pub free_space: u64,
    pub available_space: u64,
    pub total_space: u64,
    pub flags: Vec<Flag>,
}

impl Volume {
    pub fn new(path: String) -> Self {
        let (free_space, available_space, total_space) = Self::get_space_info(path.clone());

        Self {
            path: path.clone(),
            free_space: free_space,
            available_space: available_space,
            total_space: total_space,
            flags: Self::get_flags(path.clone()),
        }
    }

    // возвращаемые значения: free, available, total
    fn get_space_info(path: String) -> (u64, u64, u64) {
        let mut free_space = 0u64;
        let mut total_space = 0u64;
        let mut available_space = 0u64;

        unsafe {
            let result = GetDiskFreeSpaceExW(
                string_to_pcwstr(path),
                Some(&mut free_space),
                Some(&mut total_space),
                Some(&mut available_space),
            );

            if result.is_err() {
                println!("{}", result.err().unwrap());
            }
        };

        (free_space, available_space, total_space)
    }

    fn get_flags(path: String) -> Vec<Flag> {
        let mut flags: Vec<Flag> = vec![];

        // проверка на ф/c ntfs
        {
            let mut buf = [0u16; 32];

            unsafe {
                let result = GetVolumeInformationW(
                    string_to_pcwstr(path.clone()), 
                    None, 
                    Some(null_mut()), 
                    Some(null_mut()), 
                    Some(null_mut()), 
                    Some(&mut buf));

                if result.is_err() {
                    println!("{}", result.err().unwrap());
                } else {
                    let fs = String::from_utf16_lossy(&buf);
                    let fs = fs.trim_end_matches("\0");
                    
                    if fs.eq_ignore_ascii_case("ntfs") {
                        flags.push(Flag::NTFS);
                    }
                }
            }
        }

        // является ли диск системным
        {
            let mut buf = [0u16; MAX_PATH as usize];
            
            unsafe {
                if BOOL(GetWindowsDirectoryW(Some(&mut buf)) as i32).as_bool() {
                    let path_to_win_dir = String::from_utf16_lossy(&buf);
                    let path_to_win_dir = path_to_win_dir.trim_end_matches("\0");

                    if path_to_win_dir.starts_with(&path.clone()) {
                        flags.push(Flag::SYSTEM);
                    }
                }
            };
        }

        flags
    }

    // обязательно закрыть после использования CloseHandle
    pub unsafe fn get_handle(&self) -> Option<HANDLE> {
        match CreateFileW(
            string_to_pcwstr(format!("\\\\.\\{}:", self.path.replace(":\\", ""))),
            FILE_GENERIC_READ.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        ) {
            Ok(handle) => {
                return Some(handle);
            }

            Err(e) => {
                println!("{e:?}");
                return None;
            }
        }
    }
}

pub fn get_all_volumes() -> Vec<Volume> {
    let mut volumes: Vec<Volume> = vec![];

    {
        let mut buf = [0u16; 1024];

        let logical_drives_buf_len = unsafe {
            GetLogicalDriveStringsW(Some(&mut buf))
        };

        if logical_drives_buf_len > 0 {
            let buf_str = String::from_utf16_lossy(&buf);
            let buf_str = buf_str.split("\0").filter(|s| !s.is_empty()).collect::<Vec<&str>>();

            for vol in buf_str {
                volumes.push(Volume::new(String::from(vol)));
            }
        }
    }

    volumes
}