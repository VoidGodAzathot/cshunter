use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadDat {
    pub browser: String,
    pub file: String,
    pub url: String,
    pub timestamp: i64
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VisitDat {
    pub browser: String,
    pub title: String,
    pub url: String,
    pub timestamp: i64
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CacheDat {
    pub browser: String,
    pub url: String
}