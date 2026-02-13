use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;

pub struct RunCommand;

impl Command for RunCommand {
    fn name(&self) -> &str {
        "run"
    }

    fn description(&self) -> &str {
        "Run the volki project"
    }

    fn long_description(&self) -> &str {
        "Run the volki project using the specified configuration file."
    }

    fn options(&self) -> Vec<OptionSpec> {
        vec![
            OptionSpec {
                name: "config",
                description: "Path to configuration file",
                takes_value: true,
                required: false,
                default_value: Some("volki.toml"),
                short: Some('c'),
            },
            OptionSpec {
                name: "verbose",
                description: "Enable verbose output",
                takes_value: false,
                required: false,
                default_value: None,
                short: Some('v'),
            },
        ]
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let config = args.get_option("config").unwrap_or("volki.toml");
        let verbose = args.get_flag("verbose");

        if verbose {
            output::print_item(
                &style::dim(style::BULLET),
                &format!("verbose mode enabled"),
            );
            output::print_item(
                &style::dim(style::BULLET),
                &format!("using config: {}", style::bold(config)),
            );
        }

        output::print_item(
            &style::green(style::CHECK),
            &format!("running with config '{}'", style::bold(config)),
        );
        eprintln!();
        Ok(())
    }
}
