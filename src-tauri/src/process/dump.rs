use std::{ffi::c_void, str::from_utf8};

use serde::{Deserialize, Serialize};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        Diagnostics::{
            Debug::{ReadProcessMemory, IMAGE_FILE_HEADER, IMAGE_NT_HEADERS64, IMAGE_SECTION_HEADER},
            ToolHelp::{
                CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W,
                TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
            },
        },
        Memory::{
            VirtualQueryEx, MEMORY_BASIC_INFORMATION64, MEM_COMMIT, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_READONLY, PAGE_READWRITE
        }, SystemServices::IMAGE_DOS_HEADER,
    },
};

use crate::emitter::global_emit;

use super::process::Process;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleStrings {
    pub values: Vec<String>,
    pub module: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Strings {
    pub address: String,
    pub values: Vec<String>,
}

pub fn dump_strings_from_process(process: Process) -> Vec<Strings> {
    let mut strings_with_addr = vec![];
    let mut total = 0;

    unsafe {
        let mut address = 0 as *const c_void;
        let mut mbi = MEMORY_BASIC_INFORMATION64::default();

        while VirtualQueryEx(
            process.handle,
            Some(address),
            &mut mbi as *const _ as _,
            size_of::<MEMORY_BASIC_INFORMATION64>(),
        ) == size_of::<MEMORY_BASIC_INFORMATION64>()
        {
            if mbi.State == MEM_COMMIT
                && (mbi.Protect.0 == PAGE_READONLY.0
                    || mbi.Protect.0 == PAGE_READWRITE.0
                    || mbi.Protect.0 == PAGE_EXECUTE_READ.0
                    || mbi.Protect.0 == PAGE_EXECUTE_READWRITE.0)
            {
                let region_size = mbi.RegionSize;
                let base_addr = mbi.BaseAddress;

                let mut buffer = vec![0u8; region_size.try_into().unwrap()];
                let mut bytes_read: usize = 0;

                if ReadProcessMemory(
                    process.handle,
                    base_addr as *const _,
                    buffer.as_mut_ptr() as *mut c_void,
                    region_size as usize,
                    Some(&mut bytes_read),
                )
                .is_ok()
                {
                    let mut current = vec![];
                    let mut strings: Vec<String> = vec![];
                    if bytes_read > buffer.len() {
                        break;
                    }
                    for &b in &buffer[..bytes_read] {
                        if b >= 0x20 && b <= 0x7E {
                            current.push(b);
                        } else {
                            if current.len() >= 4 {
                                if let Ok(s) = from_utf8(&current) {
                                    if is_valid_string(s) {
                                        strings.push(String::from(s));
                                    }
                                }
                            }
                            current.clear();
                        }
                    }
                    if current.len() >= 4 {
                        if let Ok(s) = from_utf8(&current) {
                            if is_valid_string(s) {
                                strings.push(String::from(s));
                            }
                        }
                    }
                    let strings_len = strings.len();
                    if strings_len > 0 {
                        strings_with_addr.push(Strings {
                            address: format!("{:#02x}", address as usize),
                            values: strings,
                        });
                        total += strings_len;
                        global_emit("task_status_update", &format!("{} найдено", total));
                    }
                }
            }

            address = ((mbi.BaseAddress as usize) + mbi.RegionSize as usize) as *const c_void;
        }
    }

    strings_with_addr
}

fn is_valid_string(s: &str) -> bool {
    if s.len() < 4 {
        return false;
    }

    if s.len() % 2 == 0 {
        let mut odd_chars = s.chars().skip(1).step_by(2);
        if let Some(first) = odd_chars.next() {
            if odd_chars.all(|c| c == first) {
                return false;
            }
        }
    }

    let letter_count = s.chars().filter(|c| c.is_alphabetic()).count();
    if letter_count == 0 || letter_count < s.len() / 3 {
        return false;
    }

    true
}

pub fn dump_modules_strings_from_process(process: Process) -> Vec<ModuleStrings> {
    let mut strings = vec![];
    let mut total = 0;
    unsafe {
        match CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, process.pid) {
            Ok(snapshot) => {
                let mut me32 = MODULEENTRY32W {
                    dwSize: size_of::<MODULEENTRY32W>() as u32,
                    ..Default::default()
                };

                if Module32FirstW(snapshot, &mut me32).is_ok() {
                    loop {
                        let mut module_strings = ModuleStrings {
                            values: vec![],
                            module: String::from_utf16_lossy(&me32.szModule),
                            address: String::new(),
                        };
                        dump_module_strings(process.handle, &me32, &mut module_strings);
                        if module_strings.values.len() != 0 && module_strings.address.len() != 0 {
                            total += module_strings.values.len();
                            global_emit("task_status_update", &format!("{} найдено", total));
                            strings.push(module_strings);
                        }
                        if !Module32NextW(snapshot, &mut me32).is_ok() {
                            break;
                        }
                    }
                }
                let _ = CloseHandle(snapshot);
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }
    }
    strings
}

unsafe fn dump_module_strings(
    process_handle: HANDLE,
    me32: &MODULEENTRY32W,
    module_strings: &mut ModuleStrings,
) {
    let base_addr = me32.modBaseAddr as *const c_void;
    let dos_header_size = size_of::<IMAGE_DOS_HEADER>();
    let mut dos_header_buf = vec![0u8; dos_header_size];
    let mut bytes_read: usize = 0;
    if !ReadProcessMemory(
        process_handle,
        base_addr,
        dos_header_buf.as_mut_ptr() as *mut c_void,
        dos_header_size,
        Some(&mut bytes_read),
    )
    .is_ok()
        || bytes_read != dos_header_size
    {
        return;
    }
    let dos_header: IMAGE_DOS_HEADER = *(dos_header_buf.as_ptr() as *const IMAGE_DOS_HEADER);
    if dos_header.e_magic != 0x5A4D {
        return;
    }

    let nt_headers_addr =
        (me32.modBaseAddr as usize + dos_header.e_lfanew as usize) as *const c_void;
    let nt_headers_size = size_of::<IMAGE_NT_HEADERS64>();
    let mut nt_headers_buf = vec![0u8; nt_headers_size];
    if !ReadProcessMemory(
        process_handle,
        nt_headers_addr,
        nt_headers_buf.as_mut_ptr() as *mut c_void,
        nt_headers_size,
        Some(&mut bytes_read),
    )
    .is_ok()
        || bytes_read != nt_headers_size
    {
        return;
    }
    let nt_headers: IMAGE_NT_HEADERS64 = *(nt_headers_buf.as_ptr() as *const IMAGE_NT_HEADERS64);
    if nt_headers.Signature != 0x00004550 {
        return;
    }

    let section_headers_addr = (me32.modBaseAddr as usize
        + dos_header.e_lfanew as usize
        + size_of::<u32>()
        + size_of::<IMAGE_FILE_HEADER>()
        + nt_headers.FileHeader.SizeOfOptionalHeader as usize)
        as *const c_void;

    for i in 0..nt_headers.FileHeader.NumberOfSections {
        let section_addr = (section_headers_addr as usize
            + i as usize * size_of::<IMAGE_SECTION_HEADER>())
            as *const c_void;
        let mut section_buf = vec![0u8; size_of::<IMAGE_SECTION_HEADER>()];
        if !ReadProcessMemory(
            process_handle,
            section_addr,
            section_buf.as_mut_ptr() as *mut c_void,
            size_of::<IMAGE_SECTION_HEADER>(),
            Some(&mut bytes_read),
        )
        .is_ok()
            || bytes_read != size_of::<IMAGE_SECTION_HEADER>()
        {
            continue;
        }
        let section: IMAGE_SECTION_HEADER = *(section_buf.as_ptr() as *const IMAGE_SECTION_HEADER);
        let name = match section.Name.iter().position(|&c| c == 0) {
            Some(pos) => &section.Name[..pos],
            None => &section.Name[..],
        };
        let section_name = String::from_utf8_lossy(name);
        if section_name == ".rdata" {
            module_strings.address = format!(
                "{:#02x}",
                me32.modBaseAddr as usize + section.VirtualAddress as usize
            );
            module_strings.values = dump_section_strings(
                process_handle,
                me32.modBaseAddr,
                section.VirtualAddress,
                section.Misc.VirtualSize,
            );
        }
    }
}

unsafe fn dump_section_strings(
    process_handle: HANDLE,
    module_base: *mut u8,
    section_va: u32,
    section_size: u32,
) -> Vec<String> {
    let section_addr = (module_base as usize + section_va as usize) as *const c_void;
    let size = section_size as usize;
    let mut section_buf = vec![0u8; size];
    let mut bytes_read: usize = 0;
    if !ReadProcessMemory(
        process_handle,
        section_addr,
        section_buf.as_mut_ptr() as *mut c_void,
        size,
        Some(&mut bytes_read),
    )
    .is_ok()
    {
        return vec![];
    }
    let mut strings = vec![];
    let data = &section_buf[..bytes_read];
    let mut pos = 0;
    while pos < data.len() {
        if data[pos] == 0 {
            pos += 1;
            continue;
        }
        if let Some(end_offset) = data[pos..].iter().position(|&b| b == 0) {
            let string_bytes = &data[pos..pos + end_offset];
            if string_bytes.len() >= 4
                && string_bytes
                    .iter()
                    .all(|&b| (b.is_ascii_graphic() || b == b' '))
            {
                if let Ok(s) = from_utf8(string_bytes) {
                    strings.push(String::from(s));
                }
            }
            pos += end_offset + 1;
        } else {
            break;
        }
    }
    strings
}
