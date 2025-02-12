use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellBagDat {
    pub path: String
}

#[derive(Debug)]
pub struct BagMRU {
    pub path: String,
    pub entry: Vec<u8>,
    pub sub: Vec<BagMRU>,
    pub full_name: Option<String>,
    pub short_name: Option<String>
}