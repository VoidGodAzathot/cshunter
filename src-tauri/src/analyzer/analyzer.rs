use std::{
    fmt::Debug,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use memmap2::Mmap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{emitter::global_emit, utils::get_parallel_files};

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

        if name != "undefined" {
            global_emit("task_status_update", &name);
        }

        let file = File::open(&path).map_err(|e| log_error(&e)).ok()?;
        let metadata = file.metadata().map_err(|e| log_error(&e)).ok()?;
        let file_size = metadata.len();

        if file_size > 96 * 1024 * 1024 {
            return None;
        }

        let crc32 = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => crc32fast::hash(&mmap),
            Err(e) => {
                log_error(&e);
                let file = File::open(&path).map_err(|e| log_error(&e)).ok()?;
                let mut reader = BufReader::new(file);
                let mut hasher = crc32fast::Hasher::new();
                let mut buffer = [0u8; 8192];
                loop {
                    let bytes_read = reader.read(&mut buffer).map_err(|e| log_error(&e)).ok()?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                hasher.finalize()
            }
        };

        Some(ItemContext {
            name,
            path: if with_path { path } else { String::new() },
            size: file_size,
            crc32,
        })
    }

    pub fn generate_context_from_folder(start_path: String) -> Option<AnalyzerContext> {
        let items: Vec<_> = get_parallel_files(start_path)
            .par_iter()
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

                Self::create_file_context(file_name, file.to_string(), false)
            })
            .collect();

        (!items.is_empty()).then(|| AnalyzerContext { items })
    }

    pub fn generate_context(files: Vec<String>) -> Option<AnalyzerContext> {
        let items: Vec<_> = files
            .par_iter()
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

                Self::create_file_context(file_name, file.to_string(), true)
            })
            .collect();

        (!items.is_empty()).then(|| AnalyzerContext { items })
    }
}
