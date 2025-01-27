use std::{collections::HashMap, env, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::utils::random_name;

use super::{
    browser::Browser,
    dat::{CacheDat, DownloadDat, VisitDat},
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Driver {
    BLINK,
    QUANTUM,
}

pub struct Reader {
    pub employee: Box<dyn DriverReader>,
    pub browser: Browser,
    pub temporary: HashMap<String, String>,
}

pub trait DriverReader {
    fn read_downloads(&self, browser: Browser, temporary: HashMap<String, String>) -> Vec<DownloadDat>;
    fn read_visit(&self, browser: Browser, temporary: HashMap<String, String>) -> Vec<VisitDat>;
    fn read_cache(&self, browser: Browser, temporary: HashMap<String, String>) -> Vec<CacheDat>;
}

impl Reader {
    pub fn init(&mut self) -> bool {
        if !self.browser.is_present() {
            return false;
        }

        let path_history = &self.browser.path.join("History");
        let path_dips = &self.browser.path.join("DIPS");

        if !Path::exists(path_history) || !Path::exists(path_dips) {
            return false;
        }

        let temp_path_history = &env::temp_dir().join(random_name());
        let temp_path_dips = &env::temp_dir().join(random_name());

        match fs::copy(path_history, temp_path_history) {
            Ok(_) => {
                self.temporary.insert(
                    String::from("history"),
                    temp_path_history.to_string_lossy().to_string(),
                );
            }
            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }

                return false;
            }
        }

        match fs::copy(path_dips, temp_path_dips) {
            Ok(_) => {
                self.temporary.insert(
                    String::from("dips"),
                    temp_path_dips.to_string_lossy().to_string(),
                );
            }
            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }

                return false;
            }
        }

        true
    }

    pub fn wrap_downloads(&self) -> Vec<DownloadDat> {
        if !self.browser.is_present() {
            return vec![];
        }

        self.employee.read_downloads(self.browser.clone(), self.temporary.clone())
    }

    pub fn wrap_visit(&self) -> Vec<VisitDat> {
        if !self.browser.is_present() {
            return vec![];
        }

        self.employee.read_visit(self.browser.clone(), self.temporary.clone())
    }

    pub fn wrap_cache(&self) -> Vec<CacheDat> {
        if !self.browser.is_present() {
            return vec![];
        }

        self.employee.read_cache(self.browser.clone(), self.temporary.clone())
    }
}

pub struct BlinkDriverReader;

impl DriverReader for BlinkDriverReader {
    fn read_downloads(&self, browser: Browser, temporary: HashMap<String, String>) -> Vec<DownloadDat> {
        let mut response = vec![];

        match sqlite::open(temporary.get("history").unwrap()) {
            Ok(connection) => {
                for row in connection
                    .prepare("SELECT * FROM downloads")
                    .unwrap()
                    .into_iter()
                    .map(|row| row.unwrap())
                {
                    response.push(DownloadDat {
                        browser: browser.id.clone(),
                        file: String::from(row.read::<&str, _>(3)),
                        url: String::from(row.read::<&str, _>(18)),
                        timestamp: row.read::<i64, _>(11),
                    });
                }

                return response;
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        response
    }

    fn read_visit(&self, browser: Browser, temporary: HashMap<String, String>) -> Vec<VisitDat> {
        let mut response = vec![];

        match sqlite::open(temporary.get("history").unwrap()) {
            Ok(connection) => {
                for row in connection
                    .prepare("SELECT * FROM urls")
                    .unwrap()
                    .into_iter()
                    .map(|row| row.unwrap())
                {
                    response.push(VisitDat {
                        browser: browser.id.clone(),
                        title: String::from(row.read::<&str, _>(2)),
                        url: String::from(row.read::<&str, _>(1)),
                        timestamp: row.read::<i64, _>(5),
                    });
                }

                return response;
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        response
    }

    fn read_cache(&self, browser: Browser, temporary: HashMap<String, String>) -> Vec<CacheDat> {
        let mut response = vec![];

        match sqlite::open(temporary.get("dips").unwrap()) {
            Ok(connection) => {
                for row in connection
                    .prepare("SELECT * FROM bounces")
                    .unwrap()
                    .into_iter()
                    .map(|row| row.unwrap())
                {
                    response.push(CacheDat {
                        browser: browser.id.clone(),
                        url: String::from(row.read::<&str, _>(0)),
                    });
                }

                return response;
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        response
    }
}
