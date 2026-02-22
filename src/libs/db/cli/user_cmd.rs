use super::{
    connect_db, db_option, load_db_config, query_and_print, require_name, require_password,
};
use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::confirm::{self, ConfirmResult};
use crate::core::cli::error::CliError;
use crate::core::cli::parser::ParsedArgs;
use crate::core::volkiwithstds::collections::Vec;
use crate::{veprintln, vvec};

pub struct UserCommand;

impl Command for UserCommand {
    fn name(&self) -> &str {
        "db:user"
    }

    fn description(&self) -> &str {
        "Manage database users/roles"
    }

    fn long_description(&self) -> &str {
        "List and manage PostgreSQL roles. Subcommands: ls (default), add, drop."
    }

    fn options(&self) -> Vec<OptionSpec> {
        vvec![
            db_option(),
            OptionSpec {
                name: "name",
                description: "Role name (for add/drop)",
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
                "SELECT rolname, rolsuper, rolcreatedb, rolcanlogin \
                 FROM pg_roles \
                 ORDER BY rolname",
                &["Role", "Super", "CreateDB", "Login"],
                &['l', 'l', 'l', 'l'],
                db_name,
            ),
            "add" => self.add(args, db_name),
            "drop" => self.drop_role(args, db_name),
            other => Err(CliError::InvalidUsage(crate::vformat!(
                "unknown subcommand '{other}' for db:user (available: ls, add, drop)"
            ))),
        }
    }
}

impl UserCommand {
    fn add(&self, args: &ParsedArgs, db_name: Option<&str>) -> Result<(), CliError> {
        let name = require_name(args, "Role name")?;
        let password = require_password(args)?;

        let config = load_db_config(db_name)?;
        let mut conn = connect_db(&config)?;

        // DDL statements (CREATE ROLE) cannot use parameterized queries in Postgres.
        // name: validated above as alphanumeric+underscore only.
        // password: escaped via standard SQL single-quote doubling.
        let escaped_pw = password.replace("'", "''");
        let sql = crate::vformat!("CREATE ROLE {} WITH LOGIN PASSWORD '{}'", name, escaped_pw,);

        conn.execute(&sql)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("failed to create role: {e}")))?;

        veprintln!("  role '{}' created", name);
        veprintln!();
        Ok(())
    }

    fn drop_role(&self, args: &ParsedArgs, db_name: Option<&str>) -> Result<(), CliError> {
        let name = require_name(args, "Role name")?;

        let force = args.get_flag("force");
        let action = crate::vformat!("DROP ROLE {name}");

        if confirm::confirm_destructive(&action, &name, force)? == ConfirmResult::Cancelled {
            return Ok(());
        }

        let config = load_db_config(db_name)?;
        let mut conn = connect_db(&config)?;

        let sql = crate::vformat!("DROP ROLE {name}");
        conn.execute(&sql)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("failed to drop role: {e}")))?;

        veprintln!("  role '{}' dropped", name);
        veprintln!();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::collections::String;

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
    fn has_force_option() {
        let opts = UserCommand.options();
        assert!(opts.iter().any(|o| o.name == "force"));
    }

    #[test]
    fn unknown_subcommand() {
        let raw = crate::core::cli::parser::RawArgs {
            subcommand: Some(String::from("db:user")),
            tokens: vvec![String::from("badcmd")],
        };
        let parsed = ParsedArgs::resolve(&raw, &UserCommand.options()).unwrap();
        let result = UserCommand.execute(&parsed);
        assert!(result.is_err());
        let msg = crate::vformat!("{}", result.unwrap_err());
        assert!(msg.contains("unknown subcommand"));
        assert!(msg.contains("drop"));
    }
}
