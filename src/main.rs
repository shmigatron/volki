fn main() {
    let cli = volki::core::cli::build_cli();
    if let Err(e) = cli.run() {
        use volki::core::cli::style;

        eprintln!(
            "  {} {}",
            style::red(style::CROSS),
            style::red(&e.to_string()),
        );
        if let Some(hint) = e.hint() {
            eprintln!(
                "  {} {}",
                style::dim(style::ARROW),
                hint,
            );
        }
        eprintln!();
        std::process::exit(1);
    }
}
