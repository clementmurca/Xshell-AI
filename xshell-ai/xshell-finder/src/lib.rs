#[cfg(not(target_os = "macos"))]
compile_error!("xshell-finder is macOS only for now");

pub fn crate_name() -> &'static str {
    "xshell-finder"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_crate_name() {
        assert_eq!(crate_name(), "xshell-finder");
    }
}
