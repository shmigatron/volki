use std::fmt;
use std::io;
use std::path::Path;

use crate::core::package::detect::detector::detect;
use crate::core::package::detect::types::PackageManager;
use crate::libs::lang::shared::license::parsers::json::extract_top_level;
use crate::libs::lang::shared::process::{ProcessError, run_command_allow_failure};

#[derive(Debug)]
pub struct OutdatedResult {
    pub packages: Vec<OutdatedPackage>,
    pub total: usize,
}

#[derive(Debug)]
pub struct OutdatedPackage {
    pub name: String,
    pub current: String,
    pub wanted: String,
    pub latest: String,
    pub severity: UpdateSeverity,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateSeverity {
    Patch,
    Minor,
    Major,
}

impl fmt::Display for UpdateSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateSeverity::Patch => write!(f, "patch"),
            UpdateSeverity::Minor => write!(f, "minor"),
            UpdateSeverity::Major => write!(f, "major"),
        }
    }
}

#[derive(Debug)]
pub enum OutdatedError {
    Io(io::Error),
    Process(ProcessError),
    NoProject(String),
    NotNodeProject(String),
    ParseError(String),
}

impl fmt::Display for OutdatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutdatedError::Io(e) => write!(f, "IO error: {e}"),
            OutdatedError::Process(e) => write!(f, "{e}"),
            OutdatedError::NoProject(msg) => write!(f, "{msg}"),
            OutdatedError::NotNodeProject(msg) => write!(f, "{msg}"),
            OutdatedError::ParseError(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl From<io::Error> for OutdatedError {
    fn from(e: io::Error) -> Self {
        OutdatedError::Io(e)
    }
}

impl From<ProcessError> for OutdatedError {
    fn from(e: ProcessError) -> Self {
        OutdatedError::Process(e)
    }
}

pub fn check(root: &Path, include_dev: bool) -> Result<OutdatedResult, OutdatedError> {
    let projects = detect(root).map_err(|e| OutdatedError::NoProject(e.to_string()))?;

    let node_project = projects
        .iter()
        .find(|p| {
            matches!(
                p.manager,
                PackageManager::Npm
                    | PackageManager::Yarn
                    | PackageManager::Pnpm
                    | PackageManager::Bun
            )
        })
        .ok_or_else(|| OutdatedError::NotNodeProject("No Node.js project detected".to_string()))?;

    match node_project.manager {
        PackageManager::Npm => check_npm(root, include_dev),
        PackageManager::Yarn => check_yarn(root, include_dev),
        PackageManager::Pnpm => check_pnpm(root, include_dev),
        PackageManager::Bun => check_bun(root, include_dev),
        _ => Err(OutdatedError::NotNodeProject(
            "Unsupported package manager".to_string(),
        )),
    }
}

fn check_npm(root: &Path, _include_dev: bool) -> Result<OutdatedResult, OutdatedError> {
    // npm outdated --json exits non-zero when outdated packages exist
    let output = run_command_allow_failure("npm", &["outdated", "--json"], root)?;

    if output.stdout.trim().is_empty() {
        return Ok(OutdatedResult {
            packages: vec![],
            total: 0,
        });
    }

    let map = extract_top_level(&output.stdout);
    let mut packages = Vec::new();

    for (name, value) in &map {
        if let Some(obj) = value.as_object() {
            let current = obj
                .get("current")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            let wanted = obj
                .get("wanted")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            let latest = obj
                .get("latest")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();

            let severity = compute_severity(&current, &latest);

            packages.push(OutdatedPackage {
                name: name.clone(),
                current,
                wanted,
                latest,
                severity,
            });
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    let total = packages.len();

    Ok(OutdatedResult { packages, total })
}

fn check_yarn(root: &Path, _include_dev: bool) -> Result<OutdatedResult, OutdatedError> {
    let output = run_command_allow_failure("yarn", &["outdated", "--json"], root)?;

    if output.stdout.trim().is_empty() {
        return Ok(OutdatedResult {
            packages: vec![],
            total: 0,
        });
    }

    // yarn outdated --json outputs one JSON object per line (NDJSON)
    let mut packages = Vec::new();
    for line in output.stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let map = extract_top_level(line);
        // yarn v1 format: {"type":"table","data":{"body":[["name","current","wanted","latest",...]]}}
        if let Some(data) = map.get("data") {
            if let Some(data_obj) = data.as_object() {
                if let Some(body) = data_obj.get("body") {
                    if let Some(rows) = body.as_array() {
                        for row in rows {
                            if let Some(cols) = row.as_array() {
                                if cols.len() >= 4 {
                                    let name =
                                        cols[0].as_str().unwrap_or("?").to_string();
                                    let current =
                                        cols[1].as_str().unwrap_or("?").to_string();
                                    let wanted =
                                        cols[2].as_str().unwrap_or("?").to_string();
                                    let latest =
                                        cols[3].as_str().unwrap_or("?").to_string();
                                    let severity = compute_severity(&current, &latest);
                                    packages.push(OutdatedPackage {
                                        name,
                                        current,
                                        wanted,
                                        latest,
                                        severity,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    let total = packages.len();

    Ok(OutdatedResult { packages, total })
}

fn check_pnpm(root: &Path, _include_dev: bool) -> Result<OutdatedResult, OutdatedError> {
    let output =
        run_command_allow_failure("pnpm", &["outdated", "--format", "json"], root)?;

    if output.stdout.trim().is_empty() || output.stdout.trim() == "[]" {
        return Ok(OutdatedResult {
            packages: vec![],
            total: 0,
        });
    }

    // pnpm outdated --format json returns a JSON object keyed by package name
    let map = extract_top_level(&output.stdout);
    let mut packages = Vec::new();

    for (name, value) in &map {
        if let Some(obj) = value.as_object() {
            let current = obj
                .get("current")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            let wanted = obj
                .get("wanted")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            let latest = obj
                .get("latest")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();

            let severity = compute_severity(&current, &latest);
            packages.push(OutdatedPackage {
                name: name.clone(),
                current,
                wanted,
                latest,
                severity,
            });
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    let total = packages.len();

    Ok(OutdatedResult { packages, total })
}

fn check_bun(root: &Path, _include_dev: bool) -> Result<OutdatedResult, OutdatedError> {
    // bun outdated outputs text, not JSON — parse it
    let output = run_command_allow_failure("bun", &["outdated"], root)?;

    if output.stdout.trim().is_empty() {
        return Ok(OutdatedResult {
            packages: vec![],
            total: 0,
        });
    }

    let mut packages = Vec::new();

    // bun outdated format: table with columns: Package, Current, Update, Latest
    for line in output.stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Package") || line.starts_with("─") || line.starts_with('|') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let name = parts[0].to_string();
            let current = parts[1].to_string();
            let wanted = parts[2].to_string();
            let latest = parts[3].to_string();
            let severity = compute_severity(&current, &latest);
            packages.push(OutdatedPackage {
                name,
                current,
                wanted,
                latest,
                severity,
            });
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    let total = packages.len();

    Ok(OutdatedResult { packages, total })
}

fn compute_severity(current: &str, latest: &str) -> UpdateSeverity {
    let cur_parts = parse_version(current);
    let lat_parts = parse_version(latest);

    if cur_parts.0 != lat_parts.0 {
        UpdateSeverity::Major
    } else if cur_parts.1 != lat_parts.1 {
        UpdateSeverity::Minor
    } else {
        UpdateSeverity::Patch
    }
}

fn parse_version(v: &str) -> (u32, u32, u32) {
    let v = v.trim_start_matches(|c: char| !c.is_ascii_digit());
    let parts: Vec<u32> = v
        .split('.')
        .take(3)
        .filter_map(|s| {
            s.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()
        })
        .collect();

    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

/// Detect the package manager for the project at the given root.
pub fn detect_package_manager(root: &Path) -> Result<PackageManager, OutdatedError> {
    let projects = detect(root).map_err(|e| OutdatedError::NoProject(e.to_string()))?;
    projects
        .iter()
        .find(|p| {
            matches!(
                p.manager,
                PackageManager::Npm
                    | PackageManager::Yarn
                    | PackageManager::Pnpm
                    | PackageManager::Bun
            )
        })
        .map(|p| p.manager.clone())
        .ok_or_else(|| OutdatedError::NotNodeProject("No Node.js project detected".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_major() {
        assert_eq!(compute_severity("1.0.0", "2.0.0"), UpdateSeverity::Major);
    }

    #[test]
    fn severity_minor() {
        assert_eq!(compute_severity("1.0.0", "1.1.0"), UpdateSeverity::Minor);
    }

    #[test]
    fn severity_patch() {
        assert_eq!(compute_severity("1.0.0", "1.0.1"), UpdateSeverity::Patch);
    }

    #[test]
    fn severity_same() {
        assert_eq!(compute_severity("1.0.0", "1.0.0"), UpdateSeverity::Patch);
    }

    #[test]
    fn parse_version_basic() {
        assert_eq!(parse_version("1.2.3"), (1, 2, 3));
    }

    #[test]
    fn parse_version_with_prefix() {
        assert_eq!(parse_version("^1.2.3"), (1, 2, 3));
    }

    #[test]
    fn parse_version_partial() {
        assert_eq!(parse_version("1.2"), (1, 2, 0));
    }

    #[test]
    fn parse_version_with_prerelease() {
        assert_eq!(parse_version("1.2.3-beta.1"), (1, 2, 3));
    }

    #[test]
    fn outdated_error_display() {
        let err = OutdatedError::NoProject("no project".to_string());
        assert_eq!(format!("{err}"), "no project");

        let err = OutdatedError::NotNodeProject("not node".to_string());
        assert_eq!(format!("{err}"), "not node");
    }

    #[test]
    fn severity_display() {
        assert_eq!(format!("{}", UpdateSeverity::Patch), "patch");
        assert_eq!(format!("{}", UpdateSeverity::Minor), "minor");
        assert_eq!(format!("{}", UpdateSeverity::Major), "major");
    }
}
