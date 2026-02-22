use crate::core::volkiwithstds::collections::ToString;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fmt;
use crate::core::volkiwithstds::path::Path;

use crate::core::package::detect::types::PackageManager;
use crate::libs::lang::shared::process::{ProcessError, run_command};

#[derive(Debug)]
pub struct UpdateResult {
    pub package: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug)]
pub enum UpdateError {
    Process(ProcessError),
    UnsupportedManager(String),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateError::Process(e) => write!(f, "{e}"),
            UpdateError::UnsupportedManager(m) => {
                write!(f, "unsupported package manager: {m}")
            }
        }
    }
}

impl From<ProcessError> for UpdateError {
    fn from(e: ProcessError) -> Self {
        UpdateError::Process(e)
    }
}

pub fn update_packages(
    root: &Path,
    manager: &PackageManager,
    packages: &[String],
    latest: bool,
) -> Vec<UpdateResult> {
    packages
        .iter()
        .map(|pkg| update_single(root, manager, pkg, latest))
        .collect()
}

fn update_single(
    root: &Path,
    manager: &PackageManager,
    package: &str,
    latest: bool,
) -> UpdateResult {
    let result = match manager {
        PackageManager::Npm => {
            if latest {
                run_command(
                    "npm",
                    &["install", &crate::vformat!("{package}@latest")],
                    root,
                )
            } else {
                run_command("npm", &["update", package], root)
            }
        }
        PackageManager::Yarn => {
            if latest {
                run_command("yarn", &["add", &crate::vformat!("{package}@latest")], root)
            } else {
                run_command("yarn", &["upgrade", package], root)
            }
        }
        PackageManager::Pnpm => {
            if latest {
                run_command("pnpm", &["update", package, "--latest"], root)
            } else {
                run_command("pnpm", &["update", package], root)
            }
        }
        PackageManager::Bun => {
            if latest {
                run_command("bun", &["add", &crate::vformat!("{package}@latest")], root)
            } else {
                run_command("bun", &["update", package], root)
            }
        }
        _ => {
            return UpdateResult {
                package: package.to_vstring(),
                success: false,
                message: crate::vformat!("Unsupported package manager: {manager}"),
            };
        }
    };

    match result {
        Ok(output) => UpdateResult {
            package: package.to_vstring(),
            success: true,
            message: output,
        },
        Err(e) => UpdateResult {
            package: package.to_vstring(),
            success: false,
            message: e.to_vstring(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_error_display() {
        let err = UpdateError::UnsupportedManager(crate::vstr!("cargo"));
        assert!(crate::vformat!("{err}").contains("cargo"));
    }

    #[test]
    fn unsupported_manager_returns_failure() {
        let result = update_single(Path::new("."), &PackageManager::Cargo, "lodash", false);
        assert!(!result.success);
        assert!(result.message.contains("Unsupported"));
    }
}
