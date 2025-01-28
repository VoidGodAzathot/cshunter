use std::thread;

use analyzer::Analyzer;
use context::{load_context, load_context_from_url, AnalyzerContext};
use tauri::Window;

pub mod context;
pub mod analyzer;
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
pub fn generate_context_from_folder(path: String) -> Option<AnalyzerContext> {
    Analyzer::generate_context(path)
}

#[tauri::command(async)]
pub fn run_analyzer(context: AnalyzerContext, start_path: String, emitter: Window) {
    thread::spawn(|| {
        let analyzer = Analyzer::new(context);
        analyzer.run_analyze(start_path, emitter);
    });
}
