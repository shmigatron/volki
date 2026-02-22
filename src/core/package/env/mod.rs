use crate::core::volkiwithstds::collections::{String, HashMap};
use crate::core::volkiwithstds::path::Path;

/// Parse a `.env` file into key-value pairs.
///
/// Supports:
/// - `KEY=VALUE`
/// - `KEY="VALUE"` and `KEY='VALUE'` (strips quotes)
/// - Comments (`#`) and blank lines are skipped
/// - Leading `export ` prefix is stripped
pub fn parse_dotenv(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Strip optional "export " prefix
        let trimmed = trimmed.strip_prefix("export ").unwrap_or(trimmed);

        let Some(eq) = trimmed.find('=') else {
            continue;
        };

        let key = trimmed[..eq].trim();
        if key.is_empty() {
            continue;
        }

        let val = trimmed[eq + 1..].trim();

        // Strip surrounding quotes
        let val = if (val.starts_with('"') && val.ends_with('"'))
            || (val.starts_with('\'') && val.ends_with('\''))
        {
            &val[1..val.len() - 1]
        } else {
            val
        };

        map.insert(String::from(key), String::from(val));
    }

    map
}

/// Load and parse a `.env` file from the given directory.
/// Returns an empty map if the file doesn't exist.
pub fn load_dotenv(dir: &Path) -> HashMap<String, String> {
    let path = dir.join(".env");
    match crate::core::volkiwithstds::fs::read_to_string(&path) {
        Ok(content) => parse_dotenv(&content),
        Err(_) => HashMap::new(),
    }
}

/// Look up a value by trying the process environment first,
/// then falling back to the dotenv map.
pub fn get_env_or_dotenv(key: &str, dotenv: &HashMap<String, String>) -> Option<String> {
    crate::core::volkiwithstds::env::var(key).or_else(|| dotenv.get(&String::from(key)).cloned())
}

/// Try multiple env var names in order, returning the first found.
pub fn get_first_env(keys: &[&str], dotenv: &HashMap<String, String>) -> Option<String> {
    for &key in keys {
        if let Some(val) = get_env_or_dotenv(key, dotenv) {
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vformat;
    use crate::core::volkiwithstds::fs;
    use crate::core::volkiwithstds::path::PathBuf;

    // --- parse_dotenv ---

    #[test]
    fn parse_empty() {
        let map = parse_dotenv("");
        assert!(map.is_empty());
    }

    #[test]
    fn parse_comments_and_blanks() {
        let map = parse_dotenv("# comment\n\n  # another\n");
        assert!(map.is_empty());
    }

    #[test]
    fn parse_basic_key_value() {
        let map = parse_dotenv("FOO=bar\nBAZ=qux");
        assert_eq!(map.get("FOO").unwrap().as_str(), "bar");
        assert_eq!(map.get("BAZ").unwrap().as_str(), "qux");
    }

    #[test]
    fn parse_double_quoted() {
        let map = parse_dotenv("KEY=\"hello world\"");
        assert_eq!(map.get("KEY").unwrap().as_str(), "hello world");
    }

    #[test]
    fn parse_single_quoted() {
        let map = parse_dotenv("KEY='hello world'");
        assert_eq!(map.get("KEY").unwrap().as_str(), "hello world");
    }

    #[test]
    fn parse_export_prefix() {
        let map = parse_dotenv("export DB_HOST=localhost");
        assert_eq!(map.get("DB_HOST").unwrap().as_str(), "localhost");
    }

    #[test]
    fn parse_whitespace_around_equals() {
        let map = parse_dotenv("  KEY  =  value  ");
        assert_eq!(map.get("KEY").unwrap().as_str(), "value");
    }

    #[test]
    fn parse_empty_value() {
        let map = parse_dotenv("KEY=");
        assert_eq!(map.get("KEY").unwrap().as_str(), "");
    }

    #[test]
    fn parse_empty_quoted_value() {
        let map = parse_dotenv("KEY=\"\"");
        assert_eq!(map.get("KEY").unwrap().as_str(), "");
    }

    #[test]
    fn parse_no_equals_skipped() {
        let map = parse_dotenv("NOEQUALSSIGN");
        assert!(map.is_empty());
    }

    #[test]
    fn parse_empty_key_skipped() {
        let map = parse_dotenv("=value");
        assert!(map.is_empty());
    }

    #[test]
    fn parse_url_value() {
        let map = parse_dotenv("DATABASE_URL=postgres://user:pass@localhost:5432/mydb");
        assert_eq!(
            map.get("DATABASE_URL").unwrap().as_str(),
            "postgres://user:pass@localhost:5432/mydb"
        );
    }

    #[test]
    fn parse_mixed() {
        let input = "\
# Database config
DB_HOST=localhost
DB_PORT=5432
DB_USER=\"admin\"
export DB_PASSWORD='secret'

# App config
APP_ENV=production
";
        let map = parse_dotenv(input);
        assert_eq!(map.len(), 5);
        assert_eq!(map.get("DB_HOST").unwrap().as_str(), "localhost");
        assert_eq!(map.get("DB_PORT").unwrap().as_str(), "5432");
        assert_eq!(map.get("DB_USER").unwrap().as_str(), "admin");
        assert_eq!(map.get("DB_PASSWORD").unwrap().as_str(), "secret");
        assert_eq!(map.get("APP_ENV").unwrap().as_str(), "production");
    }

    // --- load_dotenv ---

    fn tmp(name: &str) -> PathBuf {
        let dir = crate::core::volkiwithstds::env::temp_dir()
            .join(&vformat!("volki_env_{}_{name}", crate::core::volkiwithstds::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_dotenv_missing_file() {
        let dir = tmp("missing");
        let map = load_dotenv(&dir);
        assert!(map.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn load_dotenv_reads_file() {
        let dir = tmp("present");
        fs::write(&dir.join(".env"), "MY_KEY=my_value\n".as_bytes()).unwrap();
        let map = load_dotenv(&dir);
        assert_eq!(map.get("MY_KEY").unwrap().as_str(), "my_value");
        cleanup(&dir);
    }

    // --- get_first_env ---

    #[test]
    fn get_first_env_from_dotenv() {
        let mut dotenv = HashMap::new();
        dotenv.insert(String::from("DB_URL"), String::from("postgres://x@y/z"));
        let val = get_first_env(&["DATABASE_URL", "DB_URL"], &dotenv);
        assert_eq!(val.unwrap().as_str(), "postgres://x@y/z");
    }

    #[test]
    fn get_first_env_none() {
        let dotenv = HashMap::new();
        let val = get_first_env(&["DOES_NOT_EXIST_ABC123"], &dotenv);
        assert!(val.is_none());
    }

    #[test]
    fn get_first_env_skips_empty() {
        let mut dotenv = HashMap::new();
        dotenv.insert(String::from("EMPTY"), String::new());
        dotenv.insert(String::from("REAL"), String::from("value"));
        let val = get_first_env(&["EMPTY", "REAL"], &dotenv);
        assert_eq!(val.unwrap().as_str(), "value");
    }
}
