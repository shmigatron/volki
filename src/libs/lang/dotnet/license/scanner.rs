use std::fs;
use std::path::Path;

use crate::libs::lang::shared::license::parsers::xml_extract::{
    parse_csproj_package_references, parse_nuspec_license,
};
use crate::libs::lang::shared::license::scan_util::{finalize_scan, home_dir};
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);

    let csproj_files = find_csproj_files(root);
    if csproj_files.is_empty() {
        return Err(LicenseError::NoManifest(
            "No .csproj file found in project directory".to_string(),
        ));
    }

    let project_name = csproj_files
        .first()
        .and_then(|p| p.file_stem())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    // NuGet cache: ~/.nuget/packages/
    let nuget_cache = home_dir().map(|h| h.join(".nuget").join("packages"));

    let mut packages = Vec::new();

    for csproj in &csproj_files {
        if let Ok(content) = fs::read_to_string(csproj) {
            let refs = parse_csproj_package_references(&content);
            for (name, version) in refs {
                let (license, source) = find_nuget_license(&name, &version, &nuget_cache);
                let category = LicenseCategory::from_license_str(&license);

                packages.push(PackageLicense {
                    name,
                    version,
                    license,
                    category,
                    source,
                });
            }
        }
    }

    packages.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    packages.dedup_by(|a, b| a.name.to_lowercase() == b.name.to_lowercase());

    Ok(finalize_scan(project_name, packages, config))
}

fn find_csproj_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "csproj" {
                    files.push(path);
                }
            }
        }
    }
    files
}

fn find_nuget_license(
    name: &str,
    version: &str,
    nuget_cache: &Option<std::path::PathBuf>,
) -> (String, LicenseSource) {
    let Some(cache) = nuget_cache else {
        return ("UNKNOWN".to_string(), LicenseSource::NotFound);
    };

    if !cache.exists() {
        return ("UNKNOWN".to_string(), LicenseSource::NotFound);
    }

    // NuGet cache uses lowercase package names
    let lower_name = name.to_lowercase();
    let pkg_dir = cache.join(&lower_name).join(version);

    if !pkg_dir.is_dir() {
        return ("UNKNOWN".to_string(), LicenseSource::NotFound);
    }

    let nuspec_path = pkg_dir.join(format!("{lower_name}.nuspec"));
    if let Ok(content) = fs::read_to_string(&nuspec_path) {
        if let Some(license) = parse_nuspec_license(&content) {
            return (license, LicenseSource::MetadataFile);
        }
    }

    // Try alternate nuspec location (inside the nupkg extracted dir)
    if let Ok(entries) = fs::read_dir(&pkg_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "nuspec" {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Some(license) = parse_nuspec_license(&content) {
                            return (license, LicenseSource::MetadataFile);
                        }
                    }
                }
            }
        }
    }

    ("UNKNOWN".to_string(), LicenseSource::NotFound)
}
