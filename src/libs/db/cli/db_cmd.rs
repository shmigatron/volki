use super::{connect_db, db_option, load_db_config, query_and_print, require_name};
use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::confirm::{self, ConfirmResult};
use crate::core::cli::error::CliError;
use crate::core::cli::parser::ParsedArgs;
use crate::core::volkiwithstds::collections::Vec;
use crate::{veprintln, vvec};

pub struct DbCommand;

impl Command for DbCommand {
    fn name(&self) -> &str {
        "db:db"
    }

    fn description(&self) -> &str {
        "Manage databases"
    }

    fn long_description(&self) -> &str {
        "List, create, and drop PostgreSQL databases. Subcommands: ls (default), create, drop."
    }

    fn options(&self) -> Vec<OptionSpec> {
        vvec![
            db_option(),
            OptionSpec {
                name: "name",
                description: "Database name (for create/drop)",
                takes_value: true,
                required: false,
                default_value: None,
                short: None,
            },
            OptionSpec {
                name: "force",
                description: "Skip confirmation for destructive actions",
                takes_value: false,
                required: false,
                default_value: None,
                short: None,
            },
        ]
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let sub = args
            .positional()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("ls");
        let db_name = args.get_option("db");

        match sub {
            "ls" => query_and_print(
                "SELECT datname, \
                        pg_catalog.pg_get_userbyid(datdba) as owner, \
                        pg_encoding_to_char(encoding) as encoding \
                 FROM pg_database \
                 ORDER BY datname",
                &["Name", "Owner", "Encoding"],
                &['l', 'l', 'l'],
                db_name,
            ),
            "create" => self.create_db(args, db_name),
            "drop" => self.drop_db(args, db_name),
            other => Err(CliError::InvalidUsage(crate::vformat!(
                "unknown subcommand '{other}' for db:db (available: ls, create, drop)"
            ))),
        }
    }
}

impl DbCommand {
    fn create_db(&self, args: &ParsedArgs, db_name: Option<&str>) -> Result<(), CliError> {
        let name = require_name(args, "Database name")?;

        let config = load_db_config(db_name)?;
        let mut conn = connect_db(&config)?;

        let sql = crate::vformat!("CREATE DATABASE {name}");
        conn.execute(&sql).map_err(|e| {
            CliError::InvalidUsage(crate::vformat!("failed to create database: {e}"))
        })?;

        veprintln!("  database '{}' created", name);
        veprintln!();
        Ok(())
    }

    fn drop_db(&self, args: &ParsedArgs, db_name: Option<&str>) -> Result<(), CliError> {
        let name = require_name(args, "Database name")?;

        let force = args.get_flag("force");
        let action = crate::vformat!("DROP DATABASE {name}");

        if confirm::confirm_destructive(&action, &name, force)? == ConfirmResult::Cancelled {
            return Ok(());
        }

        let config = load_db_config(db_name)?;
        let mut conn = connect_db(&config)?;

        let sql = crate::vformat!("DROP DATABASE {name}");
        conn.execute(&sql)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("failed to drop database: {e}")))?;

        veprintln!("  database '{}' dropped", name);
        veprintln!();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::collections::String;

    #[test]
    fn name_is_db_db() {
        assert_eq!(DbCommand.name(), "db:db");
    }

    #[test]
    fn requires_config() {
        assert!(DbCommand.requires_config());
    }

    #[test]
    fn has_name_and_force_options() {
        let opts = DbCommand.options();
        assert!(opts.iter().any(|o| o.name == "name"));
        assert!(opts.iter().any(|o| o.name == "force"));
    }

    #[test]
    fn unknown_subcommand() {
        let raw = crate::core::cli::parser::RawArgs {
            subcommand: Some(String::from("db:db")),
            tokens: vvec![String::from("badcmd")],
        };
        let parsed = ParsedArgs::resolve(&raw, &DbCommand.options()).unwrap();
        let result = DbCommand.execute(&parsed);
        assert!(result.is_err());
        let msg = crate::vformat!("{}", result.unwrap_err());
        assert!(msg.contains("unknown subcommand"));
        assert!(msg.contains("create"));
        assert!(msg.contains("drop"));
    }
}
