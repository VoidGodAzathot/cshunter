#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, time::Instant};

    use crate::analyzer::{analyzer::Analyzer, context::{AnalyzerContext, ItemContext}};

    #[test]
    fn main_test() {
        let start = Instant::now();
        let path: String = String::from("C:\\");
        let context = Analyzer::generate_context(String::from(path.clone()));
        let _ = File::create(format!("{}\\context.json", path.clone())).unwrap().write_all(serde_json::to_string(&context.unwrap()).unwrap().as_bytes());
        println!("{:?}", start.elapsed());
    }
}
