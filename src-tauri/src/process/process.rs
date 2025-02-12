use std::ptr::{addr_of, addr_of_mut, null_mut};

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, LUID},
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_DEBUG_NAME,
        SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    },
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
        Threading::{GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_ALL_ACCESS},
    },
};

pub struct Process {
    pub handle: HANDLE,
    pub pid: u32,
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_invalid() {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

impl Process {
    pub fn new(handle: HANDLE, pid: u32) -> Self {
        Self {
            handle: handle,
            pid: pid,
        }
    }

    pub fn find_by_name(name: &str) -> Option<Self> {
        let mut entry: PROCESSENTRY32W = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        unsafe {
            match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(snapshot) => {
                    if Process32FirstW(snapshot, &mut entry).is_ok() {
                        while Process32NextW(snapshot, &mut entry).is_ok() {
                            if String::from_utf16_lossy(&entry.szExeFile).contains(name) {
                                let _ = CloseHandle(snapshot);

                                return match OpenProcess(
                                    PROCESS_ALL_ACCESS,
                                    false,
                                    entry.th32ProcessID,
                                ) {
                                    Ok(handle) => Some(Self::new(handle, entry.th32ProcessID)),
                                    Err(_) => None,
                                };
                            }
                        }
                    }
                }

                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}");
                    }
                }
            }
        }

        None
    }
}

pub fn enable_debug_privilege() {
    unsafe {
        let mut token: HANDLE = HANDLE(null_mut());
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            addr_of_mut!(token),
        )
        .is_ok()
        {
            let mut luid = LUID::default();
            if LookupPrivilegeValueW(None, SE_DEBUG_NAME, addr_of_mut!(luid)).is_ok() {
                let privileges = TOKEN_PRIVILEGES {
                    PrivilegeCount: 1,
                    Privileges: [LUID_AND_ATTRIBUTES {
                        Luid: luid,
                        Attributes: SE_PRIVILEGE_ENABLED,
                    }],
                };
                let _ = AdjustTokenPrivileges(
                    token,
                    false,
                    Some(addr_of!(privileges) as *const TOKEN_PRIVILEGES),
                    size_of_val(&privileges) as u32,
                    None,
                    None,
                );
            }
            let _ = CloseHandle(token);
        }
    }
}
