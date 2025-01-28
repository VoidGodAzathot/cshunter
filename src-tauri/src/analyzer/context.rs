use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalyzerContext {
    pub items: Vec<ItemContext>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemContext {
    pub name: String,
    pub size: u64,
    pub crc32: Vec<u32>,
    pub tls: u32,
}

impl PartialEq for ItemContext {
    fn eq(&self, other: &Self) -> bool {
        if !((self.size + 1) >= other.size) {
            return false;
        }

        if self.crc32.eq(&other.crc32) {
            return true;
        }

        false
    }
}

impl Eq for ItemContext {}

pub fn load_context(path: String) -> Option<AnalyzerContext> {
    if !Path::new(&path).exists() || !path.ends_with(".json") {
        return None;
    }

    let mut buf = String::new();
    let _ = File::open(path).unwrap().read_to_string(&mut buf);

    match serde_json::from_str::<AnalyzerContext>(&buf) {
        Ok(context) => {
            return Some(context);
        }

        Err(e) => {
            if cfg!(dev) {
                println!("{e:?}");
            }
        }
    };

    None
}

pub async fn load_context_from_url(url: String) -> Option<AnalyzerContext> {
    match reqwest::get(url).await {
        Ok(response) => {
            let raw_text = response.text().await;

            if raw_text.is_ok() {
                match serde_json::from_str::<AnalyzerContext>(&raw_text.unwrap()) {
                    Ok(context) => {
                        return Some(context);
                    }

                    Err(e) => {
                        if cfg!(dev) {
                            println!("{e:?}");
                        }
                    }
                };
            }
        }

        Err(e) => {
            if cfg!(dev) {
                println!("{e:?}");
            }
        }
    }

    None
}
