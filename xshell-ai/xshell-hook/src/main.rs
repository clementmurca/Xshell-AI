fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: xshell-hook <event-type>");
        std::process::exit(2);
    }
    eprintln!("xshell-hook stub: event={}", args[1]);
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_crate_name() {
        assert_eq!(env!("CARGO_PKG_NAME"), "xshell-hook");
    }
}
