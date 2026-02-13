use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;

use super::{connect_db, load_db_config, value_to_string};

pub struct UserCommand;

impl Command for UserCommand {
    fn name(&self) -> &str {
        "db:user"
    }

    fn description(&self) -> &str {
        "Manage database users/roles"
    }

    fn long_description(&self) -> &str {
        "List and manage PostgreSQL roles. Subcommands: ls (default), add."
    }

    fn options(&self) -> Vec<OptionSpec> {
        vec![
            OptionSpec {
                name: "name",
                description: "Role name (for add)",
                takes_value: true,
                required: false,
                default_value: None,
                short: None,
            },
            OptionSpec {
                name: "password",
                description: "Role password (for add)",
                takes_value: true,
                required: false,
                default_value: None,
                short: None,
            },
        ]
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let sub = args.positional().first().map(|s| s.as_str()).unwrap_or("ls");

        match sub {
            "ls" => self.list(),
            "add" => self.add(args),
            other => Err(CliError::InvalidUsage(format!(
                "unknown subcommand '{other}' for db:user (available: ls, add)"
            ))),
        }
    }
}

impl UserCommand {
    fn list(&self) -> Result<(), CliError> {
        let config = load_db_config()?;
        let mut conn = connect_db(&config)?;

        let sql = "\
            SELECT rolname, rolsuper, rolcreatedb, rolcanlogin \
            FROM pg_roles \
            ORDER BY rolname";

        let rows = conn
            .query(sql)
            .map_err(|e| CliError::InvalidUsage(format!("query failed: {e}")))?;

        let null_val = crate::libs::db::langs::postgres::lib::types::Value::Null;
        let mut table_rows = Vec::new();
        for row in &rows {
            table_rows.push(vec![
                value_to_string(row.get_value(0).unwrap_or(&null_val)),
                value_to_string(row.get_value(1).unwrap_or(&null_val)),
                value_to_string(row.get_value(2).unwrap_or(&null_val)),
                value_to_string(row.get_value(3).unwrap_or(&null_val)),
            ]);
        }

        output::print_table(
            &["Role", "Super", "CreateDB", "Login"],
            &table_rows,
            &['l', 'l', 'l', 'l'],
        );
        eprintln!();

        let _ = conn.close();
        Ok(())
    }

    fn add(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let name = args
            .get_option("name")
            .ok_or_else(|| CliError::InvalidUsage("--name is required for db:user add".to_string()))?;
        let password = args
            .get_option("password")
            .ok_or_else(|| {
                CliError::InvalidUsage("--password is required for db:user add".to_string())
            })?;

        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(CliError::InvalidUsage(
                "role name must contain only alphanumeric characters and underscores".to_string(),
            ));
        }

        let config = load_db_config()?;
        let mut conn = connect_db(&config)?;

        // Safe: name validated above; password escaped with replace
        let sql = format!(
            "CREATE ROLE {} WITH LOGIN PASSWORD '{}'",
            name,
            password.replace('\'', "''")
        );

        conn.execute(&sql)
            .map_err(|e| CliError::InvalidUsage(format!("failed to create role: {e}")))?;

        eprintln!("  role '{}' created", name);
        eprintln!();

        let _ = conn.close();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_db_user() {
        assert_eq!(UserCommand.name(), "db:user");
    }

    #[test]
    fn requires_config() {
        assert!(UserCommand.requires_config());
    }

    #[test]
    fn has_name_and_password_options() {
        let opts = UserCommand.options();
        assert!(opts.iter().any(|o| o.name == "name"));
        assert!(opts.iter().any(|o| o.name == "password"));
    }

    #[test]
    fn unknown_subcommand() {
        let raw = crate::core::cli::parser::RawArgs {
            subcommand: Some("db:user".to_string()),
            tokens: vec!["drop".to_string()],
        };
        let parsed = ParsedArgs::resolve(&raw, &UserCommand.options()).unwrap();
        let result = UserCommand.execute(&parsed);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("unknown subcommand"));
    }
}
