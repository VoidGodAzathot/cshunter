use core::arch::asm;
use std::{ffi::OsString, iter::once, os::windows::ffi::OsStrExt, path::Path, slice::from_raw_parts_mut};

use mac_address::get_mac_address;
use windows::{
    core::{Error, GUID, PCWSTR},
    Win32::{
        Devices::DeviceAndDriverInstallation::{SetupDiEnumDeviceInfo, SetupDiGetClassDevsW, SetupDiGetDeviceRegistryPropertyW, DIGCF_ALLCLASSES, DIGCF_PRESENT, SPDRP_HARDWAREID, SP_DEVINFO_DATA}, Foundation::{CloseHandle, ERROR_MORE_DATA, GENERIC_READ, HWND}, Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, OPEN_EXISTING}, System::{
            Registry::{RegOpenKeyExW, HKEY_LOCAL_MACHINE, KEY_READ},
            Services::{
                CloseServiceHandle, EnumServicesStatusExW, OpenSCManagerW,
                ENUM_SERVICE_STATUS_PROCESSW, SC_ENUM_PROCESS_INFO, SC_MANAGER_CONNECT,
                SC_MANAGER_ENUMERATE_SERVICE, SERVICE_STATE_ALL, SERVICE_WIN32,
            },
        }
    },
};

use crate::device_id::device_id::DeviceId;

pub struct HyperVisorMethod;
pub struct VMWareBrandMethod;
pub struct VMDisplayDeviceMethod;
pub struct RegistryMethod;
pub struct ServicesMethod;
pub struct MacAddressMethod;
pub struct VirtualBoxHandleMethod;
pub struct VMFilesMethod;
pub struct HardwareMethod;

pub trait DetectMethod {
    fn new() -> Self;
    fn name(&self) -> &str;
    unsafe fn score(&self) -> (bool, u32);
}

pub struct VMDetectWrapper<T> {
    method: T,
}

impl DetectMethod for HardwareMethod {
    fn new() -> Self {
        HardwareMethod {}
    }

    fn name(&self) -> &str {
        "hardware method"
    }

    unsafe fn score(&self) -> (bool, u32) {
        let hwnd = HWND::default();
        let class_guid = GUID::zeroed();
        let flags = DIGCF_PRESENT | DIGCF_ALLCLASSES;

        let device_info_set = 
            SetupDiGetClassDevsW(
                Some(&class_guid), 
                None, 
                Some(hwnd), 
                flags).unwrap();
        
        let mut device_info_data = SP_DEVINFO_DATA {
            cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };

        let mut index = 0;
        while SetupDiEnumDeviceInfo(device_info_set, index, &mut device_info_data).is_ok() {
            let mut buffer = [0u16; 1024];
            let buffer_slice = from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer.len());

            let _ = SetupDiGetDeviceRegistryPropertyW(
                device_info_set,
                &device_info_data,
                SPDRP_HARDWAREID,
                None,
                Some(buffer_slice),
                Some(1024 as *mut u32),
            );

            let hardware_id = String::from_utf16_lossy(&buffer).trim().to_lowercase().to_string();
            
            if hardware_id.contains("vmware") 
                || hardware_id.contains("vbox") {
                return (true, 1);
            }

            index += 1;
        }

        (false, 0)
    }
}

impl DetectMethod for VMFilesMethod {
    fn new() -> Self {
        VMFilesMethod {}
    }

    fn name(&self) -> &str {
        "vm files method"
    }

    unsafe fn score(&self) -> (bool, u32) {
        let mut files_finded = 0;
        let files = [
            "C:\\windows\\system32\\drivers\\prleth.sys",
            "C:\\windows\\system32\\drivers\\prlfs.sys",
            "C:\\windows\\system32\\drivers\\prlmouse.sys",
            "C:\\windows\\system32\\drivers\\prlvideo.sys",
            "C:\\windows\\system32\\drivers\\prltime.sys",
            "C:\\windows\\system32\\drivers\\prl_pv32.sys",
            "C:\\windows\\system32\\drivers\\prl_paravirt_32.sys",
            "C:\\windows\\system32\\drivers\\VBoxMouse.sys",
            "C:\\windows\\system32\\drivers\\VBoxGuest.sys",
            "C:\\windows\\system32\\drivers\\VBoxSF.sys",
            "C:\\windows\\system32\\drivers\\VBoxVideo.sys",
            "C:\\windows\\system32\\vboxdisp.dll",
            "C:\\windows\\system32\\vboxhook.dll",
            "C:\\windows\\system32\\vboxmrxnp.dll",
            "C:\\windows\\system32\\vboxogl.dll",
            "C:\\windows\\system32\\vboxoglarrayspu.dll",
            "C:\\windows\\system32\\vboxoglcrutil.dll",
            "C:\\windows\\system32\\vboxoglerrorspu.dll",
            "C:\\windows\\system32\\vboxoglfeedbackspu.dll",
            "C:\\windows\\system32\\vboxoglpackspu.dll",
            "C:\\windows\\system32\\vboxoglpassthroughspu.dll",
            "C:\\windows\\system32\\vboxservice.exe",
            "C:\\windows\\system32\\vboxtray.exe",
            "C:\\windows\\system32\\VBoxControl.exe",
            "C:\\windows\\system32\\drivers\\vmmouse.sys",
            "C:\\windows\\system32\\drivers\\vmnet.sys",
            "C:\\windows\\system32\\drivers\\vmxnet.sys",
            "C:\\windows\\system32\\drivers\\vmhgfs.sys",
            "C:\\windows\\system32\\drivers\\vmx86.sys",
            "C:\\windows\\system32\\drivers\\hgfs.sys",
            "C:\\windows\\system32\\drivers\\vmsrvc.sys",
            "C:\\windows\\system32\\drivers\\vpc-s3.sys",
        ];

        for file in files {
            if Path::new(&file).exists() {
                files_finded += 1;
            }
        }

        if files_finded >= 5 {
            (true, 1)
        } else {
            (true, 0)
        }
    }
}

impl DetectMethod for VirtualBoxHandleMethod {
    fn new() -> Self {
        VirtualBoxHandleMethod {}
    }

    fn name(&self) -> &str {
        "virtual box handle method"
    }

    unsafe fn score(&self) -> (bool, u32) {
        match CreateFileW(
            PCWSTR(
                "\\\\.\\VBoxMiniRdrDN"
                    .encode_utf16()
                    .chain(Some(0))
                    .collect::<Vec<u16>>()
                    .as_mut_ptr(),
            ),
            GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        ) {
            Ok(handle) => {
                if !handle.is_invalid() {
                    let _ = CloseHandle(handle);
                    return (true, 2);
                };

                let _ = CloseHandle(handle);
            }
            Err(e) => {
                println!("{e:?}");
                return (false, 0);
            }
        }
        (true, 0)
    }
}

impl DetectMethod for MacAddressMethod {
    fn new() -> Self {
        MacAddressMethod {}
    }

    fn name(&self) -> &str {
        "mac address method"
    }

    unsafe fn score(&self) -> (bool, u32) {
        match get_mac_address() {
            Ok(Some(mac_address)) => {
                let mac_str = format!("{}", mac_address);

                if mac_str.starts_with("00:0C:29")
                    || mac_str.starts_with("00:50:56")
                    || mac_str.starts_with("08:00:27")
                    || mac_str.starts_with("00:1C:14")
                    || mac_str.starts_with("00:50:56")
                    || mac_str.starts_with("00:05:69")
                {
                    return (true, 1);
                }
            }
            Ok(None) => {
                return (false, 0);
            }
            Err(e) => {
                println!("{e:?}");
                return (false, 0);
            }
        }

        return (true, 0);
    }
}

impl DetectMethod for ServicesMethod {
    fn new() -> Self {
        ServicesMethod {}
    }

    fn name(&self) -> &str {
        "services method"
    }

    unsafe fn score(&self) -> (bool, u32) {
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
                    SERVICE_WIN32 as _,
                    SERVICE_STATE_ALL,
                    None,
                    &mut bytes_needed,
                    &mut services_returned,
                    None,
                    PCWSTR::null(),
                );

                if let Err(err) = result {
                    if err.code() != Error::from(ERROR_MORE_DATA).code() {
                        return (false, 0);
                    }
                }

                let mut buffer = vec![0u8; bytes_needed as usize];

                let _ = EnumServicesStatusExW(
                    sc_manager,
                    SC_ENUM_PROCESS_INFO,
                    SERVICE_WIN32 as _,
                    SERVICE_STATE_ALL,
                    Some(&mut buffer),
                    &mut bytes_needed,
                    &mut services_returned,
                    None,
                    PCWSTR::null(),
                );

                let mut offset = 0;
                for _ in 0..services_returned {
                    let service =
                        &*(buffer.as_ptr().add(offset) as *const ENUM_SERVICE_STATUS_PROCESSW);

                    let service_name = service
                        .lpServiceName
                        .to_string()
                        .unwrap_or_default()
                        .to_lowercase();

                    if service_name.contains("vmtools") || service_name.contains("vboxservice") {
                        return (true, 2);
                    }

                    offset += size_of::<ENUM_SERVICE_STATUS_PROCESSW>();
                }

                let _ = CloseServiceHandle(sc_manager);
            }

            Err(e) => {
                println!("{e:?}");
                return (false, 0);
            }
        }

        (true, 0)
    }
}

impl DetectMethod for RegistryMethod {
    fn new() -> Self {
        RegistryMethod {}
    }

    fn name(&self) -> &str {
        "registry method"
    }

    unsafe fn score(&self) -> (bool, u32) {
        let keys = [
            r"SYSTEM\CurrentControlSet\Services\VBoxGuest",
            r"HARDWARE\ACPI\DSDT\VBOX__",
            r"SOFTWARE\VMware, Inc.\VMware Tools",
        ];

        for key in keys {
            let hkey = HKEY_LOCAL_MACHINE;
            let mut phk_result = Default::default();

            if RegOpenKeyExW(
                hkey,
                PCWSTR(
                    OsString::from(key)
                        .encode_wide()
                        .chain(once(0))
                        .collect::<Vec<u16>>()
                        .as_ptr(),
                ),
                Some(0),
                KEY_READ,
                &mut phk_result,
            )
            .is_ok()
            {
                return (true, 1);
            }
        }

        (true, 0)
    }
}

impl DetectMethod for VMDisplayDeviceMethod {
    fn new() -> Self {
        VMDisplayDeviceMethod {}
    }

    fn name(&self) -> &str {
        "display device method"
    }

    unsafe fn score(&self) -> (bool, u32) {
        let device = DeviceId::get_gpu_dipslay_name();

        if device.to_lowercase().contains("vmware")
            || device.to_lowercase().contains("virtualbox")
            || device.to_lowercase().contains("rdpudd")
        {
            (true, 2)
        } else {
            (true, 0)
        }
    }
}

impl DetectMethod for VMWareBrandMethod {
    fn new() -> Self {
        VMWareBrandMethod {}
    }

    fn name(&self) -> &str {
        "vmware brand method"
    }

    unsafe fn score(&self) -> (bool, u32) {
        let mut brand = [0u32; 3];

        asm!(
            "mov eax, 0x40000000",
            "cpuid",
            "mov {0:e}, ebx",
            "mov {1:e}, ecx",
            "mov {2:e}, edx",
            out(reg) brand[0],
            out(reg) brand[1],
            out(reg) brand[2],
            options(nostack, nomem),
        );

        let bytes = brand
            .iter()
            .flat_map(|&x| x.to_le_bytes())
            .collect::<Vec<_>>();

        if String::from_utf8_lossy(&bytes).contains("VMwareVMware") {
            (true, 2)
        } else {
            (true, 0)
        }
    }
}

impl DetectMethod for HyperVisorMethod {
    fn new() -> Self {
        HyperVisorMethod {}
    }

    fn name(&self) -> &str {
        "hyper visor method"
    }

    unsafe fn score(&self) -> (bool, u32) {
        let mut result: u8;

        asm!(
            "push rbx",
            "mov eax, 1",
            "cpuid",
            "shr ecx, 31",
            "and ecx, 1",
            "mov {0}, cl",
            "pop rbx",
            out(reg_byte) result,
            out("eax") _,
            out("ecx") _,
            out("edx") _,
            options(nostack),
        );

        if result == 1 {
            (true, 1)
        } else {
            (true, 0)
        }
    }
}

impl<T: DetectMethod> VMDetectWrapper<T> {
    pub fn new() -> Self {
        VMDetectWrapper { method: T::new() }
    }

    pub unsafe fn score(&self) -> u32 {
        let score = self.method.score().1;
        println!("{} completed has code: {}", self.method.name(), score);
        return score;
    }
}

pub struct VMDetector;

impl VMDetector {
    pub fn new() -> Self {
        VMDetector {}
    }

    pub unsafe fn is_vm(&self) -> bool {
        let hyper_visor_method = VMDetectWrapper::<HyperVisorMethod>::new();
        let vmware_brand_method = VMDetectWrapper::<VMWareBrandMethod>::new();
        let services_method = VMDetectWrapper::<ServicesMethod>::new();
        let registry_method = VMDetectWrapper::<RegistryMethod>::new();
        let display_device_method = VMDetectWrapper::<VMDisplayDeviceMethod>::new();
        let mac_address_method = VMDetectWrapper::<MacAddressMethod>::new();
        let virtual_box_handle_method = VMDetectWrapper::<VirtualBoxHandleMethod>::new();
        let vm_files_method = VMDetectWrapper::<VMFilesMethod>::new();
        let hardware_method = VMDetectWrapper::<HardwareMethod>::new();

        let score = &vmware_brand_method.score()
            + mac_address_method.score()
            + services_method.score()
            + hyper_visor_method.score()
            + display_device_method.score()
            + registry_method.score()
            + virtual_box_handle_method.score()
            + vm_files_method.score()
            + hardware_method.score();

        score > 2
    }
}