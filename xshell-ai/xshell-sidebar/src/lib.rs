pub fn crate_name() -> &'static str {
    "xshell-sidebar"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_crate_name() {
        assert_eq!(crate_name(), "xshell-sidebar");
    }
}
