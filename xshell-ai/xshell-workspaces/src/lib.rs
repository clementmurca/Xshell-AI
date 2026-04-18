pub fn crate_name() -> &'static str {
    "xshell-workspaces"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_crate_name() {
        assert_eq!(crate_name(), "xshell-workspaces");
    }
}
