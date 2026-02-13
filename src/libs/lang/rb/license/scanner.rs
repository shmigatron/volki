use std::fs;
use std::path::Path;

use crate::libs::lang::shared::license::heuristic::detect_license_from_file;
use crate::libs::lang::shared::license::parsers::key_value::parse_gemfile_lock_gems;
use crate::libs::lang::shared::license::scan_util::finalize_scan;
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);
    let gemfile = root.join("Gemfile");
    let gemfile_lock = root.join("Gemfile.lock");

    if !gemfile.exists() {
        return Err(LicenseError::NoManifest(
            "No Gemfile found in project directory".to_string(),
        ));
    }
    if !gemfile_lock.exists() {
        return Err(LicenseError::NoDependencyDir(
            "No Gemfile.lock found (run bundle install first)".to_string(),
        ));
    }

    let project_name = read_project_name(root);

    let lock_content = fs::read_to_string(&gemfile_lock)?;
    let gems = parse_gemfile_lock_gems(&lock_content);

    let vendor_bundle = root.join("vendor").join("bundle");
    let gem_home = std::env::var("GEM_HOME").ok().map(std::path::PathBuf::from);

    let mut packages = Vec::new();

    for (name, version) in &gems {
        let (license, source) = find_gem_license(name, version, &vendor_bundle, &gem_home);
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

fn find_gem_license(
    name: &str,
    version: &str,
    vendor_bundle: &Path,
    gem_home: &Option<std::path::PathBuf>,
) -> (String, LicenseSource) {
    let gem_dir_name = format!("{name}-{version}");

    if vendor_bundle.is_dir() {
        if let Some(result) = search_gem_in_bundle(vendor_bundle, &gem_dir_name) {
            return result;
        }
    }

    if let Some(home) = gem_home {
        let gems_dir = home.join("gems").join(&gem_dir_name);
        if gems_dir.is_dir() {
            if let Some(l) = read_gemspec_license(home, name, version) {
                return (l, LicenseSource::MetadataFile);
            }
            if let Some(l) = detect_license_from_file(&gems_dir) {
                return (l, LicenseSource::LicenseFile);
            }
        }
    }

    ("UNKNOWN".to_string(), LicenseSource::NotFound)
}

fn search_gem_in_bundle(vendor_bundle: &Path, gem_dir_name: &str) -> Option<(String, LicenseSource)> {
    // vendor/bundle may contain ruby/VERSION/gems/
    let Ok(entries) = fs::read_dir(vendor_bundle) else {
        return None;
    };

    for entry in entries.flatten() {
        let ruby_dir = entry.path();
        if !ruby_dir.is_dir() {
            continue;
        }

        let Ok(version_entries) = fs::read_dir(&ruby_dir) else {
            continue;
        };

        for ver_entry in version_entries.flatten() {
            let gems_dir = ver_entry.path().join("gems").join(gem_dir_name);
            if gems_dir.is_dir() {
                if let Some(l) = detect_license_from_file(&gems_dir) {
                    return Some((l, LicenseSource::LicenseFile));
                }
            }
        }
    }

    None
}

fn read_gemspec_license(gem_home: &Path, name: &str, version: &str) -> Option<String> {
    let spec_path = gem_home
        .join("specifications")
        .join(format!("{name}-{version}.gemspec"));

    let content = fs::read_to_string(&spec_path).ok()?;

    // Look for s.license = "MIT" or s.licenses = ["MIT"]
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains(".license") && trimmed.contains('=') {
            if let Some(val) = extract_ruby_string(trimmed) {
                return Some(val);
            }
        }
    }

    None
}

fn extract_ruby_string(line: &str) -> Option<String> {
    // Match "value" or 'value' after =
    let after_eq = line.split('=').nth(1)?.trim();
    let trimmed = after_eq
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();

    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        Some(trimmed[1..trimmed.len() - 1].to_string())
    } else {
        None
    }
}

fn read_project_name(root: &Path) -> String {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".gemspec") {
                return name_str.trim_end_matches(".gemspec").to_string();
            }
        }
    }

    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string())
}
