use std::{fs::File, io::Write, path::Path, thread};

use analyzer::Analyzer;
use context::{load_context, load_context_from_url, AnalyzerContext};
use tauri::Window;

pub mod analyzer;
pub mod context;
pub mod tests;

#[tauri::command(async)]
pub async fn create_analyzer_context_from_url(url: String) -> Option<AnalyzerContext> {
    load_context_from_url(url).await
}

#[tauri::command]
pub fn create_analyzer_context(path: String) -> Option<AnalyzerContext> {
    load_context(path)
}

#[tauri::command]
pub fn generate_context(files: Vec<String>) -> Option<AnalyzerContext> {
    Analyzer::generate_context(files)
}

#[tauri::command]
pub fn save_context(dir: String, context: AnalyzerContext) {
    match File::create(Path::new(&dir).join("context.json")) {
        Ok(mut file) => {
            match serde_json::to_string(&context) {
                Ok(value) => {
                    let _ = file.write_all(value.as_bytes());
                },
                
                Err(e) => {
                    if cfg!(dev) {
                        println!("{e:?}");
                    }
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

#[tauri::command(async)]
pub fn run_analyzer(context: AnalyzerContext, start_path: String, emitter: Window) {
    thread::spawn(|| {
        let analyzer = Analyzer::new(context);
        analyzer.run_analyze(start_path, emitter);
    });
}
