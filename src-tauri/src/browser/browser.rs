use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::driver::Driver;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Browser {
    pub id: String,
    pub path: PathBuf,
    pub driver: Driver,
    pub support: bool,
}

impl Browser {
    pub fn new(id: String, path: PathBuf, driver: Driver) -> Self {
        Self {
            id: id,
            path: path.clone(),
            driver: driver,
            support: Path::exists(&path),
        }
    }

    pub fn is_present(&self) -> bool {
        self.support
    }
}

pub fn get_browsers() -> Vec<Browser> {
    let app_data_dir = PathBuf::from(format!("C:\\Users\\{}\\AppData", whoami::username()));

    return vec![
        Browser::new(
            String::from("chrome"),
            app_data_dir.join("Local\\Google\\Chrome\\User Data\\Default"),
            Driver::BLINK,
        ),
        Browser::new(
            String::from("yandex"),
            app_data_dir.join("Local\\Yandex\\YandexBrowser\\User Data\\Default"),
            Driver::BLINK,
        ),
        Browser::new(
            String::from("edge"),
            app_data_dir.join("Local\\Microsoft\\Edge\\User Data\\Default"),
            Driver::BLINK,
        ),
        Browser::new(
            String::from("opera"),
            app_data_dir.join("Roaming\\Opera Software\\Opera Stable\\Default"),
            Driver::BLINK,
        ),
        Browser::new(
            String::from("brave"),
            app_data_dir.join("Local\\BraveSoftware\\Brave-Browser\\User Data\\Default"),
            Driver::BLINK,
        )
    ];
}
