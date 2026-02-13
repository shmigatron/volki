use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;

use super::{connect_db, load_db_config, value_to_string};

pub struct TableCommand;

impl Command for TableCommand {
    fn name(&self) -> &str {
        "db:table"
    }

    fn description(&self) -> &str {
        "List database tables"
    }

    fn long_description(&self) -> &str {
        "List tables in the public schema of the connected PostgreSQL database."
    }

    fn options(&self) -> Vec<OptionSpec> {
        Vec::new()
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let sub = args.positional().first().map(|s| s.as_str()).unwrap_or("ls");

        match sub {
            "ls" => self.list(),
            other => Err(CliError::InvalidUsage(format!(
                "unknown subcommand '{other}' for db:table (available: ls)"
            ))),
        }
    }
}

impl TableCommand {
    fn list(&self) -> Result<(), CliError> {
        let config = load_db_config()?;
        let mut conn = connect_db(&config)?;

        let sql = "\
            SELECT table_name, table_type \
            FROM information_schema.tables \
            WHERE table_schema = 'public' \
            ORDER BY table_name";

        let rows = conn
            .query(sql)
            .map_err(|e| CliError::InvalidUsage(format!("query failed: {e}")))?;

        let null_val = crate::libs::db::langs::postgres::lib::types::Value::Null;
        let mut table_rows = Vec::new();
        for row in &rows {
            table_rows.push(vec![
                value_to_string(row.get_value(0).unwrap_or(&null_val)),
                value_to_string(row.get_value(1).unwrap_or(&null_val)),
            ]);
        }

        output::print_table(&["Name", "Type"], &table_rows, &['l', 'l']);
        eprintln!();

        let _ = conn.close();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_db_table() {
        assert_eq!(TableCommand.name(), "db:table");
    }

    #[test]
    fn requires_config() {
        assert!(TableCommand.requires_config());
    }

    #[test]
    fn unknown_subcommand() {
        let raw = crate::core::cli::parser::RawArgs {
            subcommand: Some("db:table".to_string()),
            tokens: vec!["drop".to_string()],
        };
        let parsed = ParsedArgs::resolve(&raw, &TableCommand.options()).unwrap();
        let result = TableCommand.execute(&parsed);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("unknown subcommand"));
    }
}
