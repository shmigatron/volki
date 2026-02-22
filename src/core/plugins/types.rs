use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::io::IoError;
use crate::core::volkiwithstds::path::PathBuf;
use core::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum PluginRuntime {
    Node,
    Python,
}

impl PluginRuntime {
    pub fn command(&self) -> &str {
        match self {
            PluginRuntime::Node => "node",
            PluginRuntime::Python => "python3",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "node" => Some(PluginRuntime::Node),
            "python3" | "python" => Some(PluginRuntime::Python),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedPlugin {
    pub name: String,
    pub runtime: PluginRuntime,
    pub entry_point: PathBuf,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PluginSpec {
    pub name: String,
    pub runtime: Option<PluginRuntime>,
    pub options: Vec<(String, String)>,
}

#[derive(Debug)]
pub enum PluginError {
    NotFound(String),
    RuntimeNotAvailable(String),
    SpawnFailed(String),
    Timeout,
    InvalidResponse(String),
    PluginStderr(String),
    IoError(IoError),
    ConfigError(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::NotFound(name) => write!(f, "plugin not found: {name}"),
            PluginError::RuntimeNotAvailable(rt) => write!(f, "runtime not available: {rt}"),
            PluginError::SpawnFailed(msg) => write!(f, "failed to spawn plugin: {msg}"),
            PluginError::Timeout => write!(f, "plugin timed out"),
            PluginError::InvalidResponse(msg) => write!(f, "invalid plugin response: {msg}"),
            PluginError::PluginStderr(msg) => write!(f, "plugin error: {msg}"),
            PluginError::IoError(e) => write!(f, "IO error: {e}"),
            PluginError::ConfigError(msg) => write!(f, "config error: {msg}"),
        }
    }
}

impl From<IoError> for PluginError {
    fn from(e: IoError) -> Self {
        PluginError::IoError(e)
    }
}
