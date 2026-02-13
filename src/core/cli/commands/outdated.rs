use std::path::Path;

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::libs::lang::js::outdated::checker;
use crate::libs::lang::js::outdated::checker::UpdateSeverity;

pub struct OutdatedCommand;

impl Command for OutdatedCommand {
    fn name(&self) -> &str {
        "outdated"
    }

    fn description(&self) -> &str {
        "Check for outdated npm dependencies"
    }

    fn long_description(&self) -> &str {
        "Check for outdated packages by querying the detected package manager\n\
         (npm, yarn, pnpm, or bun).\n\n\
         Shows current, wanted (semver-compatible), and latest versions\n\
         along with update severity (patch, minor, major)."
    }

    fn options(&self) -> Vec<OptionSpec> {
        vec![
            OptionSpec {
                name: "path",
                description: "Project root directory",
                takes_value: true,
                required: false,
                default_value: Some("."),
                short: Some('p'),
            },
            OptionSpec {
                name: "dev",
                description: "Include devDependencies",
                takes_value: false,
                required: false,
                default_value: None,
                short: Some('d'),
            },
        ]
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let path = args.get_option("path").unwrap_or(".");
        let dev = args.get_flag("dev");
        let root = Path::new(path);

        let result =
            checker::check(root, dev).map_err(|e| CliError::InvalidUsage(e.to_string()))?;

        if result.packages.is_empty() {
            output::print_item(
                &style::green(style::CHECK),
                &style::green("all packages are up to date"),
            );
            eprintln!();
            return Ok(());
        }

        print_outdated_table(&result);

        eprintln!();
        output::print_hint("run volki fix to update packages");
        eprintln!();

        Ok(())
    }
}

pub fn print_outdated_table(result: &checker::OutdatedResult) {
    let headers = ["Package", "Current", "Wanted", "Latest", "Severity"];
    let aligns = ['l', 'l', 'l', 'l', 'l'];

    let rows: Vec<Vec<String>> = result
        .packages
        .iter()
        .map(|pkg| {
            let sev_str = pkg.severity.to_string();
            let severity_styled = match pkg.severity {
                UpdateSeverity::Major => style::red(&format!("{} {}", style::WARN, sev_str)),
                UpdateSeverity::Minor => style::yellow(&sev_str),
                UpdateSeverity::Patch => style::dim(&sev_str),
            };
            vec![
                pkg.name.clone(),
                pkg.current.clone(),
                pkg.wanted.clone(),
                pkg.latest.clone(),
                severity_styled,
            ]
        })
        .collect();

    output::print_table(&headers, &rows, &aligns);

    eprintln!();
    output::print_summary_box(&[
        &format!(
            "{} outdated package(s)",
            style::yellow(&result.total.to_string()),
        ),
    ]);
}
