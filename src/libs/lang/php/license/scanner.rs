use std::fs;
use std::path::Path;

use crate::libs::lang::shared::license::heuristic::detect_license_from_file;
use crate::libs::lang::shared::license::parsers::json::{extract_top_level, JsonValue};
use crate::libs::lang::shared::license::scan_util::finalize_scan;
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);
    let lock_path = root.join("composer.lock");

    if !root.join("composer.json").exists() {
        return Err(LicenseError::NoManifest(
            "No composer.json found in project directory".to_string(),
        ));
    }
    if !lock_path.exists() {
        return Err(LicenseError::NoDependencyDir(
            "No composer.lock found (run composer install first)".to_string(),
        ));
    }

    let project_name = read_project_name(&root.join("composer.json"));

    let lock_content = fs::read_to_string(&lock_path)?;
    let lock_map = extract_top_level(&lock_content);

    let mut packages = Vec::new();

    if let Some(pkgs) = lock_map.get("packages").and_then(|v| v.as_array()) {
        for pkg in pkgs {
            if let Some(pl) = parse_composer_package(pkg, &root) {
                packages.push(pl);
            }
        }
    }

    if config.include_dev {
        if let Some(pkgs) = lock_map.get("packages-dev").and_then(|v| v.as_array()) {
            for pkg in pkgs {
                if let Some(pl) = parse_composer_package(pkg, &root) {
                    packages.push(pl);
                }
            }
        }
    }

    Ok(finalize_scan(project_name, packages, config))
}

fn parse_composer_package(value: &JsonValue, root: &Path) -> Option<PackageLicense> {
    let obj = value.as_object()?;

    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let version = obj
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .trim_start_matches('v')
        .to_string();

    if name.is_empty() {
        return None;
    }

    // License is in the "license" array field
    let (license, source) = if let Some(lic_val) = obj.get("license") {
        if let Some(arr) = lic_val.as_array() {
            let parts: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            if !parts.is_empty() {
                (parts.join(" OR "), LicenseSource::LockfileField)
            } else {
                try_license_file(&name, root)
            }
        } else if let Some(s) = lic_val.as_str() {
            (s.to_string(), LicenseSource::LockfileField)
        } else {
            try_license_file(&name, root)
        }
    } else {
        try_license_file(&name, root)
    };

    let category = LicenseCategory::from_license_str(&license);

    Some(PackageLicense {
        name,
        version,
        license,
        category,
        source,
    })
}

fn try_license_file(name: &str, root: &Path) -> (String, LicenseSource) {
    let vendor_dir = root.join("vendor").join(name);
    if vendor_dir.is_dir() {
        if let Some(l) = detect_license_from_file(&vendor_dir) {
            return (l, LicenseSource::LicenseFile);
        }
    }
    ("UNKNOWN".to_string(), LicenseSource::NotFound)
}

fn read_project_name(path: &Path) -> String {
    if let Ok(content) = fs::read_to_string(path) {
        let map = extract_top_level(&content);
        if let Some(name) = map.get("name").and_then(|v| v.as_str()) {
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    "unnamed".to_string()
}
