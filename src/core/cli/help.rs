use crate::core::volkiwithstds::collections::String;
use crate::veprintln;

use super::command::Command;
use super::style;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_top_level(commands: &[&dyn Command]) {
    veprintln!();
    veprintln!(
        "  {} {} {}",
        style::WOLF,
        style::bold("volki"),
        style::dim(&crate::vformat!("v{VERSION}"))
    );
    veprintln!("  {}", style::dim("code quality companion"));
    veprintln!();
    veprintln!("  {}  volki <command> [options]", style::bold("usage:"));
    veprintln!();
    veprintln!("  {}", style::bold("commands:"));

    let max_width = commands.iter().map(|c| c.name().len()).max().unwrap_or(0);
    for cmd in commands {
        veprintln!(
            "    {}    {}",
            style::cyan(&crate::vformat!("{:width$}", cmd.name(), width = max_width)),
            style::dim(cmd.description()),
        );
    }

    veprintln!();
    veprintln!("  {}", style::bold("global options:"));
    veprintln!(
        "    {}    {}",
        style::cyan(&crate::vformat!("{:<12}", "--help")),
        style::dim("print help information"),
    );
    veprintln!(
        "    {}    {}",
        style::cyan(&crate::vformat!("{:<12}", "--version")),
        style::dim("print version information"),
    );
    veprintln!(
        "    {}    {}",
        style::cyan(&crate::vformat!("{:<12}", "--no-color")),
        style::dim("disable colored output"),
    );

    veprintln!();
    veprintln!(
        "  {}  https://volki.dev {}",
        style::bold("docs:"),
        style::PAW,
    );
    veprintln!();
}

pub fn print_command_help(cmd: &dyn Command) {
    veprintln!();
    veprintln!("  {} volki {}", style::WOLF, style::bold(cmd.name()),);
    veprintln!();
    veprintln!("  {}", cmd.long_description());
    veprintln!();
    veprintln!(
        "  {}  volki {}{}",
        style::bold("usage:"),
        cmd.name(),
        format_usage_options(cmd),
    );

    let options = cmd.options();
    if !options.is_empty() {
        veprintln!();
        veprintln!("  {}", style::bold("options:"));

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
                crate::vformat!("--{} <{}>", opt.name, String::from(opt.name).to_uppercase())
            } else {
                crate::vformat!("--{}", opt.name)
            };

            let mut desc = String::from(opt.description);
            if let Some(def) = opt.default_value {
                desc.push_str(&crate::vformat!(
                    " {}",
                    style::dim(&crate::vformat!("[default: {def}]"))
                ));
            }
            if opt.required {
                desc.push_str(&crate::vformat!(" {}", style::yellow("(required)")));
            }

            veprintln!(
                "    {}    {}",
                style::cyan(&crate::vformat!("{:<width$}", flag_str, width = max_width)),
                desc,
            );
        }
    }

    veprintln!();
    veprintln!("  {}", style::bold("global options:"));
    veprintln!(
        "    {}    {}",
        style::cyan("--help"),
        style::dim("print help information"),
    );

    veprintln!();
    veprintln!(
        "  {}  https://volki.dev {}",
        style::bold("docs:"),
        style::PAW,
    );
    veprintln!();
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
                parts.push_str(&crate::vformat!(
                    " --{} <{}>",
                    opt.name,
                    String::from(opt.name).to_uppercase()
                ));
            } else {
                parts.push_str(&crate::vformat!(
                    " [--{} <{}>]",
                    opt.name,
                    String::from(opt.name).to_uppercase()
                ));
            }
        } else {
            parts.push_str(&crate::vformat!(" [--{}]", opt.name));
        }
    }
    parts
}
