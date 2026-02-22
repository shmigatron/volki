use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::core::volkiwithstds::collections::Vec;
use crate::veprintln;

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
        // Show configured databases if volki.toml is present
        let cwd = crate::core::volkiwithstds::env::current_dir().ok();
        if let Some(ref dir) = cwd {
            if let Ok(config) = crate::core::config::VolkiConfig::load(dir) {
                let names = super::discover_db_names(config.table());
                if names.is_empty() {
                    if config.table().get("db", "dialect").is_some() {
                        veprintln!("  {}  {}", style::dim("config:"), "(single)");
                    }
                } else {
                    veprintln!("  {}  {}", style::dim("config:"), names.join(", "),);
                }
                veprintln!();
            }
        }

        veprintln!("  {}", style::dim("available subcommands:"));
        veprintln!();
        veprintln!(
            "    {}    {}",
            style::cyan(&crate::vformat!("{:<12}", "db:db")),
            style::dim("list, create, drop databases"),
        );
        veprintln!(
            "    {}    {}",
            style::cyan(&crate::vformat!("{:<12}", "db:user")),
            style::dim("list, add, drop database roles"),
        );
        veprintln!(
            "    {}    {}",
            style::cyan(&crate::vformat!("{:<12}", "db:table")),
            style::dim("list, describe, drop, truncate tables"),
        );
        veprintln!(
            "    {}    {}",
            style::cyan(&crate::vformat!("{:<12}", "db:web")),
            style::dim("launch web-based table editor"),
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
