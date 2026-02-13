use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;

use super::{connect_db, load_db_config, value_to_string};

pub struct DbCommand;

impl Command for DbCommand {
    fn name(&self) -> &str {
        "db:db"
    }

    fn description(&self) -> &str {
        "List databases"
    }

    fn long_description(&self) -> &str {
        "List PostgreSQL databases with owner and encoding information."
    }

    fn options(&self) -> Vec<OptionSpec> {
        Vec::new()
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let sub = args.positional().first().map(|s| s.as_str()).unwrap_or("ls");

        match sub {
            "ls" => self.list(),
            other => Err(CliError::InvalidUsage(format!(
                "unknown subcommand '{other}' for db:db (available: ls)"
            ))),
        }
    }
}

impl DbCommand {
    fn list(&self) -> Result<(), CliError> {
        let config = load_db_config()?;
        let mut conn = connect_db(&config)?;

        let sql = "\
            SELECT datname, \
                   pg_catalog.pg_get_userbyid(datdba) as owner, \
                   pg_encoding_to_char(encoding) as encoding \
            FROM pg_database \
            ORDER BY datname";

        let rows = conn
            .query(sql)
            .map_err(|e| CliError::InvalidUsage(format!("query failed: {e}")))?;

        let mut table_rows = Vec::new();
        for row in &rows {
            table_rows.push(vec![
                value_to_string(row.get_value(0).unwrap_or(&crate::libs::db::langs::postgres::lib::types::Value::Null)),
                value_to_string(row.get_value(1).unwrap_or(&crate::libs::db::langs::postgres::lib::types::Value::Null)),
                value_to_string(row.get_value(2).unwrap_or(&crate::libs::db::langs::postgres::lib::types::Value::Null)),
            ]);
        }

        output::print_table(&["Name", "Owner", "Encoding"], &table_rows, &['l', 'l', 'l']);
        eprintln!();

        let _ = conn.close();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_db_db() {
        assert_eq!(DbCommand.name(), "db:db");
    }

    #[test]
    fn requires_config() {
        assert!(DbCommand.requires_config());
    }

    #[test]
    fn unknown_subcommand() {
        let raw = crate::core::cli::parser::RawArgs {
            subcommand: Some("db:db".to_string()),
            tokens: vec!["badcmd".to_string()],
        };
        let parsed = ParsedArgs::resolve(&raw, &DbCommand.options()).unwrap();
        let result = DbCommand.execute(&parsed);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("unknown subcommand"));
    }
}
