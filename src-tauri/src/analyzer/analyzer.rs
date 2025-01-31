use std::{fmt::Debug, fs::File, io::Read, path::Path};

use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Window};

use crate::{emit::events::ANALYZER_EMIT_EVENT, utils::get_parallel_files};

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

    pub fn run_analyze(&self, start_path: String, emitter: Window) {
        let id: u16 = rand::random_range(0..u16::MAX);

        let _ = emitter.emit(
            ANALYZER_EMIT_EVENT,
            Payload::<Empty> {
                task_id: id as u64,
                _type: String::from("start"),
                data: Empty {},
            },
        );

        let targets = get_parallel_files(start_path);

        targets.iter().par_bridge().for_each(|target| {
            let file_context =
                Analyzer::create_file_context(String::new(), target.to_string(), false);

            if file_context.is_some() {
                let file_context = file_context.unwrap();

                let matches = self
                    .context
                    .items
                    .iter()
                    .par_bridge()
                    .filter(|item| item.to_owned().to_owned() == file_context)
                    .map(|item| item.to_owned())
                    .collect::<Vec<ItemContext>>();

                if matches.len() != 0 {
                    let _match = Match {
                        items: matches,
                        path: target.clone(),
                    };

                    let _ = emitter.emit(
                        ANALYZER_EMIT_EVENT,
                        Payload::<Match> {
                            task_id: id as u64,
                            _type: String::from("match"),
                            data: _match,
                        },
                    );
                }
            }
        });

        let _ = emitter.emit(
            ANALYZER_EMIT_EVENT,
            Payload::<Empty> {
                task_id: id as u64,
                _type: String::from("stop"),
                data: Empty {},
            },
        );
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
