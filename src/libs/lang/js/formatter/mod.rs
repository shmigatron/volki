pub mod config;
pub mod formatter;
pub mod plugin_bridge;
pub mod tokenizer;
pub mod walker;

use std::path::{Path, PathBuf};

use config::FormatConfig;
use formatter::format_source;
use walker::{WalkConfig, walk_files};

use crate::core::plugins::registry::PluginRegistry;

#[derive(Debug)]
pub enum FileStatus {
    Unchanged,
    Changed,
    Error(String),
}

#[derive(Debug)]
pub struct FileResult {
    pub path: PathBuf,
    pub status: FileStatus,
}

pub fn format(root: &Path, config: &FormatConfig, plugins: Option<&PluginRegistry>) -> Vec<FileResult> {
    let walk_config = WalkConfig::default();
    let files = match walk_files(root, &walk_config) {
        Ok(f) => f,
        Err(e) => {
            return vec![FileResult {
                path: root.to_path_buf(),
                status: FileStatus::Error(e.to_string()),
            }];
        }
    };

    files.into_iter().map(|path| format_file(&path, config, plugins)).collect()
}

pub fn check(root: &Path, config: &FormatConfig, plugins: Option<&PluginRegistry>) -> Vec<FileResult> {
    let walk_config = WalkConfig::default();
    let files = match walk_files(root, &walk_config) {
        Ok(f) => f,
        Err(e) => {
            return vec![FileResult {
                path: root.to_path_buf(),
                status: FileStatus::Error(e.to_string()),
            }];
        }
    };

    files.into_iter().map(|path| check_file(&path, config, plugins)).collect()
}

fn format_file(path: &Path, config: &FormatConfig, plugins: Option<&PluginRegistry>) -> FileResult {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return FileResult {
            path: path.to_path_buf(),
            status: FileStatus::Error(e.to_string()),
        },
    };

    match format_source(&source, config, plugins) {
        Ok(formatted) => {
            if formatted == source {
                FileResult { path: path.to_path_buf(), status: FileStatus::Unchanged }
            } else {
                match std::fs::write(path, &formatted) {
                    Ok(_) => FileResult { path: path.to_path_buf(), status: FileStatus::Changed },
                    Err(e) => FileResult { path: path.to_path_buf(), status: FileStatus::Error(e.to_string()) },
                }
            }
        }
        Err(e) => FileResult {
            path: path.to_path_buf(),
            status: FileStatus::Error(e.to_string()),
        },
    }
}

fn check_file(path: &Path, config: &FormatConfig, plugins: Option<&PluginRegistry>) -> FileResult {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return FileResult {
            path: path.to_path_buf(),
            status: FileStatus::Error(e.to_string()),
        },
    };

    match format_source(&source, config, plugins) {
        Ok(formatted) => {
            if formatted == source {
                FileResult { path: path.to_path_buf(), status: FileStatus::Unchanged }
            } else {
                FileResult { path: path.to_path_buf(), status: FileStatus::Changed }
            }
        }
        Err(e) => FileResult {
            path: path.to_path_buf(),
            status: FileStatus::Error(e.to_string()),
        },
    }
}
