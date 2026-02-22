use crate::core::volkiwithstds::path::Path;

use crate::{veprintln, vvec};

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::core::volkiwithstds::collections::{String, Vec};
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

        let outdated = checker::check(root, dev)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("{e}")))?;

        if outdated.packages.is_empty() {
            output::print_item(
                &style::green(style::CHECK),
                &style::green("all packages are up to date"),
            );
            veprintln!();
            return Ok(());
        }

        super::outdated::print_outdated_table(&outdated);
        veprintln!();

        let packages_to_update: Vec<String> = match args.get_option("packages") {
            Some(list) => {
                let requested: Vec<String> =
                    list.split(",").map(|s| String::from(s.trim())).collect();
                for pkg in &requested {
                    if !outdated.packages.iter().any(|p| p.name == *pkg) {
                        return Err(CliError::InvalidUsage(crate::vformat!(
                            "Package '{pkg}' is not in the outdated list"
                        )));
                    }
                }
                requested
            }
            None => outdated.packages.iter().map(|p| p.name.clone()).collect(),
        };

        let manager = checker::detect_package_manager(root)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("{e}")))?;

        output::print_section(&crate::vformat!(
            "updating {} package(s)...",
            style::bold(&crate::vformat!("{}", packages_to_update.len())),
        ));
        veprintln!();

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
                    &crate::vformat!("updated {}", style::bold(&result.package)),
                );
            } else {
                fail_count += 1;
                output::print_step(
                    i + 1,
                    total,
                    &style::red(style::CROSS),
                    &crate::vformat!(
                        "failed {}: {}",
                        style::bold(&result.package),
                        result.message
                    ),
                );
            }
        }

        veprintln!();
        output::print_summary_box(&[&crate::vformat!(
            "{} updated, {} failed",
            style::green(&crate::vformat!("{}", success_count)),
            if fail_count > 0 {
                style::red(&crate::vformat!("{}", fail_count))
            } else {
                crate::vformat!("0")
            },
        )]);

        veprintln!();
        output::print_hint("run volki outdated to verify");
        veprintln!();

        if fail_count > 0 {
            return Err(CliError::InvalidUsage(crate::vformat!(
                "{fail_count} package(s) failed to update"
            )));
        }

        Ok(())
    }
}
