#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::utils;

    #[test]
    fn get_files_benchmark() {
        let start = Instant::now();
        println!("{}", utils::get_parallel_files(String::from("C:\\")).len());
        println!("{:?}", start.elapsed());
    }
}
