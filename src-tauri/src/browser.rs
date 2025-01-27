use std::collections::HashMap;

use browser::{get_browsers, Browser};
use dat::{CacheDat, DownloadDat, VisitDat};
use driver::{BlinkDriverReader, Driver, Reader};

pub mod browser;
pub mod dat;
pub mod driver;
pub mod tests;

#[tauri::command]
pub fn get_supported_browsers() -> Vec<Browser> {
    get_browsers()
}

#[tauri::command]
pub fn get_browser_visit_data(browser_id: String) -> Vec<VisitDat> {
    if let Some(mut reader) = get_browser_reader(browser_id) {
        if reader.init() {
            return reader.wrap_visit();
        }
    }

    vec![]
}

#[tauri::command]
pub fn get_browser_cache_data(browser_id: String) -> Vec<CacheDat> {
    if let Some(mut reader) = get_browser_reader(browser_id) {
        if reader.init() {
            return reader.wrap_cache();
        }
    }

    vec![]
}

#[tauri::command]
pub fn get_browser_download_data(browser_id: String) -> Vec<DownloadDat> {
    if let Some(mut reader) = get_browser_reader(browser_id) {
        if reader.init() {
            return reader.wrap_downloads();
        }
    }

    vec![]
}

fn get_browser_reader(browser_id: String) -> Option<Reader> {
    let binding = get_browsers();

    let browser = binding.iter().filter(|b| b.id.eq(&browser_id)).last();

    if let Some(browser) = browser {
        return if browser.driver.eq(&Driver::BLINK) {
            Some(Reader {
                employee: Box::new(BlinkDriverReader {}),
                browser: browser.clone(),
                temporary: HashMap::new(),
            })
        } else {
            None
        };
    }

    None
}
