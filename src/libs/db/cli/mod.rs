pub mod db_cmd;
pub mod db_hub_cmd;
pub mod user_cmd;
pub mod table_cmd;
pub mod web_cmd;

pub use db_cmd::DbCommand;
pub use db_hub_cmd::DbHubCommand;
pub use user_cmd::UserCommand;
pub use table_cmd::TableCommand;
pub use web_cmd::WebEditorCommand;

use crate::core::cli::command::OptionSpec;

use crate::core::cli::error::CliError;
use crate::core::cli::form::TextField;
use crate::core::cli::parser::ParsedArgs;
use crate::core::volkiwithstds::collections::{HashMap, String, Vec};
use crate::core::volkiwithstds::fmt;
use crate::core::cli::terminal;
use crate::core::cli::validate;
use crate::core::config::parser::Table;
use crate::core::package::env;
use crate::libs::db::langs::postgres::lib::connection::Connection;
use crate::libs::db::langs::postgres::lib::types::Value;
use crate::{veprintln, vformat, vvec};

fn db_option() -> OptionSpec {
    OptionSpec {
        name: "db",
        description: "Database config name (for multi-db setups)",
        takes_value: true,
        required: false,
        default_value: None,
        short: None,
    }
}

macro_rules! define_dialects {
    ( $( $variant:ident => $toml:literal, $display:literal, $port:expr );+ $(;)? ) => {
        const ALL_DIALECTS: &[&str] = &[ $( $toml ),+ ];

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Dialect { $( $variant ),+ }

        impl Dialect {
            pub fn from_toml_str(s: &str) -> Option<Self> {
                match s {
                    $( $toml => Some(Dialect::$variant), )+
                    _ => None,
                }
            }

            pub fn as_toml_str(&self) -> &'static str {
                match self {
                    $( Dialect::$variant => $toml, )+
                }
            }

            pub fn default_port(&self) -> u16 {
                match self {
                    $( Dialect::$variant => $port, )+
                }
            }

            pub fn is_implemented(&self) -> bool {
                matches!(self, Dialect::Postgres)
            }
        }

        impl fmt::Display for Dialect {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $( Dialect::$variant => write!(f, $display), )+
                }
            }
        }
    };
}

define_dialects! {
    Cassandra  => "cassandra",  "Cassandra",      9042;
    Couch      => "couch",      "CouchDB",        5984;
    Dgraph     => "dgraph",     "Dgraph",         9080;
    Dynamo     => "dynamo",     "DynamoDB",        8000;
    Elastic    => "elastic",    "Elasticsearch",   9200;
    Influx     => "influx",     "InfluxDB",        8086;
    Kafka      => "kafka",      "Kafka",           9092;
    Mariadb    => "mariadb",    "MariaDB",         3306;
    Memcached  => "memcached",  "Memcached",      11211;
    Mongo      => "mongo",      "MongoDB",        27017;
    Mssql      => "mssql",      "SQL Server",      1433;
    Mysql      => "mysql",      "MySQL",           3306;
    Nats       => "nats",       "NATS",            4222;
    Neo4j      => "neo4j",      "Neo4j",           7687;
    Opensearch => "opensearch", "OpenSearch",      9200;
    Oracle     => "oracle",     "Oracle",          1521;
    Postgres   => "postgres",   "PostgreSQL",      5432;
    Rabbitmq   => "rabbitmq",   "RabbitMQ",        5672;
    Redis      => "redis",      "Redis",           6379;
    Scylla     => "scylla",     "ScyllaDB",        9042;
    Sqlite     => "sqlite",     "SQLite",             0;
    Timescale  => "timescale",  "TimescaleDB",     5432;
    Valkey     => "valkey",     "Valkey",          6379;
}

#[derive(Debug)]
pub struct DbConfig {
    pub dialect: Dialect,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DbConfig {
    pub fn from_config(table: &Table, section: &str) -> Result<Self, CliError> {
        // dialect is always required
        let dialect = Self::parse_dialect(table, section)?;

        // credentials mode: "env" or "field" (default)
        let creds_mode = table
            .get(section, "credentials")
            .and_then(|v| v.as_str())
            .unwrap_or("field");

        match creds_mode {
            "env" => Self::from_env(table, section, dialect),
            "field" => Self::from_fields(table, section, dialect),
            other => Err(CliError::InvalidUsage(vformat!(
                "invalid credentials mode '{}' in [{}] section of volki.toml\n\n  \
                 allowed values: \"env\" or \"field\" (default)",
                other, section,
            ))),
        }
    }

    /// Resolve credentials from environment variables / `.env` file,
    /// falling back to volki.toml fields for anything not found.
    fn from_env(table: &Table, section: &str, dialect: Dialect) -> Result<Self, CliError> {
        let cwd = crate::core::volkiwithstds::env::current_dir().unwrap_or_default();
        let dotenv = env::load_dotenv(&cwd);

        // Try URL from env first
        if let Some(url) = env::get_first_env(&["DATABASE_URL", "DB_URL"], &dotenv) {
            return Self::parse_url(&url, dialect);
        }

        // Try URL from toml as fallback
        if let Some(url_val) = table.get(section, "url") {
            if let Some(url) = url_val.as_str() {
                return Self::parse_url(url, dialect);
            }
        }

        // Resolve individual fields: env > toml > default
        let host = Self::env_or_field("DB_HOST", table, section, "host", &dotenv)
            .unwrap_or_else(|| String::from("localhost"));

        let port_str = Self::env_or_field("DB_PORT", table, section, "port", &dotenv);
        let port = match port_str {
            Some(s) => s.parse::<u16>().map_err(|_| {
                CliError::InvalidUsage(vformat!("invalid DB_PORT value: '{s}'"))
            })?,
            None => dialect.default_port(),
        };

        let user = Self::env_or_field_required(
            &["DB_USER", "DB_USERNAME"],
            table,
            section,
            "user",
            &dotenv,
            "DB_USER or DB_USERNAME",
        )?;

        let password = Self::env_or_field("DB_PASSWORD", table, section, "password", &dotenv)
            .unwrap_or_default();

        let database = Self::env_or_field_required(
            &["DB_NAME", "DB_DATABASE"],
            table,
            section,
            "database",
            &dotenv,
            "DB_NAME or DB_DATABASE",
        )?;

        Ok(DbConfig { dialect, host, port, user, password, database })
    }

    /// Resolve credentials from volki.toml fields only.
    fn from_fields(table: &Table, section: &str, dialect: Dialect) -> Result<Self, CliError> {
        if let Some(url_val) = table.get(section, "url") {
            if let Some(url) = url_val.as_str() {
                return Self::parse_url(url, dialect);
            }
        }

        let has_section = table.get(section, "user").is_some()
            || table.get(section, "database").is_some()
            || table.get(section, "host").is_some();

        if !has_section {
            return Err(CliError::InvalidUsage(vformat!(
                "missing connection details in [{}] section of volki.toml\n\n  \
                 add a url:\n\n    \
                 [{}]\n    \
                 dialect = \"postgres\"\n    \
                 url = \"postgres://user:pass@localhost:5432/mydb\"\n\n  \
                 or use individual fields:\n\n    \
                 [{}]\n    \
                 dialect = \"postgres\"\n    \
                 host = \"localhost\"\n    \
                 port = 5432\n    \
                 user = \"postgres\"\n    \
                 password = \"\"\n    \
                 database = \"mydb\"\n\n  \
                 or set credentials = \"env\" to read from environment variables",
                section, section, section,
            )));
        }

        let host = table
            .get(section, "host")
            .and_then(|v| v.as_str())
            .unwrap_or("localhost");
        let host = String::from(host);

        let port = table
            .get(section, "port")
            .and_then(|v| v.as_int())
            .map(|p| p as u16)
            .unwrap_or_else(|| dialect.default_port());

        let user = table
            .get(section, "user")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CliError::InvalidUsage(vformat!(
                    "missing 'user' in [{}] section of volki.toml\n\n  \
                     add: user = \"postgres\"",
                    section,
                ))
            })?;
        let user = String::from(user);

        let password = table
            .get(section, "password")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let password = String::from(password);

        let database = table
            .get(section, "database")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CliError::InvalidUsage(vformat!(
                    "missing 'database' in [{}] section of volki.toml\n\n  \
                     add: database = \"mydb\"",
                    section,
                ))
            })?;
        let database = String::from(database);

        Ok(DbConfig { dialect, host, port, user, password, database })
    }

    /// Try env var, then toml field, return None if neither set.
    fn env_or_field(
        env_key: &str,
        table: &Table,
        section: &str,
        toml_key: &str,
        dotenv: &HashMap<String, String>,
    ) -> Option<String> {
        env::get_env_or_dotenv(env_key, dotenv).or_else(|| {
            table.get(section, toml_key).and_then(|v| {
                v.as_str()
                    .map(|s| String::from(s))
                    .or_else(|| v.as_int().map(|n| vformat!("{}", n)))
            })
        })
    }

    /// Try multiple env var names, then toml field, error if none found.
    fn env_or_field_required(
        env_keys: &[&str],
        table: &Table,
        section: &str,
        toml_key: &str,
        dotenv: &HashMap<String, String>,
        env_label: &str,
    ) -> Result<String, CliError> {
        if let Some(val) = env::get_first_env(env_keys, dotenv) {
            return Ok(val);
        }
        if let Some(val) = table.get(section, toml_key).and_then(|v| v.as_str()) {
            return Ok(String::from(val));
        }
        Err(CliError::InvalidUsage(vformat!(
            "could not resolve '{toml_key}' for database connection\n\n  \
             set env var {env_label}, add it to .env, or add '{toml_key}' to [{section}] in volki.toml",
        )))
    }

    fn parse_dialect(table: &Table, section: &str) -> Result<Dialect, CliError> {
        let raw = table
            .get(section, "dialect")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CliError::InvalidUsage(vformat!(
                    "missing 'dialect' in [{}] section of volki.toml\n\n  \
                     add: dialect = \"postgres\"\n\n  \
                     supported dialects: {}",
                    section,
                    ALL_DIALECTS.join(", "),
                ))
            })?;

        Dialect::from_toml_str(raw).ok_or_else(|| {
            CliError::InvalidUsage(vformat!(
                "unknown dialect '{}' in [{}] section of volki.toml\n\n  \
                 supported dialects: {}",
                raw, section,
                ALL_DIALECTS.join(", "),
            ))
        })
    }

    fn parse_url(url: &str, dialect: Dialect) -> Result<Self, CliError> {
        // postgres://user:pass@host:port/database
        let rest = url
            .strip_prefix("postgres://")
            .or_else(|| url.strip_prefix("postgresql://"))
            .ok_or_else(|| {
                CliError::InvalidUsage(vformat!(
                    "invalid db url in volki.toml\n\n  \
                     url must start with postgres:// or postgresql://\n  \
                     got: {url}\n\n  \
                     example: url = \"postgres://user:pass@localhost:5432/mydb\""
                ))
            })?;

        // Split on '@' to separate credentials from host/db
        let (creds, host_part) = rest.split_once('@').ok_or_else(|| {
            CliError::InvalidUsage(String::from("invalid db url: missing '@' separator"))
        })?;

        // Parse credentials: user:password or just user
        let (user, password) = match creds.split_once(':') {
            Some((u, p)) => (String::from(u), String::from(p)),
            None => (String::from(creds), String::new()),
        };

        // Parse host_part: host:port/database or host/database
        let (host_port, db_name) = host_part.split_once('/').ok_or_else(|| {
            CliError::InvalidUsage(String::from("invalid db url: missing database name after '/'"))
        })?;

        let (host, port) = match host_port.split_once(':') {
            Some((h, p)) => {
                let port = p.parse::<u16>().map_err(|_| {
                    CliError::InvalidUsage(vformat!("invalid port in db url: '{p}'"))
                })?;
                (String::from(h), port)
            }
            None => (String::from(host_port), dialect.default_port()),
        };

        if user.is_empty() {
            return Err(CliError::InvalidUsage(String::from("missing user in db url")));
        }
        if db_name.is_empty() {
            return Err(CliError::InvalidUsage(String::from("missing database in db url")));
        }

        Ok(DbConfig {
            dialect,
            host,
            port,
            user,
            password,
            database: String::from(db_name),
        })
    }
}

pub fn connect_db(config: &DbConfig) -> Result<Connection, CliError> {
    if !config.dialect.is_implemented() {
        return Err(CliError::InvalidUsage(vformat!(
            "{} driver is not yet implemented\n\n  \
             currently supported: postgres\n\n  \
             update volki.toml:\n\n    \
             [db]\n    \
             dialect = \"postgres\"",
            config.dialect,
        )));
    }

    Connection::connect(
        &config.host,
        config.port,
        &config.user,
        &config.database,
        &config.password,
    )
    .map_err(|e| {
        CliError::InvalidUsage(vformat!(
            "failed to connect to {} at {}:{} (user={}, db={})\n\n  \
             error: {e}\n\n  \
             check that:\n  \
             - {} is running on {}:{}\n  \
             - the credentials in volki.toml [db] section are correct\n  \
             - the database '{}' exists",
            config.dialect,
            config.host, config.port, config.user, config.database,
            config.dialect, config.host, config.port, config.database,
        ))
    })
}

fn discover_db_names(table: &Table) -> Vec<String> {
    if table.get("db", "dialect").is_some() {
        return vvec![];
    }
    let mut names = Vec::new();
    for (key, _) in table.entries() {
        if let Some(rest) = key.strip_prefix("db.") {
            if let Some(name) = rest.strip_suffix(".dialect") {
                if !name.contains('.') {
                    names.push(String::from(name));
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn load_db_config(db_name: Option<&str>) -> Result<DbConfig, CliError> {
    let cwd = crate::core::volkiwithstds::env::current_dir()
        .map_err(|e| CliError::InvalidUsage(vformat!("cannot determine working directory: {e}")))?;
    let config = crate::core::config::VolkiConfig::load(&cwd).map_err(|e| {
        CliError::InvalidUsage(vformat!(
            "failed to load volki.toml from {}\n\n  error: {e}",
            cwd.display()
        ))
    })?;
    let table = config.table();
    let names = discover_db_names(table);

    if names.is_empty() {
        // Single-db mode
        DbConfig::from_config(table, "db")
    } else {
        // Multi-db mode
        match db_name {
            Some(name) => {
                if !names.contains(&String::from(name)) {
                    return Err(CliError::InvalidUsage(vformat!(
                        "database '{}' not found in volki.toml\n\n  \
                         available databases: {}",
                        name,
                        names.join(", "),
                    )));
                }
                DbConfig::from_config(table, &vformat!("db.{name}"))
            }
            None => Err(CliError::InvalidUsage(vformat!(
                "multiple databases configured in volki.toml, use --db <name>\n\n  \
                 available databases: {}",
                names.join(", "),
            ))),
        }
    }
}

fn value_to_string(val: &Value) -> String {
    match val {
        Value::Null => String::from("NULL"),
        Value::Text(s) => s.clone(),
        Value::Int(n) => vformat!("{}", n),
        Value::Float(f) => vformat!("{}", f),
        Value::Bool(b) => String::from(if *b { "t" } else { "f" }),
        Value::Bytes(_) => String::from("<bytes>"),
    }
}

/// Run a read-only SQL query and print results as a table.
/// Handles: load config → connect → query → format → print.
fn query_and_print(
    sql: &str,
    headers: &[&str],
    alignments: &[char],
    db_name: Option<&str>,
) -> Result<(), CliError> {
    let config = load_db_config(db_name)?;
    let mut conn = connect_db(&config)?;

    let rows = conn
        .query(sql)
        .map_err(|e| CliError::InvalidUsage(vformat!("query failed: {e}")))?;

    let col_count = headers.len();
    let mut table_rows = Vec::new();
    for row in &rows {
        let mut cells = Vec::with_capacity(col_count);
        for i in 0..col_count {
            cells.push(value_to_string(row.get_value(i).unwrap_or(&Value::Null)));
        }
        table_rows.push(cells);
    }

    crate::core::cli::output::print_table(headers, &table_rows, alignments);
    veprintln!();
    Ok(())
}

/// If `--name` was passed, validate and return it.
/// Otherwise prompt interactively (TTY) or error (non-TTY).
fn require_name(args: &ParsedArgs, label: &str) -> Result<String, CliError> {
    if let Some(val) = args.get_option("name") {
        validate::validate_identifier(val, label)?;
        return Ok(String::from(val));
    }
    if !terminal::is_stdin_tty() {
        return Err(CliError::MissingArgument(String::from("name")));
    }
    let label_owned = String::from(label);
    TextField::new(label)
        .validate(move |v| {
            validate::validate_identifier(v, &label_owned).map_err(|e| vformat!("{}", e))
        })
        .run()
}

/// If `--password` was passed, return it.
/// Otherwise prompt interactively (TTY) or error (non-TTY).
fn require_password(args: &ParsedArgs) -> Result<String, CliError> {
    if let Some(val) = args.get_option("password") {
        return Ok(String::from(val));
    }
    if !terminal::is_stdin_tty() {
        return Err(CliError::MissingArgument(String::from("password")));
    }
    TextField::new("Password").run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::parser;

    fn parse_table(content: &str) -> Table {
        parser::parse(content).unwrap()
    }

    // --- Dialect ---

    #[test]
    fn dialect_roundtrip_all() {
        for &name in ALL_DIALECTS {
            let d = Dialect::from_toml_str(name).unwrap();
            assert_eq!(d.as_toml_str(), name);
        }
    }

    #[test]
    fn dialect_from_toml_unknown() {
        assert!(Dialect::from_toml_str("cockroach").is_none());
    }

    #[test]
    fn dialect_display() {
        assert_eq!(vformat!("{}", Dialect::Postgres), "PostgreSQL");
        assert_eq!(vformat!("{}", Dialect::Mysql), "MySQL");
        assert_eq!(vformat!("{}", Dialect::Mongo), "MongoDB");
    }

    #[test]
    fn dialect_only_postgres_implemented() {
        assert!(Dialect::Postgres.is_implemented());
        assert!(!Dialect::Mysql.is_implemented());
        assert!(!Dialect::Redis.is_implemented());
    }

    #[test]
    fn from_config_missing_dialect() {
        let table = parse_table("[db]\nuser = \"postgres\"\ndatabase = \"mydb\"");
        let result = DbConfig::from_config(&table, "db");
        assert!(result.is_err());
        let msg = vformat!("{}", result.unwrap_err());
        assert!(msg.contains("missing 'dialect'"));
        assert!(msg.contains("supported dialects"));
    }

    #[test]
    fn from_config_unknown_dialect() {
        let table = parse_table("[db]\ndialect = \"cockroach\"\nuser = \"x\"\ndatabase = \"y\"");
        let result = DbConfig::from_config(&table, "db");
        assert!(result.is_err());
        let msg = vformat!("{}", result.unwrap_err());
        assert!(msg.contains("unknown dialect 'cockroach'"));
    }

    #[test]
    fn from_config_dialect_stored() {
        let table = parse_table("[db]\ndialect = \"mysql\"\nuser = \"root\"\ndatabase = \"test\"");
        let cfg = DbConfig::from_config(&table, "db").unwrap();
        assert_eq!(cfg.dialect, Dialect::Mysql);
    }

    #[test]
    fn from_config_invalid_credentials_mode() {
        let table = parse_table("[db]\ndialect = \"postgres\"\ncredentials = \"magic\"\nuser = \"x\"\ndatabase = \"y\"");
        let result = DbConfig::from_config(&table, "db");
        assert!(result.is_err());
        let msg = vformat!("{}", result.unwrap_err());
        assert!(msg.contains("invalid credentials mode"));
    }

    // --- field mode (default) ---

    #[test]
    fn field_mode_url_full() {
        let table = parse_table("[db]\ndialect = \"postgres\"\nurl = \"postgres://admin:secret@dbhost:5433/myapp\"");
        let cfg = DbConfig::from_config(&table, "db").unwrap();
        assert_eq!(cfg.dialect, Dialect::Postgres);
        assert_eq!(cfg.host, "dbhost");
        assert_eq!(cfg.port, 5433);
        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.password, "secret");
        assert_eq!(cfg.database, "myapp");
    }

    #[test]
    fn field_mode_url_no_port() {
        let table = parse_table("[db]\ndialect = \"postgres\"\nurl = \"postgres://user:pass@localhost/testdb\"");
        let cfg = DbConfig::from_config(&table, "db").unwrap();
        assert_eq!(cfg.host, "localhost");
        assert_eq!(cfg.port, 5432);
    }

    #[test]
    fn field_mode_url_no_password() {
        let table = parse_table("[db]\ndialect = \"postgres\"\nurl = \"postgres://user@localhost:5432/testdb\"");
        let cfg = DbConfig::from_config(&table, "db").unwrap();
        assert_eq!(cfg.user, "user");
        assert_eq!(cfg.password, "");
    }

    #[test]
    fn field_mode_url_postgresql_scheme() {
        let table = parse_table("[db]\ndialect = \"postgres\"\nurl = \"postgresql://user:pass@host/db\"");
        let cfg = DbConfig::from_config(&table, "db").unwrap();
        assert_eq!(cfg.host, "host");
    }

    #[test]
    fn field_mode_url_invalid_scheme() {
        let table = parse_table("[db]\ndialect = \"postgres\"\nurl = \"mysql://user:pass@host/db\"");
        assert!(DbConfig::from_config(&table, "db").is_err());
    }

    #[test]
    fn field_mode_url_missing_at() {
        let table = parse_table("[db]\ndialect = \"postgres\"\nurl = \"postgres://userhost/db\"");
        assert!(DbConfig::from_config(&table, "db").is_err());
    }

    #[test]
    fn field_mode_url_missing_database() {
        let table = parse_table("[db]\ndialect = \"postgres\"\nurl = \"postgres://user:pass@host:5432\"");
        assert!(DbConfig::from_config(&table, "db").is_err());
    }

    #[test]
    fn field_mode_url_invalid_port() {
        let table = parse_table("[db]\ndialect = \"postgres\"\nurl = \"postgres://user:pass@host:abc/db\"");
        assert!(DbConfig::from_config(&table, "db").is_err());
    }

    #[test]
    fn field_mode_full() {
        let content = "\
[db]
dialect = \"postgres\"
host = \"myhost\"
port = 5433
user = \"admin\"
password = \"secret\"
database = \"mydb\"";
        let table = parse_table(content);
        let cfg = DbConfig::from_config(&table, "db").unwrap();
        assert_eq!(cfg.dialect, Dialect::Postgres);
        assert_eq!(cfg.host, "myhost");
        assert_eq!(cfg.port, 5433);
        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.password, "secret");
        assert_eq!(cfg.database, "mydb");
    }

    #[test]
    fn field_mode_defaults() {
        let content = "\
[db]
dialect = \"postgres\"
user = \"postgres\"
database = \"testdb\"";
        let table = parse_table(content);
        let cfg = DbConfig::from_config(&table, "db").unwrap();
        assert_eq!(cfg.host, "localhost");
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.password, "");
    }

    #[test]
    fn field_mode_missing_user() {
        let content = "\
[db]
dialect = \"postgres\"
database = \"testdb\"
host = \"localhost\"";
        let table = parse_table(content);
        assert!(DbConfig::from_config(&table, "db").is_err());
    }

    #[test]
    fn field_mode_missing_database() {
        let content = "\
[db]
dialect = \"postgres\"
user = \"postgres\"
host = \"localhost\"";
        let table = parse_table(content);
        assert!(DbConfig::from_config(&table, "db").is_err());
    }

    #[test]
    fn field_mode_no_details_mentions_env() {
        let content = "[db]\ndialect = \"postgres\"";
        let table = parse_table(content);
        let err = DbConfig::from_config(&table, "db").unwrap_err();
        let msg = vformat!("{err}");
        assert!(msg.contains("credentials = \"env\""));
    }

    // --- env mode ---

    #[test]
    fn env_mode_falls_back_to_toml_fields() {
        // When credentials = "env" but no env vars are set,
        // it should fall back to toml fields
        let content = "\
[db]
dialect = \"postgres\"
credentials = \"env\"
user = \"admin\"
database = \"mydb\"";
        let table = parse_table(content);
        let cfg = DbConfig::from_config(&table, "db").unwrap();
        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.database, "mydb");
        assert_eq!(cfg.host, "localhost");
        assert_eq!(cfg.port, 5432);
    }

    #[test]
    fn env_mode_falls_back_to_toml_url() {
        let content = "\
[db]
dialect = \"postgres\"
credentials = \"env\"
url = \"postgres://u:p@h:1234/d\"";
        let table = parse_table(content);
        let cfg = DbConfig::from_config(&table, "db").unwrap();
        assert_eq!(cfg.host, "h");
        assert_eq!(cfg.port, 1234);
        assert_eq!(cfg.user, "u");
        assert_eq!(cfg.database, "d");
    }

    #[test]
    fn env_mode_missing_user_and_database_errors() {
        let content = "\
[db]
dialect = \"postgres\"
credentials = \"env\"";
        let table = parse_table(content);
        let result = DbConfig::from_config(&table, "db");
        assert!(result.is_err());
        let msg = vformat!("{}", result.unwrap_err());
        assert!(msg.contains("could not resolve"));
    }

    // --- Missing section ---

    #[test]
    fn from_config_no_db_section() {
        let table = parse_table("[volki]\necosystem = \"node\"");
        let result = DbConfig::from_config(&table, "db");
        assert!(result.is_err());
        let msg = vformat!("{}", result.unwrap_err());
        assert!(msg.contains("missing 'dialect'"));
    }

    #[test]
    fn from_config_empty() {
        let table = parse_table("");
        assert!(DbConfig::from_config(&table, "db").is_err());
    }

    // --- multi-db discovery ---

    #[test]
    fn discover_single_db_returns_empty() {
        let content = "\
[db]
dialect = \"postgres\"
user = \"postgres\"
database = \"mydb\"";
        let table = parse_table(content);
        assert!(discover_db_names(&table).is_empty());
    }

    #[test]
    fn discover_multi_db_returns_sorted_names() {
        let content = "\
[db.analytics]
dialect = \"postgres\"
user = \"readonly\"
database = \"analytics\"

[db.primary]
dialect = \"postgres\"
user = \"postgres\"
database = \"mydb\"";
        let table = parse_table(content);
        let names = discover_db_names(&table);
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "analytics");
        assert_eq!(names[1], "primary");
    }

    #[test]
    fn from_config_with_named_section() {
        let content = "\
[db.primary]
dialect = \"postgres\"
host = \"dbhost\"
port = 5433
user = \"admin\"
password = \"secret\"
database = \"myapp\"";
        let table = parse_table(content);
        let cfg = DbConfig::from_config(&table, "db.primary").unwrap();
        assert_eq!(cfg.dialect, Dialect::Postgres);
        assert_eq!(cfg.host, "dbhost");
        assert_eq!(cfg.port, 5433);
        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.password, "secret");
        assert_eq!(cfg.database, "myapp");
    }

    #[test]
    fn from_config_named_section_defaults() {
        let content = "\
[db.staging]
dialect = \"postgres\"
user = \"postgres\"
database = \"staging_db\"";
        let table = parse_table(content);
        let cfg = DbConfig::from_config(&table, "db.staging").unwrap();
        assert_eq!(cfg.host, "localhost");
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.password, "");
    }

    #[test]
    fn from_config_named_section_url() {
        let content = "\
[db.prod]
dialect = \"postgres\"
url = \"postgres://admin:secret@prod.example.com:5432/proddb\"";
        let table = parse_table(content);
        let cfg = DbConfig::from_config(&table, "db.prod").unwrap();
        assert_eq!(cfg.host, "prod.example.com");
        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.database, "proddb");
    }
}
