use std::{env, ffi::c_void, fs, mem::zeroed, path::PathBuf};

use windows::Win32::Storage::{
    Jet::{
        JET_MoveFirst, JET_errSuccess, JetCloseTable, JetGetColumnInfoW, JetMove, JetOpenTableW,
        JetRetrieveColumn, JET_COLUMNDEF,
    },
    StructuredStorage::JET_TABLEID,
};

use crate::{
    mini_dat::registry_md::replace_device_path_with_drive_letter,
    srum::session::JetSession,
    utils::{random_name, string_to_pcwstr},
};

pub fn try_read_srum() -> Vec<String> {
    let path = &env::temp_dir().join(random_name());
    let mut strings = vec![];

    match fs::copy(PathBuf::from("C:\\Windows\\System32\\sru\\SRUDB.dat"), path) {
        Ok(_) => {
            let session = JetSession::open_instance(&random_name());
            if let Some(mut session) = session {
                if session.init() {
                    if session.begin() {
                        if session.attach_and_open_to_db(path.to_str().unwrap()) {
                            unsafe {
                                let table_name = "SruDbIdMapTable";
                                let mut table_id: JET_TABLEID = zeroed();

                                if JetOpenTableW(
                                    session.ses_id,
                                    session.dbid,
                                    string_to_pcwstr(String::from(table_name)).0,
                                    None,
                                    0,
                                    0,
                                    &mut table_id,
                                ) == JET_errSuccess
                                {
                                    let id_type_colid: u32 = find_id_column_by_name(
                                        session.clone(),
                                        String::from(table_name),
                                        String::from("IdType"),
                                    );

                                    let id_blob_colid: u32 = find_id_column_by_name(
                                        session.clone(),
                                        String::from(table_name),
                                        String::from("IdBlob"),
                                    );

                                    let mut err =
                                        JetMove(session.ses_id, table_id, JET_MoveFirst as i32, 0);
                                    while err == JET_errSuccess {
                                        let mut id_type: u8 = 0;
                                        let mut cb_actual: u32 = 0;
                                        let _ = JetRetrieveColumn(
                                            session.ses_id,
                                            table_id,
                                            id_type_colid,
                                            Some(&mut id_type as *mut u8 as *mut _),
                                            size_of::<u8>() as u32,
                                            Some(&mut cb_actual),
                                            0,
                                            None,
                                        );

                                        let max_blob_size = 256;
                                        let mut blob_buffer: Vec<u16> = vec![0u16; max_blob_size];
                                        let mut cb_actual_blob: u32 = 0;
                                        let _ = JetRetrieveColumn(
                                            session.ses_id,
                                            table_id,
                                            id_blob_colid,
                                            Some(blob_buffer.as_mut_ptr() as *mut _),
                                            max_blob_size as u32,
                                            Some(&mut cb_actual_blob),
                                            0,
                                            None,
                                        );

                                        blob_buffer.truncate(cb_actual_blob as usize);

                                        match id_type {
                                            33 => {
                                                let string = String::from_utf16_lossy(&blob_buffer);
                                                for line in string.lines() {
                                                    let line = line.trim();
                                                    if line.is_empty() {
                                                        continue;
                                                    }
                                                    let line = if line.starts_with("!!") {
                                                        &line[2..]
                                                    } else {
                                                        line
                                                    };
                                                    if let Some(pos) = line.find('!') {
                                                        let filename = &line[..pos];
                                                        strings.push(
                                                            String::from(filename)
                                                                .replace('\0', ""),
                                                        );
                                                    }
                                                }
                                            }
                                            92 => {
                                                strings.push(
                                                    replace_device_path_with_drive_letter(
                                                        &String::from_utf16_lossy(&blob_buffer),
                                                    )
                                                    .replace('\0', ""),
                                                );
                                            }
                                            77 => {
                                                // skipping (windows apps)
                                            }
                                            _ => {}
                                        }

                                        err = JetMove(session.ses_id, table_id, 2, 0);
                                    }

                                    let _ = JetCloseTable(session.ses_id, table_id);
                                }
                            }
                        }
                    }
                }

                session.close();

                let _ = fs::remove_file(path);
                let parent = path.parent();
                if let Some(parent) = parent {
                    let f_n = path.file_name();
                    if let Some(f_n) = f_n {
                        let f_n = f_n.to_str().unwrap();
                        let _ = fs::remove_file(parent.join(format!("{}.jfm", f_n)));
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

    strings
}

unsafe fn find_id_column_by_name(session: JetSession, table_name: String, name: String) -> u32 {
    let mut buf: JET_COLUMNDEF = zeroed();

    let _ = JetGetColumnInfoW(
        session.ses_id,
        session.dbid,
        string_to_pcwstr(table_name).0,
        Some(string_to_pcwstr(name).0),
        &mut buf as *mut _ as *mut c_void,
        size_of::<JET_COLUMNDEF>() as u32,
        1,
    );

    buf.columnid
}
