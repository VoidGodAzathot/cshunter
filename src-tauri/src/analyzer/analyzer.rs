use std::{ffi::OsStr, fs::File, path::Path};

use pelite::{FileMap, PeFile};
use rayon::iter::{ParallelBridge, ParallelIterator};
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
        let id: u64 = rand::random_range(0..u64::MAX);

        let _ = emitter.emit(
            ANALYZER_EMIT_EVENT,
            Payload::<Empty> {
                task_id: id,
                _type: String::from("start"),
                data: Empty {},
            },
        );

        let targets = get_parallel_files(start_path);

        targets.iter().par_bridge().for_each(|target| {
            let file_context = Analyzer::create_file_context(String::new(), target.to_string());

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
                            task_id: id,
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
                task_id: id,
                _type: String::from("stop"),
                data: Empty {},
            },
        );
    }

    pub fn create_file_context(name: String, path: String) -> Option<ItemContext> {
        if !Path::new(&path).exists() || (!path.ends_with(".dll") && !path.ends_with(".exe")) {
            return None;
        }

        match &File::open(&path) {
            Ok(file) => {
                let file_size = File::metadata(file).unwrap().len();

                if file_size > 128 * 1024 * 1024 {
                    return None;
                }

                match FileMap::open(&path) {
                    Ok(file_map) => {
                        let pe_crc = crc32fast::hash(file_map.as_ref());
                        let mut tls = None;

                        match PeFile::from_bytes(file_map.as_ref()) {
                            Ok(file) => match file.tls() {
                                Ok(_tls) => {
                                    tls = Some(_tls);
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

                        if pe_crc != 0 {
                            let crc32 = vec![pe_crc];
                            let tls = if tls.is_some() {
                                crc32fast::hash(tls.unwrap().raw_data().unwrap_or(&[0u8; 0]))
                            } else {
                                0
                            };

                            return Some(ItemContext {
                                name: name,
                                size: file_size,
                                crc32: crc32,
                                tls: tls,
                            });
                        }
                    }

                    Err(e) => {
                        if cfg!(dev) {
                            println!("{e:?}");
                        }
                    }
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

    pub fn generate_context(start_path: String) -> Option<AnalyzerContext> {
        if !Path::new(&start_path).exists() {
            return None;
        }

        let items: Vec<ItemContext> = get_parallel_files(start_path)
            .iter()
            .par_bridge()
            .filter(|file| file.ends_with(".dll") || file.ends_with(".exe"))
            .map(|file| {
                Analyzer::create_file_context(
                    Path::new(file)
                        .file_name()
                        .unwrap_or(&OsStr::new("undefined"))
                        .to_string_lossy()
                        .to_string(),
                    file.to_string(),
                )
            })
            .filter(|item| item.is_some())
            .map(|item| item.unwrap())
            .collect();

        Some(AnalyzerContext { items: items })
    }
}
