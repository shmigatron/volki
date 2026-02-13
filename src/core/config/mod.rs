pub mod parser;

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::core::package::detect::types::DetectedProject;
use crate::core::plugins::types::PluginSpec;
use crate::{log_debug, log_error};

const CONFIG_FILENAME: &str = "volki.toml";

const DEFAULT_CONFIG: &str = "\
[volki]
";

#[derive(Debug, Clone)]
pub struct VolkiConfig {
    pub path: PathBuf,
    table: parser::Table,
}

impl VolkiConfig {
    pub fn load(dir: &Path) -> Result<Self, ConfigError> {
        let path = dir.join(CONFIG_FILENAME);
        if !path.is_file() {
            log_error!("config not found: {}", path.display());
            return Err(ConfigError::NotFound(path));
        }

        log_debug!("loading config from {}", path.display());
        let content = std::fs::read_to_string(&path)?;
        let table = parser::parse(&content)?;

        Ok(VolkiConfig { path, table })
    }

    pub fn init(dir: &Path, projects: &[DetectedProject]) -> Result<PathBuf, ConfigError> {
        let path = dir.join(CONFIG_FILENAME);
        if path.exists() {
            log_error!("config already exists: {}", path.display());
            return Err(ConfigError::AlreadyExists(path));
        }
        log_debug!("writing config to {}", path.display());

        let content = if let Some(project) = projects.first() {
            let mut buf = String::from("[volki]\n");
            buf.push_str(&format!("ecosystem = \"{}\"\n", project.ecosystem.as_toml_str()));
            buf.push_str(&format!("manager = \"{}\"\n", project.manager.as_toml_str()));
            if let Some(ref fw) = project.framework {
                buf.push_str(&format!("framework = \"{}\"\n", fw.as_toml_str()));
            }
            buf
        } else {
            DEFAULT_CONFIG.to_string()
        };

        std::fs::write(&path, content)?;
        Ok(path)
    }

    pub fn table(&self) -> &parser::Table {
        &self.table
    }

    pub fn plugin_specs(&self) -> Vec<PluginSpec> {
        let list = match self.table.get("plugins", "list") {
            Some(v) => match v.as_str_array() {
                Some(names) => names,
                None => return Vec::new(),
            },
            None => return Vec::new(),
        };

        list.iter()
            .map(|name| {
                let options = self.table.entries_with_prefix(&format!("plugins.{name}"));
                PluginSpec {
                    name: name.to_string(),
                    runtime: None,
                    options,
                }
            })
            .collect()
    }
}

#[derive(Debug)]
pub enum ConfigError {
    NotFound(PathBuf),
    AlreadyExists(PathBuf),
    Io(io::Error),
    Parse(parser::ParseError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NotFound(p) => write!(f, "config not found: {}", p.display()),
            ConfigError::AlreadyExists(p) => {
                write!(f, "config already exists: {}", p.display())
            }
            ConfigError::Io(e) => write!(f, "IO error: {e}"),
            ConfigError::Parse(e) => write!(f, "{e}"),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<parser::ParseError> for ConfigError {
    fn from(e: parser::ParseError) -> Self {
        ConfigError::Parse(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::package::detect::types::{Ecosystem, Framework, PackageManager};
    use std::fs;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("volki_config_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn init_creates_file_empty_projects() {
        let dir = tmp("init_empty");
        let path = VolkiConfig::init(&dir, &[]).unwrap();
        assert!(path.is_file());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("[volki]"));
        assert!(!content.contains("ecosystem"));
        cleanup(&dir);
    }

    #[test]
    fn init_creates_file_with_project() {
        let dir = tmp("init_proj");
        let project = DetectedProject {
            ecosystem: Ecosystem::Node,
            manager: PackageManager::Npm,
            manifest: dir.join("package.json"),
            lock_file: None,
            framework: Some(Framework::NextJs),
        };
        let path = VolkiConfig::init(&dir, &[project]).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("[volki]"));
        assert!(content.contains("ecosystem = \"node\""));
        assert!(content.contains("manager = \"npm\""));
        assert!(content.contains("framework = \"nextjs\""));
        cleanup(&dir);
    }

    #[test]
    fn init_creates_file_without_framework() {
        let dir = tmp("init_nofw");
        let project = DetectedProject {
            ecosystem: Ecosystem::Rust,
            manager: PackageManager::Cargo,
            manifest: dir.join("Cargo.toml"),
            lock_file: None,
            framework: None,
        };
        let path = VolkiConfig::init(&dir, &[project]).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("ecosystem = \"rust\""));
        assert!(content.contains("manager = \"cargo\""));
        assert!(!content.contains("framework"));
        cleanup(&dir);
    }

    #[test]
    fn init_fails_if_exists() {
        let dir = tmp("init_exists");
        VolkiConfig::init(&dir, &[]).unwrap();
        let result = VolkiConfig::init(&dir, &[]);
        assert!(matches!(result, Err(ConfigError::AlreadyExists(_))));
        cleanup(&dir);
    }

    #[test]
    fn load_not_found() {
        let dir = tmp("load_missing");
        let result = VolkiConfig::load(&dir);
        assert!(matches!(result, Err(ConfigError::NotFound(_))));
        cleanup(&dir);
    }

    #[test]
    fn load_valid() {
        let dir = tmp("load_valid");
        VolkiConfig::init(&dir, &[]).unwrap();
        let config = VolkiConfig::load(&dir).unwrap();
        assert!(config.path.ends_with(CONFIG_FILENAME));
        cleanup(&dir);
    }
}
