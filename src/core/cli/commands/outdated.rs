use crate::core::volkiwithstds::path::Path;

use crate::{veprintln, vvec};

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::core::volkiwithstds::collections::Vec;
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
        vvec![
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

        let result = checker::check(root, dev)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("{e}")))?;

        if result.packages.is_empty() {
            output::print_item(
                &style::green(style::CHECK),
                &style::green("all packages are up to date"),
            );
            veprintln!();
            return Ok(());
        }

        print_outdated_table(&result);

        veprintln!();
        output::print_hint("run volki fix to update packages");
        veprintln!();

        Ok(())
    }
}

pub fn print_outdated_table(result: &checker::OutdatedResult) {
    use crate::core::volkiwithstds::collections::{String, Vec};

    let headers = ["Package", "Current", "Wanted", "Latest", "Severity"];
    let aligns = ['l', 'l', 'l', 'l', 'l'];

    let mut rows: Vec<Vec<String>> = Vec::new();
    for pkg in &result.packages {
        let sev_str = crate::vformat!("{}", pkg.severity);
        let severity_styled = match pkg.severity {
            UpdateSeverity::Major => style::red(&crate::vformat!("{} {}", style::WARN, sev_str)),
            UpdateSeverity::Minor => style::yellow(&sev_str),
            UpdateSeverity::Patch => style::dim(&sev_str),
        };
        let mut row = Vec::new();
        row.push(String::from(pkg.name.as_str()));
        row.push(String::from(pkg.current.as_str()));
        row.push(String::from(pkg.wanted.as_str()));
        row.push(String::from(pkg.latest.as_str()));
        row.push(severity_styled);
        rows.push(row);
    }

    output::print_table(&headers, rows.as_slice(), &aligns);

    veprintln!();
    output::print_summary_box(&[&crate::vformat!(
        "{} outdated package(s)",
        style::yellow(&crate::vformat!("{}", result.total)),
    )]);
}
