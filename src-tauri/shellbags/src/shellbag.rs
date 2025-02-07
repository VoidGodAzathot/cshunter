use std::{collections::BTreeMap};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ShellFolderItem {
    pub id : u8,
    pub guid : u128,
    pub name : String
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ShellVolumeItem {
    pub name : String
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ShellFileItem {
    pub file_size : u32,
    pub m_time : i64,
    pub c_time : i64,
    pub a_time : i64,
    pub fflags : u16,
    pub short_name : String,
    pub long_name : String,
    pub ext_size : u32,
    pub ext_version : u32,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ShellNetworkItem {
    pub guid : Option<u128>,
    pub flags : u32,
    pub location : String,
    pub description : String,
    pub comment : String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ShellItem {
    Folder(ShellFolderItem),
    Volume(ShellVolumeItem),
    File(ShellFileItem),
    Network(ShellNetworkItem),
    Unknown(u32)
}
impl Default for ShellItem {
    fn default() -> Self {
        Self::Unknown(0)
    }
}
#[derive(Default, Clone,Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct ShellBagPath(pub Vec<u32>);

impl ShellBagPath {
    pub fn as_str(&self) -> String {
        let mut to_ret = String::with_capacity(self.0.len() * 2 + 4);
        for element in &self.0 {
            to_ret.push_str(&format!("/{}",*element));
        }
        to_ret
    }
}
impl Serialize for ShellBagPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            serializer.serialize_str(&self.as_str())
    }
}

/// Data contained under HKEY_CLASSES_ROOT\Local Settings\Software\Microsoft\Windows\Shell\Bags\<xxx>\<KeyName>\<UUID>
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WindowBagInfo {
    pub slot : u32,
    pub uuid : u128,
    pub coll_info : Vec<u8>,
    pub f_flags : u32,
    pub group_by_direction : u32,
    pub group_by_key_fmtid : String,
    pub group_by_key_pid : u32,
    pub group_view : u32,
    pub icon_size : u32,
    pub logical_view_mode : u32,
    pub mode : u32,
    pub rev : u32,
    pub sort : Vec<u8>,
    pub vid : u128
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShellBagList {
    pub list : BTreeMap<ShellBagPath, (Option<u32>,ShellItem)>,
    pub removed_bags : Vec<ShellBagPath>,
    /// (MRUListEx, BagMRUKeys)
    pub mru_anomalies : Option<(Vec<u32>, Vec<u32>)>,
    pub node_slots : BTreeMap<NodeSlot, BTreeMap<String,WindowBagInfo>>
}

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord,)]
pub struct NodeSlot(pub u32);

impl Serialize for NodeSlot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            serializer.serialize_str(&format!("{}",self.0))
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShellBags {
    pub ntuser : ShellBagList,
    pub usr_class : ShellBagList,
}

impl ShellBagList {
    pub fn new() -> Self {
        Self::default()
    }
}