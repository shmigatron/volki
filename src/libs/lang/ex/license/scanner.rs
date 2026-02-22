use crate::core::volkiwithstds::collections::ToString;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::Path;

use crate::libs::lang::shared::license::heuristic::detect_license_from_file;
use crate::libs::lang::shared::license::parsers::key_value::parse_mix_lock_deps;
use crate::libs::lang::shared::license::scan_util::finalize_scan;
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);
    let mix_exs = root.join("mix.exs");

    if !mix_exs.exists() {
        return Err(LicenseError::NoManifest(crate::vstr!(
            "No mix.exs found in project directory"
        )));
    }

    let project_name = read_project_name(&mix_exs);
    let deps_dir = root.join("deps");

    // Try mix.lock first, fall back to scanning deps/
    let mix_lock = root.join("mix.lock");
    let dep_list = if mix_lock.exists() {
        let content = fs::read_to_string(&mix_lock)?;
        parse_mix_lock_deps(&content)
    } else if deps_dir.is_dir() {
        scan_deps_dir(&deps_dir)
    } else {
        return Err(LicenseError::NoDependencyDir(crate::vstr!(
            "No mix.lock or deps/ found (run mix deps.get first)"
        )));
    };

    let mut packages = Vec::new();

    for (name, version) in &dep_list {
        let (license, source) = find_elixir_dep_license(name, &deps_dir);
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

fn find_elixir_dep_license(name: &str, deps_dir: &Path) -> (String, LicenseSource) {
    let dep_dir = deps_dir.join(name);
    if !dep_dir.is_dir() {
        return (crate::vstr!("UNKNOWN"), LicenseSource::NotFound);
    }

    let mix_exs = dep_dir.join("mix.exs");
    if let Ok(content) = fs::read_to_string(&mix_exs) {
        if let Some(license) = extract_mix_exs_license(&content) {
            return (license, LicenseSource::ManifestField);
        }
    }

    if let Some(l) = detect_license_from_file(&dep_dir) {
        return (l, LicenseSource::LicenseFile);
    }

    (crate::vstr!("UNKNOWN"), LicenseSource::NotFound)
}

fn extract_mix_exs_license(content: &str) -> Option<String> {
    // Look for: licenses: ["MIT"] or licenses: ["Apache-2.0"]
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("licenses:") {
            if let Some(start) = trimmed.find('[') {
                if let Some(end) = trimmed[start..].find(']') {
                    let inside = &trimmed[start + 1..start + end];
                    let licenses: Vec<&str> = inside
                        .split(",")
                        .map(|s| s.trim().trim_matches('"').trim_matches('\''))
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !licenses.is_empty() {
                        return Some(licenses.join(" OR "));
                    }
                }
            }
        }
    }
    None
}

fn scan_deps_dir(deps_dir: &Path) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    if let Ok(entries) = fs::read_dir(deps_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_vstring();
                let version =
                    read_dep_version(&entry.path()).unwrap_or_else(|| crate::vstr!("0.0.0"));
                deps.push((name, version));
            }
        }
    }
    deps
}

fn read_dep_version(dep_dir: &Path) -> Option<String> {
    let mix_exs = dep_dir.join("mix.exs");
    let content = fs::read_to_string(&mix_exs).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("version:") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_vstring());
                }
            }
        }
    }
    None
}

fn read_project_name(mix_exs: &Path) -> String {
    if let Ok(content) = fs::read_to_string(mix_exs) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("app:") {
                if let Some(rest) = trimmed.split("app:").nth(1) {
                    let name = rest
                        .trim()
                        .trim_start_matches(':')
                        .trim_end_matches(',')
                        .trim();
                    if !name.is_empty() {
                        return name.to_vstring();
                    }
                }
            }
        }
    }
    crate::vstr!("unnamed")
}
