use std::fs;
use std::path::Path;

use crate::libs::lang::shared::license::heuristic::detect_license_from_file;
use crate::libs::lang::shared::license::parsers::key_value::parse_pubspec_lock_packages;
use crate::libs::lang::shared::license::scan_util::{finalize_scan, home_dir};
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);
    let pubspec_yaml = root.join("pubspec.yaml");
    let pubspec_lock = root.join("pubspec.lock");

    if !pubspec_yaml.exists() {
        return Err(LicenseError::NoManifest(
            "No pubspec.yaml found in project directory".to_string(),
        ));
    }
    if !pubspec_lock.exists() {
        return Err(LicenseError::NoDependencyDir(
            "No pubspec.lock found (run dart pub get first)".to_string(),
        ));
    }

    let project_name = read_project_name(&pubspec_yaml);

    let lock_content = fs::read_to_string(&pubspec_lock)?;
    let deps = parse_pubspec_lock_packages(&lock_content);

    // Pub cache: ~/.pub-cache/hosted/pub.dev/
    let pub_cache = home_dir().map(|h| h.join(".pub-cache").join("hosted").join("pub.dev"));

    let mut packages = Vec::new();

    for (name, version) in &deps {
        let (license, source) = find_dart_package_license(name, version, &pub_cache);
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

fn find_dart_package_license(
    name: &str,
    version: &str,
    pub_cache: &Option<std::path::PathBuf>,
) -> (String, LicenseSource) {
    let Some(cache) = pub_cache else {
        return ("UNKNOWN".to_string(), LicenseSource::NotFound);
    };

    let pkg_dir = cache.join(format!("{name}-{version}"));
    if pkg_dir.is_dir() {
        if let Some(l) = detect_license_from_file(&pkg_dir) {
            return (l, LicenseSource::LicenseFile);
        }
    }

    ("UNKNOWN".to_string(), LicenseSource::NotFound)
}

fn read_project_name(pubspec_yaml: &Path) -> String {
    if let Ok(content) = fs::read_to_string(pubspec_yaml) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("name:") {
                let val = rest.trim().trim_matches('"').trim_matches('\'');
                if !val.is_empty() {
                    return val.to_string();
                }
            }
        }
    }
    "unnamed".to_string()
}
