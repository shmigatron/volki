use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;

pub struct DbHubCommand;

impl Command for DbHubCommand {
    fn name(&self) -> &str {
        "db"
    }

    fn description(&self) -> &str {
        "Database management commands"
    }

    fn long_description(&self) -> &str {
        "Manage PostgreSQL databases, users, and tables. Reads connection details from the [db] section in volki.toml."
    }

    fn options(&self) -> Vec<OptionSpec> {
        Vec::new()
    }

    fn execute(&self, _args: &ParsedArgs) -> Result<(), CliError> {
        eprintln!("  {}", style::dim("available subcommands:"));
        eprintln!();
        eprintln!(
            "    {}    {}",
            style::cyan(&format!("{:<12}", "db:db")),
            style::dim("list databases"),
        );
        eprintln!(
            "    {}    {}",
            style::cyan(&format!("{:<12}", "db:user")),
            style::dim("manage database users/roles"),
        );
        eprintln!(
            "    {}    {}",
            style::cyan(&format!("{:<12}", "db:table")),
            style::dim("list database tables"),
        );
        eprintln!();
        output::print_hint("run volki <subcommand> --help for details");
        eprintln!();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_db() {
        assert_eq!(DbHubCommand.name(), "db");
    }

    #[test]
    fn requires_config() {
        assert!(DbHubCommand.requires_config());
    }

    #[test]
    fn description_nonempty() {
        assert!(!DbHubCommand.description().is_empty());
    }
}
