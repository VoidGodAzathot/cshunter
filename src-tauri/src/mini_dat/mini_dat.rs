use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MiniDat {
    pub value: String,
    pub id: &'static str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MiniDatInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub stable: bool,
    pub filtering: bool
}

pub trait MiniDatWrapper {
    fn new_instance(value: String) -> MiniDat;
}

pub trait MiniDatEmployee<E>
{
    fn run() -> Vec<E>;
}