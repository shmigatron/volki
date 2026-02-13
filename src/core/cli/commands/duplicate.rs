use std::path::Path;

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::libs::lang::js::duplicate::detector;

pub struct DuplicateCommand;

impl Command for DuplicateCommand {
    fn name(&self) -> &str {
        "duplicate"
    }

    fn description(&self) -> &str {
        "Find duplicate code blocks in JS/TS projects"
    }

    fn long_description(&self) -> &str {
        "Detect duplicate (cloned) code blocks using token-based analysis.\n\n\
         Uses normalized token fingerprinting to find Type-2 clones\n\
         (structurally identical code with different variable names).\n\n\
         Use --min-tokens to control the minimum clone size (default: 50 tokens)."
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
                name: "min-tokens",
                description: "Minimum number of tokens for a clone (default: 50)",
                takes_value: true,
                required: false,
                default_value: Some("50"),
                short: Some('m'),
            },
        ]
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let path = args.get_option("path").unwrap_or(".");
        let root = Path::new(path);

        let min_tokens: usize = args
            .get_option("min-tokens")
            .unwrap_or("50")
            .parse()
            .map_err(|_| CliError::InvalidUsage("--min-tokens must be a number".to_string()))?;

        if min_tokens < 5 {
            return Err(CliError::InvalidUsage(
                "--min-tokens must be at least 5".to_string(),
            ));
        }

        let result = detector::detect(root, min_tokens)
            .map_err(|e| CliError::InvalidUsage(e.to_string()))?;

        if result.clones.is_empty() {
            output::print_item(
                &style::green(style::CHECK),
                &style::green("no duplicate code found"),
            );
            eprintln!();
            output::print_hint("adjust --min-tokens to change sensitivity");
            eprintln!();
            return Ok(());
        }

        for (i, group) in result.clones.iter().enumerate() {
            output::print_section(&format!(
                "clone #{} {}",
                i + 1,
                style::dim(&format!(
                    "({} tokens, {} instances)",
                    group.token_count,
                    group.instances.len()
                )),
            ));
            for (j, instance) in group.instances.iter().enumerate() {
                let is_last = j + 1 == group.instances.len();
                let connector = if is_last { style::TREE_LAST } else { style::TREE_BRANCH };
                output::print_item(
                    &style::dim(connector),
                    &format!(
                        "{} {}",
                        instance.file.display(),
                        style::dim(&format!("(lines {}-{})", instance.start_line, instance.end_line)),
                    ),
                );
            }
            eprintln!();
        }

        output::print_summary_box(&[
            &format!(
                "{} clone group(s)",
                style::yellow(&result.clones.len().to_string()),
            ),
            &format!(
                "~{} duplicated lines total",
                result.total_duplicated_lines,
            ),
        ]);

        eprintln!();
        output::print_hint("use --min-tokens to adjust sensitivity");
        eprintln!();

        Ok(())
    }
}
