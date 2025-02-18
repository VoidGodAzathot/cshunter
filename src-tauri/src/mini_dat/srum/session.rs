use std::{env, fs, mem::zeroed};

use windows::Win32::Storage::Jet::{
    JET_bitDbReadOnly, JET_errSuccess, JetAttachDatabaseW, JetBeginSessionW, JetCloseDatabase,
    JetCreateInstance2W, JetDetachDatabaseW, JetEndSession, JetInit, JetOpenDatabaseW, JetTerm,
    JET_INSTANCE, JET_SESID,
};

use crate::utils::string_to_pcwstr;

#[derive(Clone)]
pub struct JetSession {
    pub instance: JET_INSTANCE,
    pub ses_id: JET_SESID,
    pub dbid: u32,
    db_path: Option<String>,
}

impl JetSession {
    pub fn open_instance(instance_name: &str) -> Option<Self> {
        unsafe {
            let mut instance: JET_INSTANCE = zeroed();
            let instance_wide = string_to_pcwstr(instance_name.to_string());
            let instance_name_ptr = Some(instance_wide.as_ptr());

            if JetCreateInstance2W(&mut instance, instance_name_ptr, instance_name_ptr, 0)
                != JET_errSuccess
            {
                None
            } else {
                Some(Self {
                    instance,
                    ses_id: zeroed(),
                    dbid: 0,
                    db_path: None,
                })
            }
        }
    }

    pub fn init(&mut self) -> bool {
        unsafe { JetInit(Some(&mut self.instance)) == JET_errSuccess }
    }

    pub fn begin(&mut self) -> bool {
        unsafe { JetBeginSessionW(self.instance, &mut self.ses_id, None, None) == JET_errSuccess }
    }

    pub fn attach_and_open_to_db(&mut self, path: &str) -> bool {
        unsafe {
            let path_wide = string_to_pcwstr(path.to_string());
            let path_ptr = path_wide.as_ptr();

            let attach_err = JetAttachDatabaseW(self.ses_id, path_ptr, JET_bitDbReadOnly);
            if cfg!(dev) {
                println!("JetAttachDatabaseW: {}", attach_err);
            }

            let err = JetOpenDatabaseW(
                self.ses_id,
                path_ptr,
                None,
                &mut self.dbid,
                JET_bitDbReadOnly,
            );
            if cfg!(dev) {
                println!("JetOpenDatabaseW: {}", err);
            }

            self.db_path = Some(path.to_string());
            err == JET_errSuccess
        }
    }

    pub fn close(&mut self) {
        unsafe {
            if JetCloseDatabase(self.ses_id, self.dbid, 0) != JET_errSuccess {
                if cfg!(debug_assertions) {
                    println!("close db is failed: {:?}", self.instance);
                }
            }

            if let Some(ref path) = self.db_path {
                let path_wide = string_to_pcwstr(path.clone());
                let path_ptr = path_wide.as_ptr();
                if JetDetachDatabaseW(self.ses_id, Some(path_ptr)) != JET_errSuccess {
                    if cfg!(dev) {
                        println!("detach db is failed: {:?}", self.instance);
                    }
                }
            }

            if JetEndSession(self.ses_id, 0) != JET_errSuccess {
                if cfg!(dev) {
                    println!("end session is failed: {:?}", self.instance);
                }
            }

            if JetTerm(self.instance) != JET_errSuccess {
                if cfg!(dev) {
                    println!("terminate instance is failed: {:?}", self.instance);
                }
            }

            if let Ok(dir) = env::current_dir() {
                if let Ok(read_dir) = dir.read_dir() {
                    for entry in read_dir.filter_map(Result::ok) {
                        if let Some(fname) = entry.file_name().to_str() {
                            if fname.contains("edb")
                                && (fname.ends_with(".chk")
                                    || fname.ends_with(".log")
                                    || fname.ends_with(".jrs"))
                            {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
    }
}
