use std::fs;
use std::path::Path;

use crate::libs::lang::shared::license::heuristic::detect_license_from_file;
use crate::libs::lang::shared::license::parsers::json::{extract_top_level, JsonValue};
use crate::libs::lang::shared::license::scan_util::finalize_scan;
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};
use crate::{log_debug, log_warn};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);
    let pkg_json_path = root.join("package.json");
    let node_modules = root.join("node_modules");

    if !pkg_json_path.exists() {
        return Err(LicenseError::NoManifest(
            "No package.json found in project directory".to_string(),
        ));
    }
    if !node_modules.exists() {
        return Err(LicenseError::NoDependencyDir(
            "No node_modules directory found (run npm install first)".to_string(),
        ));
    }

    log_debug!("scanning node_modules at {}", node_modules.display());

    // Read root package.json for project name and dependency list
    let root_json = fs::read_to_string(&pkg_json_path)?;
    let root_info = extract_package_info(&root_json);
    let _declared_deps = extract_dependencies(&root_json, config.include_dev);
    let project_name = if root_info.name.is_empty() {
        "unnamed".to_string()
    } else {
        root_info.name
    };

    // Walk node_modules
    let mut packages = Vec::new();
    walk_node_modules(&node_modules, &mut packages)?;

    log_debug!("found {} packages in node_modules", packages.len());

    Ok(finalize_scan(project_name, packages, config))
}

fn walk_node_modules(
    node_modules: &Path,
    packages: &mut Vec<PackageLicense>,
) -> Result<(), LicenseError> {
    let entries = fs::read_dir(node_modules)?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden dirs and non-directories
        if name_str.starts_with('.') {
            continue;
        }

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Handle scoped packages (@scope/pkg)
        if name_str.starts_with('@') {
            let scope_entries = fs::read_dir(&path)?;
            for scope_entry in scope_entries {
                let scope_entry = scope_entry?;
                let scope_path = scope_entry.path();
                if scope_path.is_dir() {
                    let scoped_name =
                        format!("{}/{}", name_str, scope_entry.file_name().to_string_lossy());
                    if let Some(pkg) = read_package(&scope_path, &scoped_name) {
                        packages.push(pkg);
                    }
                }
            }
        } else if let Some(pkg) = read_package(&path, &name_str) {
            packages.push(pkg);
        }
    }
    Ok(())
}

fn read_package(dir: &Path, fallback_name: &str) -> Option<PackageLicense> {
    let pkg_json_path = dir.join("package.json");
    let json_content = fs::read_to_string(&pkg_json_path).ok()?;
    let info = extract_package_info(&json_content);

    let name = if info.name.is_empty() {
        fallback_name.to_string()
    } else {
        info.name
    };

    let (license, source) = match info.license {
        Some(l) if !l.is_empty() => {
            let source = if json_content.contains("\"licenses\"") && l.contains(" OR ") {
                LicenseSource::ManifestLegacy
            } else {
                LicenseSource::ManifestField
            };
            (l, source)
        }
        _ => match detect_license_from_file(dir) {
            Some(l) => (l, LicenseSource::LicenseFile),
            None => {
                log_warn!("no license found for {}", name);
                ("UNKNOWN".to_string(), LicenseSource::NotFound)
            }
        },
    };

    let category = LicenseCategory::from_license_str(&license);

    Some(PackageLicense {
        name,
        version: info.version,
        license,
        category,
        source,
    })
}

// --- JS-specific JSON extraction (moved from json.rs) ---

struct PackageInfo {
    name: String,
    version: String,
    license: Option<String>,
}

fn extract_package_info(json: &str) -> PackageInfo {
    let map = extract_top_level(json);

    let name = map
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let version = map
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();

    // Try "license" field (string)
    if let Some(val) = map.get("license") {
        match val {
            JsonValue::Str(s) => {
                return PackageInfo {
                    name,
                    version,
                    license: Some(s.clone()),
                };
            }
            JsonValue::Object(obj) => {
                // { "type": "MIT" } format
                if let Some(t) = obj.get("type").and_then(|v| v.as_str()) {
                    return PackageInfo {
                        name,
                        version,
                        license: Some(t.to_string()),
                    };
                }
            }
            _ => {}
        }
    }

    // Try "licenses" field (legacy array format)
    if let Some(val) = map.get("licenses") {
        if let Some(arr) = val.as_array() {
            let mut parts = Vec::new();
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if let Some(t) = obj.get("type").and_then(|v| v.as_str()) {
                        parts.push(t.to_string());
                    }
                }
            }
            if !parts.is_empty() {
                return PackageInfo {
                    name,
                    version,
                    license: Some(parts.join(" OR ")),
                };
            }
        }
    }

    PackageInfo {
        name,
        version,
        license: None,
    }
}

fn extract_dependencies(json: &str, include_dev: bool) -> Vec<String> {
    let map = extract_top_level(json);
    let mut deps = Vec::new();

    if let Some(val) = map.get("dependencies") {
        if let Some(obj) = val.as_object() {
            deps.extend(obj.keys().cloned());
        }
    }

    if include_dev {
        if let Some(val) = map.get("devDependencies") {
            if let Some(obj) = val.as_object() {
                deps.extend(obj.keys().cloned());
            }
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- extract_package_info ---

    #[test]
    fn package_info_simple() {
        let json = r#"{"name": "lodash", "version": "4.17.21", "license": "MIT"}"#;
        let info = extract_package_info(json);
        assert_eq!(info.name, "lodash");
        assert_eq!(info.version, "4.17.21");
        assert_eq!(info.license, Some("MIT".to_string()));
    }

    #[test]
    fn package_info_license_object() {
        let json = r#"{"name": "old-pkg", "version": "1.0.0", "license": {"type": "MIT", "url": "https://..."}}"#;
        let info = extract_package_info(json);
        assert_eq!(info.license, Some("MIT".to_string()));
    }

    #[test]
    fn package_info_licenses_array() {
        let json = r#"{"name": "dual", "version": "1.0.0", "licenses": [{"type": "MIT"}, {"type": "Apache-2.0"}]}"#;
        let info = extract_package_info(json);
        assert_eq!(info.license, Some("MIT OR Apache-2.0".to_string()));
    }

    #[test]
    fn package_info_no_license() {
        let json = r#"{"name": "nolic", "version": "1.0.0"}"#;
        let info = extract_package_info(json);
        assert!(info.license.is_none());
    }

    #[test]
    fn package_info_empty_name_no_version() {
        let json = r#"{}"#;
        let info = extract_package_info(json);
        assert_eq!(info.name, "");
        assert_eq!(info.version, "0.0.0");
    }

    // --- extract_dependencies ---

    #[test]
    fn deps_prod_only() {
        let json = r#"{"dependencies": {"lodash": "^4.0.0"}, "devDependencies": {"jest": "^29.0.0"}}"#;
        let deps = extract_dependencies(json, false);
        assert!(deps.contains(&"lodash".to_string()));
        assert!(!deps.contains(&"jest".to_string()));
    }

    #[test]
    fn deps_with_dev() {
        let json = r#"{"dependencies": {"lodash": "^4.0.0"}, "devDependencies": {"jest": "^29.0.0"}}"#;
        let deps = extract_dependencies(json, true);
        assert!(deps.contains(&"lodash".to_string()));
        assert!(deps.contains(&"jest".to_string()));
    }

    #[test]
    fn deps_none() {
        let json = r#"{"name": "empty"}"#;
        let deps = extract_dependencies(json, true);
        assert!(deps.is_empty());
    }
}
