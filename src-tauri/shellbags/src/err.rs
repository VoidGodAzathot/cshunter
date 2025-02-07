use windows::Win32::Foundation::WIN32_ERROR;

use crate::shellbag::ShellBagPath;

pub type ShellBagResult<T> = Result<T, ShellBagError>;

#[derive(Debug)]
pub enum ShellBagError {
    /// Generic Win32Error
    Win32Error(WIN32_ERROR),
    /// Permissions error
    Permissions(String),
    /// 
    MalformedData(String)
}

#[derive(Debug)]
pub struct ShellBagErrorResume {
    pub error : ShellBagError,
    pub path : ShellBagPath
}

impl From<WIN32_ERROR> for ShellBagError {
    fn from(err : WIN32_ERROR) -> Self {
        ShellBagError::Win32Error(err)
    }
}