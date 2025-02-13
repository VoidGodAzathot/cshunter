use std::{env, fs, mem::zeroed, thread, time::Duration};

use windows::Win32::Storage::Jet::{
    JET_bitDbReadOnly, JET_bitTermComplete, JET_errDatabaseDuplicate, JET_errSuccess,
    JetAttachDatabaseW, JetBeginSessionW, JetCloseDatabase, JetCreateInstance2W, JetEndSession,
    JetInit, JetOpenDatabaseW, JetTerm, JetTerm2, JET_INSTANCE, JET_SESID,
};

use crate::utils::string_to_pcwstr;

#[derive(Clone)]
pub struct JetSession {
    pub instance: JET_INSTANCE,
    pub ses_id: JET_SESID,
    pub dbid: u32,
}

impl JetSession {
    pub fn open_instance(instance_name: &str) -> Option<Self> {
        unsafe {
            let mut instance: JET_INSTANCE = zeroed();
            let instance_name = Some(string_to_pcwstr(String::from(instance_name)).as_ptr());
            if JetCreateInstance2W(&mut instance, instance_name, instance_name, 0) != JET_errSuccess
            {
                None
            } else {
                Some(Self {
                    instance: instance,
                    ses_id: zeroed(),
                    dbid: 0,
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

    pub fn close(&self) {
        unsafe {
            if JetCloseDatabase(self.ses_id, self.dbid, 0) != JET_errSuccess {
                if cfg!(dev) {
                    println!("failed close database on {:?}", self.instance);
                }
            }
            if JetEndSession(self.ses_id, 0) != JET_errSuccess {
                if cfg!(dev) {
                    println!("failed end jet session on {:?}", self.instance);
                }
            }
            if JetTerm(self.instance) != JET_errSuccess {
                if cfg!(dev) {
                    println!("failed terminate jet session on {:?}", self.instance);
                }
            }

            match env::current_dir() {
                Ok(dir) => {
                    let read_dir = dir.read_dir();
                    if let Ok(read_dir) = read_dir {
                        for file in read_dir {
                            if file.is_ok() {
                                let file = file.unwrap();
                                if file.file_name().to_str().unwrap_or("").contains("edb") {
                                    let _ = fs::remove_file(file.path());
                                }
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
    }

    pub fn attach_and_open_to_db(&mut self, path: &str) -> bool {
        unsafe {
            let path_ptr = string_to_pcwstr(String::from(path)).as_ptr();

            let mut attach_attempts = 0;
            loop {
                let attach_result = JetAttachDatabaseW(self.ses_id, path_ptr, JET_bitDbReadOnly);
                if attach_result == JET_errSuccess || attach_result == JET_errDatabaseDuplicate {
                    break;
                } else {
                    if cfg!(dev) {
                        println!(
                            "JetAttachDatabaseW attempt {} failed with error: {}",
                            attach_attempts, attach_result
                        );
                    }
                    thread::sleep(Duration::from_millis(500));
                    attach_attempts += 1;
                    if attach_attempts >= 10 {
                        if cfg!(dev) {
                            println!("Failed to attach database after 10 attempts");
                        }
                        return false;
                    }
                }
            }

            let mut open_attempts = 0;
            loop {
                let open_result = JetOpenDatabaseW(
                    self.ses_id,
                    path_ptr,
                    None,
                    &mut self.dbid,
                    JET_bitDbReadOnly,
                );
                if open_result == JET_errSuccess {
                    return true;
                } else {
                    if cfg!(dev) {
                        println!(
                            "JetOpenDatabaseW attempt {} failed with error: {}",
                            open_attempts, open_result
                        );
                    }
                    let _ = JetTerm2(self.instance, JET_bitTermComplete);
                    thread::sleep(Duration::from_millis(500));
                    open_attempts += 1;
                    if open_attempts >= 10 {
                        if cfg!(dev) {
                            println!("Failed to open database after 10 attempts");
                        }
                        return false;
                    }
                }
            }
        }
    }
}
