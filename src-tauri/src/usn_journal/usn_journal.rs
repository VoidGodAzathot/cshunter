use std::{
    collections::HashSet,
    ffi::c_void,
    i64,
    mem::{offset_of, zeroed},
    ptr::{addr_of_mut, null_mut, read_unaligned},
    slice::from_raw_parts,
    usize,
};

use windows::{
    Wdk::Storage::FileSystem::MFT_ENUM_DATA,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::{
            Ioctl::{
                FSCTL_ENUM_USN_DATA, FSCTL_QUERY_USN_JOURNAL, FSCTL_READ_USN_JOURNAL,
                READ_USN_JOURNAL_DATA_V1, USN_JOURNAL_DATA_V2, USN_RECORD_UNION, USN_RECORD_V2,
                USN_RECORD_V3,
            },
            IO::DeviceIoControl,
        },
    },
};

use super::{
    usn_record::{FileIdentifier, FileRecord, UsnRecord, Version},
    volume::Volume,
};

pub struct UsnJournal {
    pub volume: Volume,
    volume_handle: Option<HANDLE>,
    journal_data: USN_JOURNAL_DATA_V2,
    journal_data_size: u32,
}

impl Drop for UsnJournal {
    fn drop(&mut self) {
        if self.volume_handle.is_some() {
            if !self.volume_handle.unwrap().is_invalid() {
                unsafe {
                    let _ = CloseHandle(self.volume_handle.unwrap());
                }
            }
        }
    }
}

#[repr(align(64))]
#[derive(Clone)]
struct Buffer {
    pub source: Vec<u8>,
    pub size: usize,
}

impl Buffer {
    pub fn new(size: usize) -> Self {
        let vec = vec![0u8; size];

        Self {
            source: vec,
            size: size,
        }
    }
}

impl UsnJournal {
    pub fn new(volume: Volume) -> Self {
        Self {
            volume: volume.clone(),
            volume_handle: Some(HANDLE(null_mut())),
            journal_data: unsafe { zeroed() }, // временно заполняем до вызова init'a
            journal_data_size: 0,
        }
    }

    pub fn init(&mut self) -> bool {
        unsafe {
            self.volume_handle = self.volume.clone().get_handle();
        };

        if self.volume_handle.is_none() || self.volume_handle.unwrap().is_invalid() {
            return false;
        }

        unsafe {
            let _ = DeviceIoControl(
                self.volume_handle.unwrap(),
                FSCTL_QUERY_USN_JOURNAL, // получаем текущий журнал; для создания журнала используется FSCTL_CREATE_USN_JOURNAL
                Some(null_mut()),        // нам не нужно отправлять параметры запроса
                0,
                Some(addr_of_mut!(self.journal_data) as *mut c_void),
                size_of::<USN_JOURNAL_DATA_V2>() as u32,
                Some(addr_of_mut!(self.journal_data_size)),
                None, // зачем нам нужна асинхронность в вызове?
            );
        };

        self.journal_data.NextUsn = 0;

        true
    }

    fn align_buffer(&self, size: usize) -> Buffer {
        // перед выполнением основного запроса для получения действий из журнала, нам необходимо создать буфер для передачи этих данных
        Buffer::new(size)
    }

    // получаем размер получаемого журнала и заполняем буфер
    fn fill_buffer(&self, buffer: &mut Buffer, start_usn: i64, reason_mask: u32) -> Option<usize> {
        let mut data = READ_USN_JOURNAL_DATA_V1 {
            StartUsn: start_usn,
            ReasonMask: reason_mask,
            ReturnOnlyOnClose: 0,
            Timeout: 0,
            BytesToWaitFor: 0,
            UsnJournalID: self.journal_data.UsnJournalID,
            MinMajorVersion: self.journal_data.MinSupportedMajorVersion,
            MaxMajorVersion: self.journal_data.MaxSupportedMajorVersion,
        };

        let mut size: u32 = 0;

        match unsafe {
            DeviceIoControl(
                self.volume_handle.unwrap(),
                FSCTL_READ_USN_JOURNAL,
                Some(addr_of_mut!(data) as *mut c_void),
                size_of::<READ_USN_JOURNAL_DATA_V1>() as u32,
                Some(buffer.source.as_mut_ptr() as *mut c_void),
                buffer.size as u32,
                Some(&mut size),
                None,
            )
        } {
            Ok(_) => {
                return Some(size as usize);
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}")
                }
                
                return None;
            }
        }
    }

    // получение всех записей, которые когда либо проходили через журнал посредством mft запроса без исключений
    pub fn read_all(&mut self) -> Vec<FileRecord> {
        let mut enum_data = MFT_ENUM_DATA {
            StartFileReferenceNumber: 0,
            LowUsn: 0,
            HighUsn: 0,
            MaxMajorVersion: 3,
            MinMajorVersion: 2,
        };

        let mut files: Vec<UsnRecord> = Vec::new();
        let mut buf = self.align_buffer(u32::MAX as usize);
        let mut size = 0;

        match unsafe {
            DeviceIoControl(
                self.volume_handle.unwrap(),
                FSCTL_ENUM_USN_DATA,
                Some(addr_of_mut!(enum_data) as *mut c_void),
                size_of::<MFT_ENUM_DATA>() as u32,
                Some(buf.source.as_mut_ptr() as *mut c_void),
                buf.size as u32,
                Some(&mut size),
                None,
            )
        } {
            Ok(_) => {
                let mut offset: usize = 8;

                while offset < size as usize {
                    // не допускаем выхода за размеры журнала
                    let (len, record) = unsafe { self.read_usn_union(&buf, offset, size as usize) };

                    if record.is_some() {
                        files.push(record.unwrap());
                    }

                    offset += len;
                }
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}")
                }
            },
        }

        let files = files.iter().map(|record| {
            FileRecord::new(
                record.clone(),
                self.volume.clone(),
                self.volume_handle.clone().unwrap(),
            )
        });

        Vec::from_iter(files)
    }

    pub fn read(&mut self, reason_mask: u32) -> Vec<FileRecord> {
        let mut response: Vec<UsnRecord> = vec![];

        let mut buf = self.align_buffer(u32::MAX as usize);
        let data_size = self.fill_buffer(&mut buf, self.journal_data.NextUsn, reason_mask);

        if data_size.is_some() {
            let data_size = data_size.unwrap();
            let next_usn = i64::from_le_bytes(buf.source[0..8].try_into().unwrap()); // исключаем размер USN(8 байт) из буфера

            if !(next_usn == 0 || next_usn < self.journal_data.NextUsn) {
                self.journal_data.NextUsn = next_usn;
            } else {
                return vec![];
            }

            let mut offset: usize = 8; // опять же пропускаем USN

            while offset < data_size {
                // не допускаем выхода за размеры журнала
                let (len, record) = unsafe { self.read_usn_union(&buf, offset, data_size) };

                if record.is_some() {
                    response.push(record.unwrap());
                }

                offset += len;
            }
        }

        let unique_data: HashSet<_> = response.into_iter().collect();

        let unique_data = unique_data.iter().map(|record| {
            FileRecord::new(
                record.clone(),
                self.volume.clone(),
                self.volume_handle.clone().unwrap(),
            )
        });

        Vec::from_iter(unique_data)
    }

    unsafe fn read_usn_union(
        &self,
        buf: &Buffer,
        offset: usize,
        size: usize,
    ) -> (usize, Option<UsnRecord>) {
        let rec = read_unaligned(buf.source[offset..].as_ptr() as *const USN_RECORD_UNION);

        let len: usize = rec.Header.RecordLength as usize;

        if len == 0 || offset + len > size {
            return (0, None);
        }

        // читаем имя файла из буфера
        let f_n_offset = if rec.Header.MajorVersion == 2 {
            offset_of!(USN_RECORD_V2, FileName)
        } else {
            offset_of!(USN_RECORD_V3, FileName)
        };

        let f_n = String::from_utf8_lossy(from_raw_parts(
            buf.source[offset + f_n_offset as usize..].as_ptr(),
            if rec.Header.MajorVersion == 2 {
                rec.V2.FileNameLength as usize
            } else {
                rec.V3.FileNameLength as usize
            },
        ))
        .to_string();

        (
            len,
            match rec.Header.MajorVersion {
                2 => Some(UsnRecord {
                    version: Version::_2,
                    file_id: FileIdentifier::_2(rec.V2.FileReferenceNumber),
                    parent_file_id: FileIdentifier::_2(rec.V2.ParentFileReferenceNumber),
                    file_name: f_n.replace("\0", ""),
                    reason: rec.V2.Reason,
                    timestamp: rec.V2.TimeStamp,
                }),
                3 => Some(UsnRecord {
                    version: Version::_3,
                    file_id: FileIdentifier::_3(rec.V3.FileReferenceNumber),
                    parent_file_id: FileIdentifier::_3(rec.V3.ParentFileReferenceNumber),
                    file_name: f_n.replace("\0", ""),
                    reason: rec.V3.Reason,
                    timestamp: rec.V3.TimeStamp,
                }),
                _ => None,
            },
        )
    }
}
