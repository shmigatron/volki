use super::{connect_db, db_option, load_db_config, query_and_print, require_name};
use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::confirm::{self, ConfirmResult};
use crate::core::cli::error::CliError;
use crate::core::cli::parser::ParsedArgs;
use crate::core::volkiwithstds::collections::Vec;
use crate::{veprintln, vvec};

pub struct TableCommand;

impl Command for TableCommand {
    fn name(&self) -> &str {
        "db:table"
    }

    fn description(&self) -> &str {
        "Manage database tables"
    }

    fn long_description(&self) -> &str {
        "List, describe, drop, and truncate tables in the public schema. Subcommands: ls (default), describe, drop, truncate."
    }

    fn options(&self) -> Vec<OptionSpec> {
        vvec![
            db_option(),
            OptionSpec {
                name: "name",
                description: "Table name (for describe/drop/truncate)",
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
                "SELECT table_name, table_type \
                 FROM information_schema.tables \
                 WHERE table_schema = 'public' \
                 ORDER BY table_name",
                &["Name", "Type"],
                &['l', 'l'],
                db_name,
            ),
            "describe" => self.describe(args, db_name),
            "drop" => self.drop_table(args, db_name),
            "truncate" => self.truncate_table(args, db_name),
            other => Err(CliError::InvalidUsage(crate::vformat!(
                "unknown subcommand '{other}' for db:table (available: ls, describe, drop, truncate)"
            ))),
        }
    }
}

impl TableCommand {
    fn describe(&self, args: &ParsedArgs, db_name: Option<&str>) -> Result<(), CliError> {
        let name = require_name(args, "Table name")?;

        let sql = crate::vformat!(
            "SELECT column_name, data_type, is_nullable, column_default \
             FROM information_schema.columns \
             WHERE table_schema = 'public' AND table_name = '{}' \
             ORDER BY ordinal_position",
            name,
        );

        query_and_print(
            &sql,
            &["Column", "Type", "Nullable", "Default"],
            &['l', 'l', 'l', 'l'],
            db_name,
        )
    }

    fn drop_table(&self, args: &ParsedArgs, db_name: Option<&str>) -> Result<(), CliError> {
        let name = require_name(args, "Table name")?;

        let force = args.get_flag("force");
        let action = crate::vformat!("DROP TABLE {name}");

        if confirm::confirm_destructive(&action, &name, force)? == ConfirmResult::Cancelled {
            return Ok(());
        }

        let config = load_db_config(db_name)?;
        let mut conn = connect_db(&config)?;

        let sql = crate::vformat!("DROP TABLE {name}");
        conn.execute(&sql)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("failed to drop table: {e}")))?;

        veprintln!("  table '{}' dropped", name);
        veprintln!();
        Ok(())
    }

    fn truncate_table(&self, args: &ParsedArgs, db_name: Option<&str>) -> Result<(), CliError> {
        let name = require_name(args, "Table name")?;

        let force = args.get_flag("force");
        let action = crate::vformat!("TRUNCATE TABLE {name}");

        if confirm::confirm_destructive(&action, &name, force)? == ConfirmResult::Cancelled {
            return Ok(());
        }

        let config = load_db_config(db_name)?;
        let mut conn = connect_db(&config)?;

        let sql = crate::vformat!("TRUNCATE TABLE {name}");
        conn.execute(&sql).map_err(|e| {
            CliError::InvalidUsage(crate::vformat!("failed to truncate table: {e}"))
        })?;

        veprintln!("  table '{}' truncated", name);
        veprintln!();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::collections::String;

    #[test]
    fn name_is_db_table() {
        assert_eq!(TableCommand.name(), "db:table");
    }

    #[test]
    fn requires_config() {
        assert!(TableCommand.requires_config());
    }

    #[test]
    fn has_name_and_force_options() {
        let opts = TableCommand.options();
        assert!(opts.iter().any(|o| o.name == "name"));
        assert!(opts.iter().any(|o| o.name == "force"));
    }

    #[test]
    fn unknown_subcommand() {
        let raw = crate::core::cli::parser::RawArgs {
            subcommand: Some(String::from("db:table")),
            tokens: vvec![String::from("badcmd")],
        };
        let parsed = ParsedArgs::resolve(&raw, &TableCommand.options()).unwrap();
        let result = TableCommand.execute(&parsed);
        assert!(result.is_err());
        let msg = crate::vformat!("{}", result.unwrap_err());
        assert!(msg.contains("unknown subcommand"));
        assert!(msg.contains("describe"));
        assert!(msg.contains("drop"));
        assert!(msg.contains("truncate"));
    }
}
