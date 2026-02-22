//! web — hub command listing web framework subcommands.

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::core::volkiwithstds::collections::Vec;
use crate::veprintln;

pub struct WebHubCommand;

impl Command for WebHubCommand {
    fn name(&self) -> &str {
        "web"
    }

    fn description(&self) -> &str {
        "Web framework commands"
    }

    fn long_description(&self) -> &str {
        "Build and serve web applications using the volki web framework."
    }

    fn options(&self) -> Vec<OptionSpec> {
        Vec::new()
    }

    fn requires_config(&self) -> bool {
        false
    }

    fn execute(&self, _args: &ParsedArgs) -> Result<(), CliError> {
        veprintln!();
        veprintln!("  {}", style::dim("available subcommands:"));
        veprintln!();
        veprintln!(
            "    {}    {}",
            style::cyan(&crate::vformat!("{:<12}", "web:build")),
            style::dim("compile .volki files to Rust"),
        );
        veprintln!(
            "    {}    {}",
            style::cyan(&crate::vformat!("{:<12}", "web:start")),
            style::dim("start the web server"),
        );
        veprintln!(
            "    {}    {}",
            style::cyan(&crate::vformat!("{:<12}", "web:dev")),
            style::dim("start development server with hot reload"),
        );
        veprintln!();
        output::print_hint("run volki <subcommand> --help for details");
        veprintln!();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_hub_name() {
        assert_eq!(WebHubCommand.name(), "web");
    }

    #[test]
    fn test_web_hub_no_config_required() {
        assert!(!WebHubCommand.requires_config());
    }

    #[test]
    fn test_web_hub_description_nonempty() {
        assert!(!WebHubCommand.description().is_empty());
    }
}
