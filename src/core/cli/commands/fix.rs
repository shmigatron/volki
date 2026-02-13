use std::path::Path;

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::libs::lang::js::outdated::{checker, updater};

pub struct FixCommand;

impl Command for FixCommand {
    fn name(&self) -> &str {
        "fix"
    }

    fn description(&self) -> &str {
        "Update outdated npm dependencies"
    }

    fn long_description(&self) -> &str {
        "Check for outdated packages and update them.\n\n\
         By default, updates to the semver-compatible version. Use --latest to\n\
         install the absolute latest version (may include breaking changes).\n\n\
         Use --packages to update specific packages only (comma-separated)."
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
                name: "packages",
                description: "Comma-separated list of packages to update (defaults to all)",
                takes_value: true,
                required: false,
                default_value: None,
                short: None,
            },
            OptionSpec {
                name: "latest",
                description: "Install absolute latest versions (ignore semver ranges)",
                takes_value: false,
                required: false,
                default_value: None,
                short: Some('l'),
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
        let latest = args.get_flag("latest");
        let dev = args.get_flag("dev");
        let root = Path::new(path);

        let outdated =
            checker::check(root, dev).map_err(|e| CliError::InvalidUsage(e.to_string()))?;

        if outdated.packages.is_empty() {
            output::print_item(
                &style::green(style::CHECK),
                &style::green("all packages are up to date"),
            );
            eprintln!();
            return Ok(());
        }

        super::outdated::print_outdated_table(&outdated);
        eprintln!();

        let packages_to_update: Vec<String> = match args.get_option("packages") {
            Some(list) => {
                let requested: Vec<String> =
                    list.split(',').map(|s| s.trim().to_string()).collect();
                for pkg in &requested {
                    if !outdated.packages.iter().any(|p| p.name == *pkg) {
                        return Err(CliError::InvalidUsage(format!(
                            "Package '{pkg}' is not in the outdated list"
                        )));
                    }
                }
                requested
            }
            None => outdated.packages.iter().map(|p| p.name.clone()).collect(),
        };

        let manager = checker::detect_package_manager(root)
            .map_err(|e| CliError::InvalidUsage(e.to_string()))?;

        output::print_section(&format!(
            "updating {} package(s)...",
            style::bold(&packages_to_update.len().to_string()),
        ));
        eprintln!();

        let results = updater::update_packages(root, &manager, &packages_to_update, latest);

        let total = results.len();
        let mut success_count = 0;
        let mut fail_count = 0;

        for (i, result) in results.iter().enumerate() {
            if result.success {
                success_count += 1;
                output::print_step(
                    i + 1,
                    total,
                    &style::green(style::CHECK),
                    &format!("updated {}", style::bold(&result.package)),
                );
            } else {
                fail_count += 1;
                output::print_step(
                    i + 1,
                    total,
                    &style::red(style::CROSS),
                    &format!("failed {}: {}", style::bold(&result.package), result.message),
                );
            }
        }

        eprintln!();
        output::print_summary_box(&[
            &format!(
                "{} updated, {} failed",
                style::green(&success_count.to_string()),
                if fail_count > 0 { style::red(&fail_count.to_string()) } else { "0".to_string() },
            ),
        ]);

        eprintln!();
        output::print_hint("run volki outdated to verify");
        eprintln!();

        if fail_count > 0 {
            return Err(CliError::InvalidUsage(format!(
                "{fail_count} package(s) failed to update"
            )));
        }

        Ok(())
    }
}
