pub mod db_cmd;
pub mod db_hub_cmd;
pub mod user_cmd;
pub mod table_cmd;

pub use db_cmd::DbCommand;
pub use db_hub_cmd::DbHubCommand;
pub use user_cmd::UserCommand;
pub use table_cmd::TableCommand;

use crate::core::cli::error::CliError;
use crate::core::config::parser::Table;
use crate::libs::db::langs::postgres::lib::connection::Connection;

#[derive(Debug)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DbConfig {
    pub fn from_config(table: &Table) -> Result<Self, CliError> {
        if let Some(url_val) = table.get("db", "url") {
            if let Some(url) = url_val.as_str() {
                return Self::parse_url(url);
            }
        }

        let has_section = table.get("db", "user").is_some()
            || table.get("db", "database").is_some()
            || table.get("db", "host").is_some();

        if !has_section {
            return Err(CliError::InvalidUsage(
                "missing [db] section in volki.toml".to_string(),
            ));
        }

        let host = table
            .get("db", "host")
            .and_then(|v| v.as_str())
            .unwrap_or("localhost")
            .to_string();

        let port = table
            .get("db", "port")
            .and_then(|v| v.as_int())
            .map(|p| p as u16)
            .unwrap_or(5432);

        let user = table
            .get("db", "user")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CliError::InvalidUsage("missing 'user' in [db] section".to_string()))?
            .to_string();

        let password = table
            .get("db", "password")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let database = table
            .get("db", "database")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CliError::InvalidUsage("missing 'database' in [db] section".to_string())
            })?
            .to_string();

        Ok(DbConfig {
            host,
            port,
            user,
            password,
            database,
        })
    }

    fn parse_url(url: &str) -> Result<Self, CliError> {
        // postgres://user:pass@host:port/database
        let rest = url
            .strip_prefix("postgres://")
            .or_else(|| url.strip_prefix("postgresql://"))
            .ok_or_else(|| {
                CliError::InvalidUsage("db url must start with postgres:// or postgresql://".to_string())
            })?;

        // Split on '@' to separate credentials from host/db
        let (creds, host_part) = rest.split_once('@').ok_or_else(|| {
            CliError::InvalidUsage("invalid db url: missing '@' separator".to_string())
        })?;

        // Parse credentials: user:password or just user
        let (user, password) = match creds.split_once(':') {
            Some((u, p)) => (u.to_string(), p.to_string()),
            None => (creds.to_string(), String::new()),
        };

        // Parse host_part: host:port/database or host/database
        let (host_port, db_name) = host_part.split_once('/').ok_or_else(|| {
            CliError::InvalidUsage("invalid db url: missing database name after '/'".to_string())
        })?;

        let (host, port) = match host_port.split_once(':') {
            Some((h, p)) => {
                let port = p.parse::<u16>().map_err(|_| {
                    CliError::InvalidUsage(format!("invalid port in db url: '{p}'"))
                })?;
                (h.to_string(), port)
            }
            None => (host_port.to_string(), 5432),
        };

        if user.is_empty() {
            return Err(CliError::InvalidUsage("missing user in db url".to_string()));
        }
        if db_name.is_empty() {
            return Err(CliError::InvalidUsage("missing database in db url".to_string()));
        }

        Ok(DbConfig {
            host,
            port,
            user,
            password,
            database: db_name.to_string(),
        })
    }
}

pub fn connect_db(config: &DbConfig) -> Result<Connection, CliError> {
    Connection::connect(
        &config.host,
        config.port,
        &config.user,
        &config.database,
        &config.password,
    )
    .map_err(|e| CliError::InvalidUsage(format!("database connection failed: {e}")))
}

fn load_db_config() -> Result<DbConfig, CliError> {
    let cwd = std::env::current_dir()
        .map_err(|e| CliError::InvalidUsage(format!("cannot determine working directory: {e}")))?;
    let config = crate::core::config::VolkiConfig::load(&cwd)
        .map_err(|e| CliError::InvalidUsage(format!("failed to load config: {e}")))?;
    DbConfig::from_config(config.table())
}

fn value_to_string(val: &crate::libs::db::langs::postgres::lib::types::Value) -> String {
    use crate::libs::db::langs::postgres::lib::types::Value;
    match val {
        Value::Null => "NULL".to_string(),
        Value::Text(s) => s.clone(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => if *b { "t" } else { "f" }.to_string(),
        Value::Bytes(_) => "<bytes>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::parser;

    fn parse_table(content: &str) -> Table {
        parser::parse(content).unwrap()
    }

    // --- URL format ---

    #[test]
    fn from_config_url_full() {
        let table = parse_table("[db]\nurl = \"postgres://admin:secret@dbhost:5433/myapp\"");
        let cfg = DbConfig::from_config(&table).unwrap();
        assert_eq!(cfg.host, "dbhost");
        assert_eq!(cfg.port, 5433);
        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.password, "secret");
        assert_eq!(cfg.database, "myapp");
    }

    #[test]
    fn from_config_url_no_port() {
        let table = parse_table("[db]\nurl = \"postgres://user:pass@localhost/testdb\"");
        let cfg = DbConfig::from_config(&table).unwrap();
        assert_eq!(cfg.host, "localhost");
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.user, "user");
        assert_eq!(cfg.password, "pass");
        assert_eq!(cfg.database, "testdb");
    }

    #[test]
    fn from_config_url_no_password() {
        let table = parse_table("[db]\nurl = \"postgres://user@localhost:5432/testdb\"");
        let cfg = DbConfig::from_config(&table).unwrap();
        assert_eq!(cfg.user, "user");
        assert_eq!(cfg.password, "");
        assert_eq!(cfg.database, "testdb");
    }

    #[test]
    fn from_config_url_postgresql_scheme() {
        let table = parse_table("[db]\nurl = \"postgresql://user:pass@host/db\"");
        let cfg = DbConfig::from_config(&table).unwrap();
        assert_eq!(cfg.host, "host");
        assert_eq!(cfg.user, "user");
    }

    #[test]
    fn from_config_url_invalid_scheme() {
        let table = parse_table("[db]\nurl = \"mysql://user:pass@host/db\"");
        let result = DbConfig::from_config(&table);
        assert!(result.is_err());
    }

    #[test]
    fn from_config_url_missing_at() {
        let table = parse_table("[db]\nurl = \"postgres://userhost/db\"");
        let result = DbConfig::from_config(&table);
        assert!(result.is_err());
    }

    #[test]
    fn from_config_url_missing_database() {
        let table = parse_table("[db]\nurl = \"postgres://user:pass@host:5432\"");
        let result = DbConfig::from_config(&table);
        assert!(result.is_err());
    }

    #[test]
    fn from_config_url_invalid_port() {
        let table = parse_table("[db]\nurl = \"postgres://user:pass@host:abc/db\"");
        let result = DbConfig::from_config(&table);
        assert!(result.is_err());
    }

    // --- Individual fields ---

    #[test]
    fn from_config_fields_full() {
        let content = "\
[db]
host = \"myhost\"
port = 5433
user = \"admin\"
password = \"secret\"
database = \"mydb\"";
        let table = parse_table(content);
        let cfg = DbConfig::from_config(&table).unwrap();
        assert_eq!(cfg.host, "myhost");
        assert_eq!(cfg.port, 5433);
        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.password, "secret");
        assert_eq!(cfg.database, "mydb");
    }

    #[test]
    fn from_config_fields_defaults() {
        let content = "\
[db]
user = \"postgres\"
database = \"testdb\"";
        let table = parse_table(content);
        let cfg = DbConfig::from_config(&table).unwrap();
        assert_eq!(cfg.host, "localhost");
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.password, "");
    }

    #[test]
    fn from_config_missing_user() {
        let content = "\
[db]
database = \"testdb\"
host = \"localhost\"";
        let table = parse_table(content);
        let result = DbConfig::from_config(&table);
        assert!(result.is_err());
    }

    #[test]
    fn from_config_missing_database() {
        let content = "\
[db]
user = \"postgres\"
host = \"localhost\"";
        let table = parse_table(content);
        let result = DbConfig::from_config(&table);
        assert!(result.is_err());
    }

    // --- Missing section ---

    #[test]
    fn from_config_no_db_section() {
        let table = parse_table("[volki]\necosystem = \"node\"");
        let result = DbConfig::from_config(&table);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("missing [db] section"));
    }

    #[test]
    fn from_config_empty() {
        let table = parse_table("");
        let result = DbConfig::from_config(&table);
        assert!(result.is_err());
    }
}
