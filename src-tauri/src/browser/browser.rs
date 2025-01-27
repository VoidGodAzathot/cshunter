use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::driver::Driver;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Browser {
    pub id: String,
    pub path: PathBuf,
    pub driver: Driver,
}

impl Browser {
    pub fn is_present(&self) -> bool {
        Path::exists(&self.path)
    }
}

pub fn get_browsers() -> Vec<Browser> {
    let app_data_dir = PathBuf::from(format!("C:\\Users\\{}\\AppData", whoami::username()));

    return vec![
        Browser {
            id: String::from("chrome"),
            path: app_data_dir.join("Local\\Google\\Chrome\\User Data\\Default"),
            driver: Driver::BLINK,
        },
        Browser {
            id: String::from("yandex"),
            path: app_data_dir.join("Local\\Yandex\\YandexBrowser\\User Data\\Default"),
            driver: Driver::BLINK,
        },
        Browser {
            id: String::from("edge"),
            path: app_data_dir.join("Local\\Microsoft\\Edge\\User Data\\Default"),
            driver: Driver::BLINK,
        },
        Browser {
            id: String::from("opera"),
            path: app_data_dir.join("Roaming\\Opera Software\\Opera Stable\\Default"),
            driver: Driver::BLINK,
        },
        Browser {
            id: String::from("brave"),
            path: app_data_dir.join("Local\\BraveSoftware\\Brave-Browser\\User Data\\Default"),
            driver: Driver::BLINK,
        },
    ];
}
