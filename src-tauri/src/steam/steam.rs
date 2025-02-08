use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use windows_registry::{CURRENT_USER, LOCAL_MACHINE};

use crate::steam::convert::{convert_from_login_user, Convertable};

use super::tree::Tree;

pub struct Steam {
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SteamAccount {
    pub id: String,
    pub persona_name: String,
    pub account_name: String,
    pub most_recent: u16,
    pub timestamp: u32,
}

impl SteamAccount {
    pub async fn is_vac(&self) -> bool {
        match reqwest::get(format!("https://steamcommunity.com/id/{}", self.id)).await {
            Ok(response) => {
                let re = Regex::new(r#"<div class="profile_ban">([^">]+)"#).unwrap();
                let text = &response.text().await.unwrap();
                return re.is_match(text);
            }

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        false
    }
}

impl Steam {
    pub fn new() -> Self {
        Self {
            location: Steam::find_steam(),
        }
    }

    fn find_steam() -> Option<String> {
        match CURRENT_USER.open("SOFTWARE\\Valve\\Steam") {
            Ok(key) => match key.get_string("SteamPath") {
                Ok(path) => {
                    return Some(path);
                }

                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}");
                    }
                }
            },

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        match LOCAL_MACHINE.open("SOFTWARE\\WOW6432Node\\Valve\\Steam") {
            Ok(key) => match key.get_string("InstallPath") {
                Ok(path) => {
                    return Some(path);
                }

                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}");
                    }
                }
            },

            Err(e) => {
                if cfg!(dev) {
                    println!("{e:?}");
                }
            }
        }

        None
    }

    pub fn get_avatar_cache(&self) -> Vec<String> {
        if self.location.is_some() {
            match fs::read_dir(format!(
                "{}\\config\\avatarcache",
                self.location.clone().unwrap()
            )) {
                Ok(dir) => {
                    let mut response = vec![];

                    for val in dir {
                        match val {
                            Ok(val) => {
                                let name =
                                    val.file_name().to_str().unwrap_or("undefined").to_string();
                                
                                if !name.eq("undefined") && name.contains(".") {
                                    let id_and_ext = name.split(".").collect::<Vec<&str>>();
                                    if id_and_ext.len() == 2 {
                                        response.push(id_and_ext[0].to_string());
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

                    return response;
                }

                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}");
                    }
                }
            }
        }

        vec![]
    }

    pub fn get_history_accounts(&self) -> Vec<SteamAccount> {
        if self.location.is_some() {
            match File::open(format!(
                "{}\\config\\loginusers.vdf",
                self.location.clone().unwrap()
            )) {
                Ok(mut file) => {
                    let mut buf: String = String::new();
                    let _ = File::read_to_string(&mut file, &mut buf);
                    let mut tree = Tree::new(buf);
                    tree.parse();

                    {
                        struct _Unknown_ {
                            users: Vec<SteamAccount>,
                        }

                        impl Convertable for _Unknown_ {
                            fn generate(map: HashMap<String, HashMap<String, String>>) -> Self {
                                Self {
                                    users: map
                                        .keys()
                                        .map(|k| {
                                            let val = map.get_key_value(k).unwrap();

                                            SteamAccount {
                                                id: k.to_string(),
                                                account_name: val
                                                    .1
                                                    .get("AccountName")
                                                    .unwrap()
                                                    .to_string(),
                                                persona_name: val
                                                    .1
                                                    .get("PersonaName")
                                                    .unwrap()
                                                    .to_string(),
                                                most_recent: val
                                                    .1
                                                    .get("MostRecent")
                                                    .unwrap()
                                                    .to_string()
                                                    .parse()
                                                    .unwrap_or(0),
                                                timestamp: val
                                                    .1
                                                    .get("Timestamp")
                                                    .unwrap()
                                                    .to_string()
                                                    .parse()
                                                    .unwrap_or(0),
                                            }
                                        })
                                        .collect(),
                                }
                            }
                        }

                        let result = convert_from_login_user::<_Unknown_>(tree);
                        return result.unwrap().users;
                    }
                }

                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}");
                    }
                }
            }
        }

        vec![]
    }
}
