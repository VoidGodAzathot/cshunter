#[cfg(test)]
mod tests {
    use crate::steam::steam::Steam;

    #[test]
    fn main_test() {
        let steam = Steam::new();
        println!("{:?}", steam.get_history_accounts());
    }
}
