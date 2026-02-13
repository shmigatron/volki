use std::fs;
use std::path::Path;

use crate::libs::lang::shared::license::heuristic::detect_license_from_file;
use crate::libs::lang::shared::license::parsers::json::extract_top_level;
use crate::libs::lang::shared::license::scan_util::finalize_scan;
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);

    if !root.join("Package.swift").exists() {
        return Err(LicenseError::NoManifest(
            "No Package.swift found in project directory".to_string(),
        ));
    }

    let resolved_path = root.join("Package.resolved");
    if !resolved_path.exists() {
        return Err(LicenseError::NoDependencyDir(
            "No Package.resolved found (run swift package resolve first)".to_string(),
        ));
    }

    let project_name = read_project_name(root);

    let resolved_content = fs::read_to_string(&resolved_path)?;
    let deps = parse_package_resolved(&resolved_content);

    let checkouts = root.join(".build").join("checkouts");

    let mut packages = Vec::new();

    for (name, version) in &deps {
        let (license, source) = find_swift_package_license(name, &checkouts);
        let category = LicenseCategory::from_license_str(&license);

        packages.push(PackageLicense {
            name: name.clone(),
            version: version.clone(),
            license,
            category,
            source,
        });
    }

    Ok(finalize_scan(project_name, packages, config))
}

fn parse_package_resolved(content: &str) -> Vec<(String, String)> {
    let map = extract_top_level(content);
    let mut deps = Vec::new();

    // Package.resolved v2 format: { "pins": [ { "identity": "...", "state": { "version": "..." } } ] }
    if let Some(pins) = map.get("pins").and_then(|v| v.as_array()) {
        for pin in pins {
            if let Some(obj) = pin.as_object() {
                let identity = obj.get("identity").and_then(|v| v.as_str()).unwrap_or("");
                let version = obj
                    .get("state")
                    .and_then(|v| v.as_object())
                    .and_then(|s| s.get("version"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0");

                if !identity.is_empty() {
                    deps.push((identity.to_string(), version.to_string()));
                }
            }
        }
        return deps;
    }

    // Package.resolved v1 format: { "object": { "pins": [...] } }
    if let Some(object) = map.get("object").and_then(|v| v.as_object()) {
        if let Some(pins) = object.get("pins").and_then(|v| v.as_array()) {
            for pin in pins {
                if let Some(obj) = pin.as_object() {
                    let package = obj.get("package").and_then(|v| v.as_str()).unwrap_or("");
                    let version = obj
                        .get("state")
                        .and_then(|v| v.as_object())
                        .and_then(|s| s.get("version"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.0.0");

                    if !package.is_empty() {
                        deps.push((package.to_string(), version.to_string()));
                    }
                }
            }
        }
    }

    deps
}

fn find_swift_package_license(name: &str, checkouts: &Path) -> (String, LicenseSource) {
    if checkouts.is_dir() {
        let pkg_dir = checkouts.join(name);
        if pkg_dir.is_dir() {
            if let Some(l) = detect_license_from_file(&pkg_dir) {
                return (l, LicenseSource::LicenseFile);
            }
        }
    }
    ("UNKNOWN".to_string(), LicenseSource::NotFound)
}

fn read_project_name(root: &Path) -> String {
    let pkg_swift = root.join("Package.swift");
    if let Ok(content) = fs::read_to_string(&pkg_swift) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("name:") {
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start + 1..].find('"') {
                        return trimmed[start + 1..start + 1 + end].to_string();
                    }
                }
            }
        }
    }
    "unnamed".to_string()
}
