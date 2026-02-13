use std::path::Path;

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::core::config::VolkiConfig;
use crate::core::plugins::registry::PluginRegistry;
use crate::libs::lang::js::formatter;
use crate::libs::lang::js::formatter::config::FormatConfig;
use crate::libs::lang::js::formatter::FileStatus;

pub struct FormatCommand;

impl Command for FormatCommand {
    fn name(&self) -> &str {
        "format"
    }

    fn description(&self) -> &str {
        "Format source files (JS/TS)"
    }

    fn long_description(&self) -> &str {
        "Format JavaScript and TypeScript source files.\n\n\
         Supports .js, .jsx, .ts, .tsx, .mjs, .cjs files.\n\n\
         Use --check to verify formatting without writing changes."
    }

    fn options(&self) -> Vec<OptionSpec> {
        vec![
            OptionSpec {
                name: "check",
                description: "Check if files are formatted (exit non-zero if not)",
                takes_value: false,
                required: false,
                default_value: None,
                short: Some('c'),
            },
        ]
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let path_str = args
            .positional()
            .first()
            .map(|s| s.as_str())
            .unwrap_or(".");
        let check = args.get_flag("check");
        let path = Path::new(path_str);

        let config = FormatConfig::default();

        let registry = VolkiConfig::load(path)
            .ok()
            .map(|cfg| {
                let specs = cfg.plugin_specs();
                PluginRegistry::load(&specs, path)
            });
        let plugins = registry.as_ref().filter(|r| !r.is_empty());

        let results = if check {
            formatter::check(path, &config, plugins)
        } else {
            formatter::format(path, &config, plugins)
        };

        let mut changed = 0usize;
        let mut unchanged = 0usize;
        let mut errors = 0usize;

        for result in &results {
            match &result.status {
                FileStatus::Changed => {
                    changed += 1;
                    if check {
                        output::print_item(
                            &style::yellow(style::WARN),
                            &format!("{}", result.path.display()),
                        );
                    } else {
                        output::print_item(
                            &style::green(style::CHECK),
                            &format!("formatted {}", result.path.display()),
                        );
                    }
                }
                FileStatus::Unchanged => unchanged += 1,
                FileStatus::Error(e) => {
                    errors += 1;
                    output::print_item(
                        &style::red(style::CROSS),
                        &format!("{}: {}", result.path.display(), e),
                    );
                }
            }
        }

        let total = changed + unchanged + errors;
        eprintln!();

        if check {
            if changed > 0 {
                output::print_summary_box(&[
                    &format!(
                        "{} file(s) would be reformatted",
                        style::yellow(&changed.to_string()),
                    ),
                    &format!("{unchanged} already formatted"),
                ]);
                eprintln!();
                output::print_hint("run volki format to fix");
                eprintln!();
                return Err(CliError::InvalidUsage(format!(
                    "{} file(s) not formatted", changed
                )));
            }
            output::print_item(
                &style::green(style::CHECK),
                &format!("all {total} file(s) already formatted"),
            );
        } else {
            output::print_summary_box(&[
                &format!(
                    "{} formatted, {} unchanged, {} error(s)",
                    style::green(&changed.to_string()),
                    unchanged,
                    if errors > 0 { style::red(&errors.to_string()) } else { errors.to_string() },
                ),
                &format!("{total} total files"),
            ]);
        }

        eprintln!();
        output::print_hint("use --check to verify without writing");
        eprintln!();

        Ok(())
    }
}
