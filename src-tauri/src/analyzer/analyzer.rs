use std::{fmt::Debug, fs::File, io::Read, path::Path};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::utils::get_parallel_files;

use super::context::{AnalyzerContext, ItemContext};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    pub items: Vec<ItemContext>,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Empty {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Payload<T> {
    pub task_id: u64,
    pub _type: String,
    pub data: T,
}

pub struct Analyzer {
    pub context: AnalyzerContext,
}

impl Analyzer {
    pub fn new(context: AnalyzerContext) -> Self {
        Self { context: context }
    }

    pub fn create_file_context(name: String, path: String, with_path: bool) -> Option<ItemContext> {
        #[inline]
        fn log_error(e: &impl Debug) {
            if cfg!(dev) {
                println!("{e:?}");
            }
        }

        let mut file_map = || -> Option<File> {
            let file = File::open(&path).inspect_err(log_error).ok()?;
            let file_size = file.metadata().inspect_err(log_error).ok()?.len();

            if file_size > 128 * 1024 * 1024 {
                return None;
            }

            Some(file)
        }()?;

        let mut buf: Vec<u8> = vec![];
        let _ = file_map.read_to_end(&mut buf);
        let pe_crc = crc32fast::hash(buf.as_slice());

        Some(ItemContext {
            name,
            path: if with_path { path } else { String::new() },
            size: if file_map.metadata().is_ok() {
                file_map.metadata().unwrap().len()
            } else {
                0
            },
            crc32: pe_crc,
        })
    }

    pub fn generate_context_from_folder(start_path: String) -> Option<AnalyzerContext> {
        let items: Vec<_> = get_parallel_files(start_path)
            .into_par_iter()
            .filter_map(|file| {
                let path = Path::new(&file);

                match path.extension() {
                    Some(ext) if ext == "dll" || ext == "exe" => {}
                    _ => return None,
                }

                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_owned())
                    .unwrap_or_else(|| "undefined".into());

                Self::create_file_context(file_name, file, false)
            })
            .collect();

        (!items.is_empty()).then(|| AnalyzerContext { items })
    }

    pub fn generate_context(files: Vec<String>) -> Option<AnalyzerContext> {
        use rayon::prelude::*;

        let items: Vec<_> = files
            .into_par_iter()
            .filter_map(|file| {
                let path = Path::new(&file);

                match path.extension()?.to_str()? {
                    "dll" | "exe" => Some(()),
                    _ => None,
                }?;

                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_owned())
                    .unwrap_or_else(|| "undefined".to_string());

                Self::create_file_context(file_name, file, true)
            })
            .collect();

        (!items.is_empty()).then(|| AnalyzerContext { items })
    }
}
