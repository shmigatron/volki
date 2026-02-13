use super::command::Command;
use super::style;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_top_level(commands: &[&dyn Command]) {
    eprintln!();
    eprintln!(
        "  {} {} {}",
        style::WOLF,
        style::bold("volki"),
        style::dim(&format!("v{VERSION}"))
    );
    eprintln!(
        "  {}",
        style::dim("code quality companion")
    );
    eprintln!();
    eprintln!(
        "  {}  volki <command> [options]",
        style::bold("usage:")
    );
    eprintln!();
    eprintln!("  {}", style::bold("commands:"));

    let max_width = commands.iter().map(|c| c.name().len()).max().unwrap_or(0);
    for cmd in commands {
        eprintln!(
            "    {}    {}",
            style::cyan(&format!("{:width$}", cmd.name(), width = max_width)),
            style::dim(cmd.description()),
        );
    }

    eprintln!();
    eprintln!("  {}", style::bold("global options:"));
    eprintln!(
        "    {}    {}",
        style::cyan(&format!("{:<12}", "--help")),
        style::dim("print help information"),
    );
    eprintln!(
        "    {}    {}",
        style::cyan(&format!("{:<12}", "--version")),
        style::dim("print version information"),
    );
    eprintln!(
        "    {}    {}",
        style::cyan(&format!("{:<12}", "--no-color")),
        style::dim("disable colored output"),
    );

    eprintln!();
    eprintln!(
        "  {}  https://volki.dev {}",
        style::bold("docs:"),
        style::PAW,
    );
    eprintln!();
}

pub fn print_command_help(cmd: &dyn Command) {
    eprintln!();
    eprintln!(
        "  {} volki {}",
        style::WOLF,
        style::bold(cmd.name()),
    );
    eprintln!();
    eprintln!("  {}", cmd.long_description());
    eprintln!();
    eprintln!(
        "  {}  volki {}{}",
        style::bold("usage:"),
        cmd.name(),
        format_usage_options(cmd),
    );

    let options = cmd.options();
    if !options.is_empty() {
        eprintln!();
        eprintln!("  {}", style::bold("options:"));

        let max_width = options
            .iter()
            .map(|o| {
                if o.takes_value {
                    o.name.len() + o.name.len() + 5 // --name <NAME>
                } else {
                    o.name.len() + 2 // --name
                }
            })
            .max()
            .unwrap_or(0);

        for opt in &options {
            let flag_str = if opt.takes_value {
                format!("--{} <{}>", opt.name, opt.name.to_uppercase())
            } else {
                format!("--{}", opt.name)
            };

            let mut desc = opt.description.to_string();
            if let Some(def) = opt.default_value {
                desc.push_str(&format!(" {}", style::dim(&format!("[default: {def}]"))));
            }
            if opt.required {
                desc.push_str(&format!(" {}", style::yellow("(required)")));
            }

            eprintln!(
                "    {}    {}",
                style::cyan(&format!("{:<width$}", flag_str, width = max_width)),
                desc,
            );
        }
    }

    eprintln!();
    eprintln!("  {}", style::bold("global options:"));
    eprintln!(
        "    {}    {}",
        style::cyan("--help"),
        style::dim("print help information"),
    );

    eprintln!();
    eprintln!(
        "  {}  https://volki.dev {}",
        style::bold("docs:"),
        style::PAW,
    );
    eprintln!();
}

fn format_usage_options(cmd: &dyn Command) -> String {
    let options = cmd.options();
    if options.is_empty() {
        return String::new();
    }

    let mut parts = String::new();
    for opt in &options {
        if opt.takes_value {
            if opt.required {
                parts.push_str(&format!(" --{} <{}>", opt.name, opt.name.to_uppercase()));
            } else {
                parts.push_str(&format!(" [--{} <{}>]", opt.name, opt.name.to_uppercase()));
            }
        } else {
            parts.push_str(&format!(" [--{}]", opt.name));
        }
    }
    parts
}
