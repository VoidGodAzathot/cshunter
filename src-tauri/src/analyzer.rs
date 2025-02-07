use std::{fs::File, io::Write, path::Path};

use analyzer::Analyzer;
use context::{load_context, load_context_from_url, AnalyzerContext};

use crate::emitter::global_emit;

pub mod analyzer;
pub mod context;

#[tauri::command(async)]
pub async fn create_analyzer_context_from_url(url: String) -> Option<AnalyzerContext> {
    load_context_from_url(url).await
}

#[tauri::command(async)]
pub fn create_analyzer_context(path: String) -> Option<AnalyzerContext> {
    global_emit("task_status_update", "initialization");

    load_context(path)
}

#[tauri::command(async)]
pub fn generate_context(files: Vec<String>) -> Option<AnalyzerContext> {
    Analyzer::generate_context(files)
}

#[tauri::command(async)]
pub fn save_context(dir: String, context: AnalyzerContext) {
    match File::create(Path::new(&dir).join("context.json")) {
        Ok(mut file) => match serde_json::to_string(&context) {
            Ok(value) => {
                let _ = file.write_all(value.as_bytes());
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
}