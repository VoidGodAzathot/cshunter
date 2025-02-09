use std::{
    ffi::c_void,
    ptr::{addr_of_mut, null_mut},
};

use serde::{Deserialize, Serialize};
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{
            GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_MORE_DATA, HANDLE, HWND, S_OK,
        },
        Security::WinTrust::{
            WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0,
            WINTRUST_DATA_UICONTEXT, WINTRUST_FILE_INFO, WINTRUST_SIGNATURE_SETTINGS,
            WTD_CHOICE_FILE, WTD_REVOCATION_CHECK_NONE, WTD_REVOKE_NONE, WTD_STATEACTION_IGNORE,
            WTD_UI_NONE,
        },
        System::Services::{
            CloseServiceHandle, EnumServicesStatusExW, OpenSCManagerW, OpenServiceW, QueryServiceConfigW, ENUM_SERVICE_STATUS_PROCESSW, QUERY_SERVICE_CONFIGW, SC_ENUM_PROCESS_INFO, SC_HANDLE, SC_MANAGER_CONNECT, SC_MANAGER_ENUMERATE_SERVICE, SERVICE_FILE_SYSTEM_DRIVER, SERVICE_KERNEL_DRIVER, SERVICE_QUERY_CONFIG, SERVICE_STATE_ALL
        },
    },
};

use crate::utils::string_to_pcwstr;

#[derive(Debug, Deserialize, Serialize)]
pub struct DriverInfo {
    pub name: String,
    pub description: String,
    pub path: String,
    pub trust: bool,
}

pub fn collect_drivers_info() -> Vec<DriverInfo> {
    unsafe {
        match OpenSCManagerW(
            PCWSTR::null(),
            PCWSTR::null(),
            SC_MANAGER_CONNECT | SC_MANAGER_ENUMERATE_SERVICE,
        ) {
            Ok(sc_manager) => {
                let mut bytes_needed: u32 = 0;
                let mut services_returned: u32 = 0;

                let result = EnumServicesStatusExW(
                    sc_manager,
                    SC_ENUM_PROCESS_INFO,
                    SERVICE_KERNEL_DRIVER | SERVICE_FILE_SYSTEM_DRIVER,
                    SERVICE_STATE_ALL,
                    None,
                    &mut bytes_needed,
                    &mut services_returned,
                    None,
                    PCWSTR::null(),
                );

                if let Err(err) = result {
                    if err.code() != ERROR_MORE_DATA.to_hresult() {
                        return vec![];
                    }
                }

                let mut buffer = vec![0u8; bytes_needed as usize];

                let _ = EnumServicesStatusExW(
                    sc_manager,
                    SC_ENUM_PROCESS_INFO,
                    SERVICE_KERNEL_DRIVER | SERVICE_FILE_SYSTEM_DRIVER,
                    SERVICE_STATE_ALL,
                    Some(&mut buffer),
                    &mut bytes_needed,
                    &mut services_returned,
                    None,
                    PCWSTR::null(),
                );

                let mut drivers = vec![];
                let mut offset = 0;

                for _ in 0..services_returned {
                    let service =
                        &*(buffer.as_ptr().add(offset) as *const ENUM_SERVICE_STATUS_PROCESSW);

                    let service_name = service.lpServiceName.to_string().unwrap_or_default();

                    let display_name = service.lpDisplayName.to_string().unwrap_or_default();

                    if let Some(driver_path) = get_driver_path(&service_name, sc_manager) {
                        let trust = verify_file_signature(&driver_path);

                        drivers.push(DriverInfo {
                            name: service_name,
                            description: display_name,
                            path: driver_path,
                            trust: trust,
                        });
                    }

                    offset += size_of::<ENUM_SERVICE_STATUS_PROCESSW>();
                }

                let _ = CloseServiceHandle(sc_manager);

                return drivers;
            }

            Err(e) => {
                println!("{e:?}");
            }
        }
    }

    vec![]
}

unsafe fn get_driver_path(service_name: &str, scm_handle: SC_HANDLE) -> Option<String> {
    let service_name_w: Vec<u16> = service_name.encode_utf16().chain(Some(0)).collect();
    let service_handle = OpenServiceW(
        scm_handle,
        PCWSTR(service_name_w.as_ptr()),
        SERVICE_QUERY_CONFIG,
    )
    .ok()?;
    let mut bytes_needed: u32 = 0;

    let _ = QueryServiceConfigW(service_handle, None, 0, &mut bytes_needed);
    if GetLastError().0 != ERROR_INSUFFICIENT_BUFFER.0 {
        let _ = CloseServiceHandle(service_handle);
        return None;
    }
    let mut buffer = vec![0u8; bytes_needed as usize];
    let lpqsc = buffer.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW;

    if !QueryServiceConfigW(service_handle, Some(lpqsc), bytes_needed, &mut bytes_needed).is_ok() {
        let _ = CloseServiceHandle(service_handle);
        return None;
    }
    let config = &*lpqsc;

    let driver_path = if !config.lpBinaryPathName.is_null() {
        config.lpBinaryPathName.to_string().unwrap_or_default()
    } else {
        String::new()
    };
    let _ = CloseServiceHandle(service_handle);
    Some(driver_path)
}

fn verify_file_signature(file_path: &str) -> bool {
    let mut file_info = WINTRUST_FILE_INFO {
        cbStruct: size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: string_to_pcwstr(String::from(file_path)),
        hFile: HANDLE(null_mut()),
        pgKnownSubject: null_mut(),
    };

    let mut pg_signature_settings = WINTRUST_SIGNATURE_SETTINGS::default();

    let mut data = WINTRUST_DATA {
        cbStruct: size_of::<WINTRUST_DATA>() as u32,
        pPolicyCallbackData: null_mut(),
        pSIPClientData: null_mut(),
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: WINTRUST_DATA_0 {
            pFile: &mut file_info,
        },
        dwStateAction: WTD_STATEACTION_IGNORE,
        hWVTStateData: HANDLE(null_mut()),
        pwszURLReference: PWSTR::null(),
        dwProvFlags: WTD_REVOCATION_CHECK_NONE,
        dwUIContext: WINTRUST_DATA_UICONTEXT(0),
        pSignatureSettings: addr_of_mut!(pg_signature_settings),
    };

    let mut guid_action = WINTRUST_ACTION_GENERIC_VERIFY_V2;

    let result = unsafe {
        WinVerifyTrust(
            HWND(null_mut()),
            addr_of_mut!(guid_action),
            addr_of_mut!(data) as *mut c_void,
        )
    };

    result == S_OK.0
}
