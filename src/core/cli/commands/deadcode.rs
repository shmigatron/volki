use std::path::Path;

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::libs::lang::js::deadcode::detector;

pub struct DeadCodeCommand;

impl Command for DeadCodeCommand {
    fn name(&self) -> &str {
        "deadcode"
    }

    fn description(&self) -> &str {
        "Find unused files, exports, and imports in JS/TS projects"
    }

    fn long_description(&self) -> &str {
        "Analyze a JS/TS project to find dead code:\n\n\
         - Unused files: files never imported from any reachable file\n\
         - Unused exports: exported symbols never imported anywhere\n\
         - Unused imports: imported symbols never referenced in the importing file\n\n\
         Entry points are auto-detected from package.json (main, module, exports)\n\
         or specified manually with --entry."
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
                name: "entry",
                description: "Comma-separated entry point files (auto-detected if omitted)",
                takes_value: true,
                required: false,
                default_value: None,
                short: Some('e'),
            },
        ]
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let path = args.get_option("path").unwrap_or(".");
        let root = Path::new(path);
        let abs_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let entry_points: Vec<String> = match args.get_option("entry") {
            Some(entries) => entries.split(',').map(|s| s.trim().to_string()).collect(),
            None => vec![],
        };

        let result = detector::detect(root, &entry_points)
            .map_err(|e| CliError::InvalidUsage(e.to_string()))?;

        let rel = |p: &Path| -> String {
            p.strip_prefix(&abs_root)
                .or_else(|_| p.strip_prefix(root))
                .unwrap_or(p)
                .display()
                .to_string()
        };

        if !result.unused_files.is_empty() {
            output::print_section(&format!(
                "unused files {}",
                style::dim(&format!("({})", result.unused_files.len()))
            ));
            eprintln!();
            for (i, file) in result.unused_files.iter().enumerate() {
                let is_last = i + 1 == result.unused_files.len();
                let connector = if is_last { style::TREE_LAST } else { style::TREE_BRANCH };
                output::print_item(
                    &style::dim(connector),
                    &style::red(&rel(file)),
                );
            }
            eprintln!();
        }

        if !result.unused_exports.is_empty() {
            output::print_section(&format!(
                "unused exports {}",
                style::dim(&format!("({})", result.unused_exports.len()))
            ));
            eprintln!();
            for (i, exp) in result.unused_exports.iter().enumerate() {
                let is_last = i + 1 == result.unused_exports.len();
                let connector = if is_last { style::TREE_LAST } else { style::TREE_BRANCH };
                output::print_item(
                    &style::dim(connector),
                    &format!(
                        "{} {} {}",
                        style::yellow(&exp.name),
                        style::dim(&format!("{}:{}", rel(&exp.file), exp.line)),
                        style::dim(style::ARROW),
                    ),
                );
            }
            eprintln!();
        }

        if !result.unused_imports.is_empty() {
            output::print_section(&format!(
                "unused imports {}",
                style::dim(&format!("({})", result.unused_imports.len()))
            ));
            eprintln!();
            for (i, imp) in result.unused_imports.iter().enumerate() {
                let is_last = i + 1 == result.unused_imports.len();
                let connector = if is_last { style::TREE_LAST } else { style::TREE_BRANCH };
                output::print_item(
                    &style::dim(connector),
                    &format!(
                        "{} {} from {}",
                        style::yellow(&imp.name),
                        style::dim(&format!("{}:{}", rel(&imp.file), imp.line)),
                        style::dim(&format!("\"{}\"", imp.source)),
                    ),
                );
            }
            eprintln!();
        }

        let total = result.unused_files.len()
            + result.unused_exports.len()
            + result.unused_imports.len();
        if total == 0 {
            output::print_item(
                &style::green(style::CHECK),
                &style::green("no dead code found"),
            );
        } else {
            output::print_summary_box(&[
                &format!(
                    "{} {} found",
                    style::bold(&total.to_string()),
                    if total == 1 { "issue" } else { "issues" },
                ),
                &format!(
                    "{} unused file(s), {} unused export(s), {} unused import(s)",
                    result.unused_files.len(),
                    result.unused_exports.len(),
                    result.unused_imports.len(),
                ),
            ]);
        }

        eprintln!();
        output::print_hint("use --entry to specify custom entry points");
        eprintln!();

        Ok(())
    }
}
