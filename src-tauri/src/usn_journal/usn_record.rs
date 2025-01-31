use std::{
    ffi::OsString,
    hash::Hash,
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
    slice::from_raw_parts,
};

use serde::{Deserialize, Serialize};
use windows::Win32::{
    Foundation::{CloseHandle, ERROR_MORE_DATA, HANDLE, MAX_PATH},
    Storage::FileSystem::{
        ExtendedFileIdType, FileIdType, FileNameInfo, GetFileInformationByHandleEx, OpenFileById,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_ID_128, FILE_ID_DESCRIPTOR, FILE_ID_DESCRIPTOR_0,
        FILE_NAME_INFO, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    },
    System::Ioctl::{
        USN_REASON_BASIC_INFO_CHANGE, USN_REASON_CLOSE, USN_REASON_COMPRESSION_CHANGE,
        USN_REASON_DATA_EXTEND, USN_REASON_DATA_OVERWRITE, USN_REASON_DATA_TRUNCATION,
        USN_REASON_DESIRED_STORAGE_CLASS_CHANGE, USN_REASON_EA_CHANGE,
        USN_REASON_ENCRYPTION_CHANGE, USN_REASON_FILE_CREATE, USN_REASON_FILE_DELETE,
        USN_REASON_HARD_LINK_CHANGE, USN_REASON_INDEXABLE_CHANGE, USN_REASON_INTEGRITY_CHANGE,
        USN_REASON_NAMED_DATA_EXTEND, USN_REASON_NAMED_DATA_OVERWRITE,
        USN_REASON_NAMED_DATA_TRUNCATION, USN_REASON_OBJECT_ID_CHANGE, USN_REASON_RENAME_NEW_NAME,
        USN_REASON_RENAME_OLD_NAME, USN_REASON_REPARSE_POINT_CHANGE, USN_REASON_SECURITY_CHANGE,
        USN_REASON_STREAM_CHANGE, USN_REASON_TRANSACTED_CHANGE,
    },
};

use super::volume::Volume;

#[derive(Clone)]
pub enum Version {
    _2,
    _3,
}

#[derive(Clone)]
pub enum FileIdentifier {
    _2(u64),
    _3(FILE_ID_128),
}

#[derive(Clone)]
pub struct UsnRecord {
    pub version: Version,
    pub file_id: FileIdentifier,
    pub parent_file_id: FileIdentifier,
    pub file_name: String,
    pub reason: u32,
    pub timestamp: i64,
}

impl Hash for UsnRecord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.file_name.hash(state);
    }
}

impl PartialEq for UsnRecord {
    fn eq(&self, other: &Self) -> bool {
        self.file_name.eq(&other.file_name)
    }
}

impl Eq for UsnRecord {}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub name: String,         // имя файла включая расширение
    pub path: Option<String>, // путь до файла
    pub timestamp: i64,
    pub reason: String, // причина добавления в журнал
}

impl FileRecord {
    pub fn new(usn_record: UsnRecord, volume: Volume, handle: HANDLE) -> Self {
        Self {
            reason: Self::get_reason_str(usn_record.clone().reason),
            name: usn_record.clone().file_name,
            path: Self::get_file_path(usn_record.clone(), volume, handle),
            timestamp: usn_record.clone().timestamp,
        }
    }

    fn get_file_path(usn_record: UsnRecord, volume: Volume, handle: HANDLE) -> Option<String> {
        let (id, id_type) = match usn_record.file_id {
            FileIdentifier::_2(id) => (FILE_ID_DESCRIPTOR_0 { FileId: id as i64 }, FileIdType),
            FileIdentifier::_3(id) => (
                FILE_ID_DESCRIPTOR_0 { ExtendedFileId: id },
                ExtendedFileIdType,
            ),
        };

        let file_id_descriptor = FILE_ID_DESCRIPTOR {
            Type: id_type,
            dwSize: size_of::<FILE_ID_DESCRIPTOR>() as u32,
            Anonymous: id,
        };

        unsafe {
            match OpenFileById(
                handle,
                &file_id_descriptor,
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                None,
                FILE_FLAG_BACKUP_SEMANTICS,
            ) {
                Ok(file) => {
                    if !file.is_invalid() {
                        let mut buf_size =
                            size_of::<FILE_NAME_INFO>() + (MAX_PATH as usize) * size_of::<u16>();
                        // нам нужно использовать буфер который сможет принимать максимальный путь, обязательно переведя в utf16, так как в utf8 будут пустые байты и путь будет с пустыми символами, пример: C : \ W i n d o w s
                        let mut buf = vec![0u8; buf_size];

                        let result = loop {
                            match GetFileInformationByHandleEx(
                                file,
                                FileNameInfo,
                                buf.as_mut_ptr() as *mut _,
                                buf_size as u32,
                            ) {
                                Ok(_) => {
                                    let (_, body, _) = buf.align_to::<FILE_NAME_INFO>();
                                    let info = &body[0];
                                    let name_len = info.FileNameLength as usize / size_of::<u16>();
                                    let name_u16 = from_raw_parts(
                                        info.FileName.as_ptr() as *const u16,
                                        name_len,
                                    );
                                    break Some(PathBuf::from(OsString::from_wide(name_u16)));
                                }

                                Err(e) => {
                                    // мы слишком много выделили для получения результата, поэтому мы можем попробовать еще раз изменив размер на требуемый
                                    if e.code() == ERROR_MORE_DATA.to_hresult() {
                                        let required_size = buf.align_to::<FILE_NAME_INFO>().1[0]
                                            .FileNameLength
                                            as usize;

                                        buf_size = size_of::<FILE_NAME_INFO>() + required_size;
                                        buf.resize(buf_size, 0);
                                    } else {
                                        break None;
                                    }
                                }
                            }
                        };

                        let _ = CloseHandle(file);

                        if result.is_none() {
                            return None;
                        }

                        // мы получаем не полный путь, буквы диска нет, и имени файла тоже, поэтому возьмем это все уже с известного
                        let path = String::from(
                            Path::new(volume.path.replace("\\", "").as_str())
                                .join(result.unwrap().join(usn_record.file_name))
                                .to_str()
                                .unwrap_or("undefined"),
                        );

                        if path.eq("undefined") {
                            return None;
                        }

                        return Some(path);
                    }
                }

                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}")
                    }
                }
            }
        }

        None
    }

    fn get_reason_str(reason: u32) -> String {
        let reasons = [
            (USN_REASON_BASIC_INFO_CHANGE, "USN_REASON_BASIC_INFO_CHANGE"),
            (USN_REASON_CLOSE, "USN_REASON_CLOSE"),
            (
                USN_REASON_COMPRESSION_CHANGE,
                "USN_REASON_COMPRESSION_CHANGE",
            ),
            (USN_REASON_DATA_EXTEND, "USN_REASON_DATA_EXTEND"),
            (USN_REASON_DATA_OVERWRITE, "USN_REASON_DATA_OVERWRITE"),
            (USN_REASON_DATA_TRUNCATION, "USN_REASON_DATA_TRUNCATION"),
            (
                USN_REASON_DESIRED_STORAGE_CLASS_CHANGE,
                "USN_REASON_DESIRED_STORAGE_CLASS_CHANGE",
            ),
            (USN_REASON_EA_CHANGE, "USN_REASON_EA_CHANGE"),
            (USN_REASON_ENCRYPTION_CHANGE, "USN_REASON_ENCRYPTION_CHANGE"),
            (USN_REASON_FILE_CREATE, "USN_REASON_FILE_CREATE"),
            (USN_REASON_FILE_DELETE, "USN_REASON_FILE_DELETE"),
            (USN_REASON_HARD_LINK_CHANGE, "USN_REASON_HARD_LINK_CHANGE"),
            (USN_REASON_INDEXABLE_CHANGE, "USN_REASON_INDEXABLE_CHANGE"),
            (USN_REASON_INTEGRITY_CHANGE, "USN_REASON_INTEGRITY_CHANGE"),
            (USN_REASON_NAMED_DATA_EXTEND, "USN_REASON_NAMED_DATA_EXTEND"),
            (
                USN_REASON_NAMED_DATA_OVERWRITE,
                "USN_REASON_NAMED_DATA_OVERWRITE",
            ),
            (
                USN_REASON_NAMED_DATA_TRUNCATION,
                "USN_REASON_NAMED_DATA_TRUNCATION",
            ),
            (USN_REASON_OBJECT_ID_CHANGE, "USN_REASON_OBJECT_ID_CHANGE"),
            (USN_REASON_RENAME_NEW_NAME, "USN_REASON_RENAME_NEW_NAME"),
            (USN_REASON_RENAME_OLD_NAME, "USN_REASON_RENAME_OLD_NAME"),
            (
                USN_REASON_REPARSE_POINT_CHANGE,
                "USN_REASON_REPARSE_POINT_CHANGE",
            ),
            (USN_REASON_SECURITY_CHANGE, "USN_REASON_SECURITY_CHANGE"),
            (USN_REASON_STREAM_CHANGE, "USN_REASON_STREAM_CHANGE"),
            (USN_REASON_TRANSACTED_CHANGE, "USN_REASON_TRANSACTED_CHANGE"),
        ];

        let mut reason_str = String::new();

        for (flag, description) in reasons.iter() {
            if reason & flag != 0 {
                reason_str.push_str(description);
                reason_str.push(' ');
            }
        }

        reason_str
    }
}
