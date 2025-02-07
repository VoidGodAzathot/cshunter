use std::{
    ffi::OsString,
    os::windows::{ffi::OsStringExt, process::CommandExt},
    path::PathBuf,
    process::Command,
    ptr::{addr_of_mut, null_mut},
    thread,
    time::Duration,
};

use jwalk::WalkDir;
use rand::{distr::Alphanumeric, Rng};
use rayon::iter::{ParallelBridge, ParallelIterator};
use regex::Regex;
use tauri::{AppHandle, Manager};
use windows::{
    core::{GUID, PCWSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, TokenUser, TOKEN_QUERY,
            TOKEN_USER,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
        UI::Shell::{SHGetKnownFolderPath, KNOWN_FOLDER_FLAG},
    },
};

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

pub fn rot13(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'A'..='M' | 'a'..='m' => ((c as u8) + 13) as char,
            'N'..='Z' | 'n'..='z' => ((c as u8) - 13) as char,
            _ => c,
        })
        .collect()
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

#[tauri::command]
pub fn open_explorer(path: String) {
    let mut path_buf = PathBuf::from(&path);

    if path_buf.exists() && path_buf.is_file() {
        if let Some(parent) = path_buf.parent() {
            path_buf = parent.to_path_buf();
        }
    }

    if !path_buf.is_dir() {
        return;
    }

    let _ = Command::new("explorer")
        .arg(path_buf)
        .creation_flags(0x08000000)
        .spawn();
}

#[tauri::command]
pub fn open_url(url: String) {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return;
    }

    let _ = Command::new("cmd")
        .args(&["/C", "start", url.as_str()])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| e.to_string());
}

#[tauri::command]
pub fn run_main_window_and_close_preload(app: AppHandle) {
    thread::spawn(move || {
        let preload_window = app.get_webview_window("main");
        if preload_window.is_none() {
            return;
        }
        let cshunter_window = app.get_webview_window("cshunter");
        if cshunter_window.is_none() {
            return;
        }
        let _ = preload_window.unwrap().hide();
        let cshunter_window = cshunter_window.unwrap();
        let _ = cshunter_window.eval("window.location.reload()");
        thread::sleep(Duration::from_millis(1000));
        let _ = cshunter_window.show();
    });
}

pub fn get_known_folder_path(rfid: *const GUID) -> String {
    unsafe {
        let result = SHGetKnownFolderPath(rfid, KNOWN_FOLDER_FLAG(0), Some(HANDLE::default()));
        if result.is_err() {
            return String::from("");
        }
        let result = result.unwrap();
        let path = OsString::from_wide(result.as_wide())
            .to_string_lossy()
            .into_owned();
        path
    }
}

pub fn parse_guid_from_string(guid_str: &str) -> Result<GUID, String> {
    let clean_guid = guid_str.trim_matches(|c| c == '{' || c == '}');
    let parts: Vec<&str> = clean_guid.split('-').collect();

    if parts.len() != 5 {
        return Err("Invalid GUID format".to_string());
    }

    let data1 =
        u32::from_str_radix(parts[0], 16).map_err(|_| "Failed to parse first part of GUID")?;
    let data2 =
        u16::from_str_radix(parts[1], 16).map_err(|_| "Failed to parse second part of GUID")?;
    let data3 =
        u16::from_str_radix(parts[2], 16).map_err(|_| "Failed to parse third part of GUID")?;

    let data4_str = parts[3].to_string() + &parts[4];
    let mut data4 = [0u8; 8];

    for (i, chunk) in data4_str.as_bytes().chunks(2).enumerate() {
        if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16) {
            data4[i] = byte;
        } else {
            return Err("Failed to parse Data4 bytes".to_string());
        }
    }

    Ok(GUID {
        data1: data1,
        data2: data2,
        data3: data3,
        data4,
    })
}

pub fn extract_guid(input: &str) -> Option<String> {
    let re = Regex::new(r"(?i)\{([A-F0-9\-]{36})\}").unwrap();

    if let Some(captures) = re.captures(input) {
        Some(captures.get(1).map_or("", |m| m.as_str()).to_string())
    } else {
        None
    }
}

pub fn replace_guid_in_path(input: &str, guid: &str, guid_str: &str) -> Option<String> {
    Some(String::from(
        input.replace(&format!("{{{}}}", guid).to_string(), guid_str),
    ))
}

pub fn known_folder_in_path(value: String) -> String {
    let guid_str = extract_guid(&value);
    if guid_str.is_some() {
        let guid_str: String = guid_str.unwrap();
        match parse_guid_from_string(&guid_str) {
            Ok(guid) => {
                let guid_ptr: *const GUID = &guid;
                let folder = get_known_folder_path(guid_ptr);
                let replaced = replace_guid_in_path(&value, &guid_str, &folder);
                if let Some(replaced) = replaced {
                    return replaced;
                }
            }
            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }
    }

    value
}

pub fn get_current_username_in_sid() -> Option<String> {
    unsafe {
        let process = GetCurrentProcess();

        let mut token_handle = HANDLE::default();
        let _ = OpenProcessToken(process, TOKEN_QUERY, addr_of_mut!(token_handle));

        let mut return_length = 0;
        let _ = GetTokenInformation(
            token_handle,
            TokenUser,
            Some(null_mut()),
            0,
            &mut return_length,
        );

        let mut buffer = vec![0u8; return_length as usize];
        if GetTokenInformation(
            token_handle,
            TokenUser,
            Some(buffer.as_mut_ptr() as *mut _),
            return_length,
            &mut return_length,
        )
        .is_err()
        {
            let _ = CloseHandle(token_handle);
            return None;
        }

        let token_user = &*(buffer.as_ptr() as *const TOKEN_USER);

        let mut string_sid: PWSTR = PWSTR(std::ptr::null_mut());
        if ConvertSidToStringSidW(token_user.User.Sid, &mut string_sid).is_err() {
            let _ = CloseHandle(token_handle);
            return None;
        }

        let sid = {
            let mut len = 0;
            while *string_sid.0.offset(len) != 0 {
                len += 1;
            }

            let slice = std::slice::from_raw_parts(string_sid.0, len as usize);
            String::from_utf16_lossy(slice)
        };

        let _ = CloseHandle(token_handle);
        Some(sid)
    }
}
