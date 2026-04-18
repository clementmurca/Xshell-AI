pub fn crate_name() -> &'static str {
    "xshell-themes"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_crate_name() {
        assert_eq!(crate_name(), "xshell-themes");
    }
}
