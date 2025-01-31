#[cfg(test)]
mod tests {
    use crate::device_id::device_id::DeviceId;

    #[test]
    fn main_test() {
        match DeviceId::generate() {
            Ok(device_id) => {
                println!("{:?}", device_id.pairs);
                println!("{}", device_id.to_string());
            }

            Err(_) => {}
        };
    }
}
