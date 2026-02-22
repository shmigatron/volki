use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::Path;

use crate::libs::lang::shared::license::heuristic::detect_license_from_file;
use crate::libs::lang::shared::license::parsers::toml_simple::{
    extract_toml_string_value, parse_cargo_lock_packages,
};
use crate::libs::lang::shared::license::scan_util::{finalize_scan, home_dir};
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};
use crate::{vformat, vstr};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);
    let cargo_toml = root.join("Cargo.toml");
    let cargo_lock = root.join("Cargo.lock");

    if !cargo_toml.exists() {
        return Err(LicenseError::NoManifest(vstr!(
            "No Cargo.toml found in project directory"
        )));
    }
    if !cargo_lock.exists() {
        return Err(LicenseError::NoDependencyDir(vstr!(
            "No Cargo.lock found (run cargo build first)"
        )));
    }

    let toml_content = fs::read_to_string(&cargo_toml)?;
    let project_name =
        extract_toml_string_value(&toml_content, "name").unwrap_or_else(|| crate::vstr!("unnamed"));

    let lock_content = fs::read_to_string(&cargo_lock)?;
    let lock_packages = parse_cargo_lock_packages(&lock_content);

    // Find cargo registry cache
    let registry_base = home_dir().map(|h| h.join(".cargo").join("registry").join("src"));

    let mut packages = Vec::new();

    for (name, version) in &lock_packages {
        if name == &project_name {
            continue;
        }

        let (license, source) = find_crate_license(name, version, &registry_base);
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

fn find_crate_license(
    name: &str,
    version: &str,
    registry_base: &Option<crate::core::volkiwithstds::path::PathBuf>,
) -> (String, LicenseSource) {
    let Some(base) = registry_base else {
        return (vstr!("UNKNOWN"), LicenseSource::NotFound);
    };

    if !base.exists() {
        return (vstr!("UNKNOWN"), LicenseSource::NotFound);
    }

    // Registry src contains directories like "index.crates.io-*"
    let Ok(entries) = fs::read_dir(base) else {
        return (vstr!("UNKNOWN"), LicenseSource::NotFound);
    };

    for entry in entries.flatten() {
        let crate_dir = entry.path().join(&vformat!("{name}-{version}"));
        if crate_dir.is_dir() {
            let toml_path = crate_dir.join("Cargo.toml");
            if let Ok(content) = fs::read_to_string(&toml_path) {
                if let Some(license) = extract_toml_string_value(&content, "license") {
                    return (license, LicenseSource::ManifestField);
                }
            }

            if let Some(l) = detect_license_from_file(&crate_dir) {
                return (l, LicenseSource::LicenseFile);
            }
        }
    }

    (crate::vstr!("UNKNOWN"), LicenseSource::NotFound)
}
