use std::fs;
use std::path::Path;

use crate::libs::lang::shared::license::heuristic::detect_license_from_file;
use crate::libs::lang::shared::license::parsers::key_value::parse_go_mod_requires;
use crate::libs::lang::shared::license::scan_util::{finalize_scan, home_dir};
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);
    let go_mod = root.join("go.mod");

    if !go_mod.exists() {
        return Err(LicenseError::NoManifest(
            "No go.mod found in project directory".to_string(),
        ));
    }

    let mod_content = fs::read_to_string(&go_mod)?;

    let project_name = mod_content
        .lines()
        .next()
        .and_then(|l| l.strip_prefix("module "))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let deps = parse_go_mod_requires(&mod_content);

    // Find GOPATH module cache
    let gopath = std::env::var("GOPATH")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| home_dir().map(|h| h.join("go")));

    let mod_cache = gopath.map(|g| g.join("pkg").join("mod"));

    let mut packages = Vec::new();

    for (module_path, version) in &deps {
        let (license, source) = find_go_module_license(module_path, version, &mod_cache);
        let category = LicenseCategory::from_license_str(&license);

        packages.push(PackageLicense {
            name: module_path.clone(),
            version: version.clone(),
            license,
            category,
            source,
        });
    }

    Ok(finalize_scan(project_name, packages, config))
}

fn find_go_module_license(
    module_path: &str,
    version: &str,
    mod_cache: &Option<std::path::PathBuf>,
) -> (String, LicenseSource) {
    let Some(cache) = mod_cache else {
        return ("UNKNOWN".to_string(), LicenseSource::NotFound);
    };

    if !cache.exists() {
        return ("UNKNOWN".to_string(), LicenseSource::NotFound);
    }

    // Go module cache uses bang-encoding for uppercase letters
    let encoded_path = encode_module_path(module_path);
    let dir_name = format!("{encoded_path}@{version}");
    let full_path = cache.join(&dir_name);

    if full_path.is_dir() {
        if let Some(l) = detect_license_from_file(&full_path) {
            return (l, LicenseSource::LicenseFile);
        }
    }

    let parts: Vec<&str> = encoded_path.split('/').collect();
    for i in (1..parts.len()).rev() {
        let parent_path = parts[..i].join("/");
        let parent_dir = cache.join(format!("{parent_path}@{version}"));
        if parent_dir.is_dir() {
            if let Some(l) = detect_license_from_file(&parent_dir) {
                return (l, LicenseSource::LicenseFile);
            }
        }
    }

    ("UNKNOWN".to_string(), LicenseSource::NotFound)
}

/// Encode uppercase characters in module paths for Go module cache.
/// Uppercase letters are replaced with `!` followed by the lowercase letter.
fn encode_module_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for ch in path.chars() {
        if ch.is_ascii_uppercase() {
            encoded.push('!');
            encoded.push(ch.to_ascii_lowercase());
        } else {
            encoded.push(ch);
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_all_lowercase() {
        assert_eq!(encode_module_path("github.com/pkg/errors"), "github.com/pkg/errors");
    }

    #[test]
    fn encode_with_uppercase() {
        assert_eq!(encode_module_path("github.com/Azure/azure-sdk"), "github.com/!azure/azure-sdk");
    }

    #[test]
    fn encode_multiple_uppercase() {
        assert_eq!(encode_module_path("github.com/BurntSushi/toml"), "github.com/!burnt!sushi/toml");
    }

    #[test]
    fn encode_empty() {
        assert_eq!(encode_module_path(""), "");
    }
}
