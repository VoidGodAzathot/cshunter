#[cfg(test)]
mod tests {
    use crate::process::{
        dump::{dump_modules_strings_from_process, dump_strings_from_process},
        process::{enable_debug_privilege, Process},
    };

    #[test]
    fn test() {
        enable_debug_privilege();
        let process = Process::find_by_name("chrome");
        if process.is_some() {
            let process = process.unwrap();
            let mut summary = 0;
            println!("--- modules start ---");
            for module in dump_modules_strings_from_process(process) {
                println!(
                    "{} ({}) (строк: {})",
                    module.module,
                    module.address,
                    module.values.len()
                );
                summary += module.values.len();
            }
            println!("{}", summary);
            println!("--- modules end ---");
        }

        let process = Process::find_by_name("chrome");
        if process.is_some() {
            let process = process.unwrap();
            println!("--- regions start ---");
            for string in dump_strings_from_process(process) {
                println!("({}): {}", string.address, string.values.len());
            }
            println!("--- regions end ---");
        }
    }
}
