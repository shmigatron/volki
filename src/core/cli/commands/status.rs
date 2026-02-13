use crate::core::cli::command::Command;
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;

pub struct StatusCommand;

impl Command for StatusCommand {
    fn name(&self) -> &str {
        "status"
    }

    fn description(&self) -> &str {
        "Show project status"
    }

    fn long_description(&self) -> &str {
        "Display the current status of the volki project."
    }

    fn execute(&self, _args: &ParsedArgs) -> Result<(), CliError> {
        output::print_item(
            &style::green(style::CHECK),
            &format!("project status: {}", style::green("ok")),
        );
        eprintln!();
        output::print_hint("run volki --help to see available commands");
        eprintln!();
        Ok(())
    }
}
