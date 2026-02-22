use crate::core::volkiwithstds::path::Path;

use crate::{veprintln, vvec};

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::core::volkiwithstds::collections::Vec;
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
            .map_err(|_| {
                CliError::InvalidUsage(crate::vformat!("--min-tokens must be a number"))
            })?;

        if min_tokens < 5 {
            return Err(CliError::InvalidUsage(crate::vformat!(
                "--min-tokens must be at least 5"
            )));
        }

        let result = detector::detect(root, min_tokens)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("{e}")))?;

        if result.clones.is_empty() {
            output::print_item(
                &style::green(style::CHECK),
                &style::green("no duplicate code found"),
            );
            veprintln!();
            output::print_hint("adjust --min-tokens to change sensitivity");
            veprintln!();
            return Ok(());
        }

        for (i, group) in result.clones.iter().enumerate() {
            output::print_section(&crate::vformat!(
                "clone #{} {}",
                i + 1,
                style::dim(&crate::vformat!(
                    "({} tokens, {} instances)",
                    group.token_count,
                    group.instances.len()
                )),
            ));
            for (j, instance) in group.instances.iter().enumerate() {
                let is_last = j + 1 == group.instances.len();
                let connector = if is_last {
                    style::TREE_LAST
                } else {
                    style::TREE_BRANCH
                };
                output::print_item(
                    &style::dim(connector),
                    &crate::vformat!(
                        "{} {}",
                        instance.file.display(),
                        style::dim(&crate::vformat!(
                            "(lines {}-{})",
                            instance.start_line,
                            instance.end_line
                        )),
                    ),
                );
            }
            veprintln!();
        }

        output::print_summary_box(&[
            &crate::vformat!(
                "{} clone group(s)",
                style::yellow(&crate::vformat!("{}", result.clones.len())),
            ),
            &crate::vformat!("~{} duplicated lines total", result.total_duplicated_lines,),
        ]);

        veprintln!();
        output::print_hint("use --min-tokens to adjust sensitivity");
        veprintln!();

        Ok(())
    }
}
