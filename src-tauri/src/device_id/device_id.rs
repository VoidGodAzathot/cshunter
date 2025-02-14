use core::arch::asm;
use std::{
    ffi::{c_void, CStr},
    mem::zeroed,
    ptr::{addr_of_mut, null_mut},
};

use nvml_wrapper::{enums::device::DeviceArchitecture, Nvml};
use sha2::{Digest, Sha256};
use windows::Win32::{
    Foundation::CloseHandle,
    Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW},
    Storage::FileSystem::{
        CreateFileW, FILE_FLAG_OVERLAPPED, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
        FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    },
    System::{
        Ioctl::{
            PropertyStandardQuery, StorageDeviceProperty, IOCTL_STORAGE_QUERY_PROPERTY,
            STORAGE_DESCRIPTOR_HEADER, STORAGE_DEVICE_DESCRIPTOR, STORAGE_PROPERTY_QUERY,
        },
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        IO::DeviceIoControl,
    },
    UI::WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
};

use crate::utils::string_to_pcwstr;

use super::shuffle::shuffle;

pub struct DeviceId {
    pub pairs: [String; 4],
}

impl DeviceId {
    pub fn generate() -> Result<Self, ()> {
        unsafe {
            let gpu_serial = match Self::get_idiomatic_serial_gpu() {
                Ok(serial) => serial,
                Err(_) => String::from("undefined"),
            };

            let cpu_serial = match Self::get_idiomatic_serial_cpu() {
                Ok(serial) => serial,
                Err(_) => String::from("undefined"),
            };

            let volume_serial = match Self::get_idiomatic_serial_volume() {
                Ok(serial) => serial,
                Err(_) => String::from("undefined"),
            };

            let ram_serial = match Self::get_idiomatic_serial_ram() {
                Ok(serial) => serial,
                Err(_) => String::from("undefined"),
            };

            Ok(DeviceId {
                pairs: [gpu_serial, cpu_serial, volume_serial, ram_serial],
            })
        }
    }

    pub fn to_string(&self) -> String {
        let mut str_val = String::new();

        for (i, pair) in self.pairs.iter().enumerate() {
            if i + 1 >= self.pairs.len() {
                break;
            }

            str_val.push_str(&shuffle(pair.into(), self.pairs[i + 1].clone()));
        }

        let hash = Sha256::digest(str_val);
        hex::encode(hash)
    }

    pub unsafe fn get_idiomatic_serial_ram() -> Result<String, ()> {
        let mut memory_status_ex = MEMORYSTATUSEX {
            dwLength: size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };

        match GlobalMemoryStatusEx(&mut memory_status_ex) {
            Ok(_) => {
                return Ok((memory_status_ex.ullTotalPhys / 1024 / 1024 / 1024).to_string());
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}")
                }
            }
        }

        Ok(String::from("undefined"))
    }

    pub unsafe fn get_idiomatic_serial_volume() -> Result<String, ()> {
        match CreateFileW(
            string_to_pcwstr(String::from("\\\\.\\c:")),
            (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            None,
        ) {
            Ok(handle) => {
                let mut storage_property_query = STORAGE_PROPERTY_QUERY {
                    PropertyId: StorageDeviceProperty,
                    QueryType: PropertyStandardQuery,
                    AdditionalParameters: [0; 1],
                };

                let mut storage_descriptor_header: STORAGE_DESCRIPTOR_HEADER = zeroed();
                let mut bytes_returned = 0;

                let _ = DeviceIoControl(
                    handle,
                    IOCTL_STORAGE_QUERY_PROPERTY,
                    Some(addr_of_mut!(storage_property_query) as *mut c_void),
                    size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                    Some(addr_of_mut!(storage_descriptor_header) as *mut c_void),
                    size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32,
                    Some(&mut bytes_returned),
                    Some(null_mut()),
                );

                let out_buffer_size = storage_descriptor_header.Size;
                let mut out_buffer: Vec<u8> = vec![0; out_buffer_size as usize];

                match DeviceIoControl(
                    handle,
                    IOCTL_STORAGE_QUERY_PROPERTY,
                    Some(addr_of_mut!(storage_property_query) as *mut c_void),
                    size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                    Some(out_buffer.as_mut_ptr() as *mut c_void),
                    out_buffer_size,
                    Some(&mut bytes_returned),
                    Some(null_mut()),
                ) {
                    Ok(_) => {
                        let device_descriptor: &STORAGE_DEVICE_DESCRIPTOR =
                            unsafe { &*(out_buffer.as_ptr() as *const STORAGE_DEVICE_DESCRIPTOR) };

                        let serial_number_offset = device_descriptor.SerialNumberOffset;
                        if serial_number_offset == 0 {
                            return Err(());
                        }

                        let serial_number_ptr =
                            unsafe { out_buffer.as_ptr().offset(serial_number_offset as isize) };
                        let serial_number =
                            unsafe { CStr::from_ptr(serial_number_ptr as *const i8) };

                        let _ = CloseHandle(handle);

                        return Ok(serial_number
                            .to_string_lossy()
                            .into_owned()
                            .trim()
                            .to_string());
                    }

                    Err(e) => {
                        if cfg!(dev) {
                            println!("{e:?}")
                        }
                    }
                }

                let _ = CloseHandle(handle);
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}")
                }
            }
        }

        Err(())
    }

    pub unsafe fn get_idiomatic_serial_cpu() -> Result<String, ()> {
        let mut buffer = [0u8; 48];
        let mut offset = 0;

        for i in 0x80000002u32..=0x80000004u32 {
            asm!(
                "cpuid",
                "mov [rdi], eax",
                "mov [rdi + 4], ebx",
                "mov [rdi + 8], ecx",
                "mov [rdi + 12], edx",
                in("eax") i,
                in("rdi") buffer.as_mut_ptr().add(offset),
                out("ecx") _,
                out("edx") _,
            );

            offset += 16;
        }

        Ok(String::from(
            String::from_utf8_lossy(&buffer).into_owned().trim(),
        ))
    }

    pub unsafe fn get_gpu_dipslay_name() -> String {
        let mut display_device = DISPLAY_DEVICEW {
            cb: size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };

        let result =
            EnumDisplayDevicesW(None, 0, &mut display_device, EDD_GET_DEVICE_INTERFACE_NAME);

        if result.as_bool() {
            let device_string = &display_device.DeviceString;
            let device_str = device_string
                .iter()
                .map(|&byte| byte as u8)
                .take_while(|&byte| byte != 0)
                .collect::<Vec<u8>>();
            return String::from_utf8_lossy(&device_str)
                .to_string()
                .trim()
                .to_string();
        }

        String::from("undefined")
    }

    pub unsafe fn get_idiomatic_serial_gpu() -> Result<String, ()> {
        let display_name = Self::get_gpu_dipslay_name();

        if display_name.to_lowercase().contains("nvidia") {
            match Nvml::init() {
                Ok(nvml) => match nvml.device_by_index(0) {
                    Ok(device) => match device.serial() {
                        Ok(serial) => {
                            return Ok(serial);
                        }

                        Err(_) => {
                            let device_name = device.name().unwrap_or(String::from("undefined"));
                            let num_cores = device.num_cores().unwrap_or(0);
                            let num_fans = device.num_fans().unwrap_or(0);
                            let arch = device.architecture().unwrap_or(DeviceArchitecture::Unknown);
                            let vbios_version =
                                device.vbios_version().unwrap_or(String::from("undefined"));

                            return Ok(String::from(format!(
                                "{}\\{}\\{}\\{}\\{}",
                                device_name, num_cores, num_fans, arch, vbios_version
                            ))
                            .trim()
                            .to_string());
                        }
                    },

                    Err(e) => {
                        if cfg!(dev) {
                            println!("{e:?}")
                        }
                    }
                },

                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}")
                    }
                }
            }
        } else {
            return Ok(display_name);
        }

        Err(())
    }
}
