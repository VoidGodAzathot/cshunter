#[cfg(test)]
mod tests {
    use crate::shellbag::shellbag::collect_shell_bag;

    #[test]
    fn test() {
        for dat in collect_shell_bag() {
            println!("{}", dat.path);
        }
    }
}