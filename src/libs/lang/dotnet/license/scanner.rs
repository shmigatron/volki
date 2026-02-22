use crate::core::volkiwithstds::collections::ToString;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::Path;

use crate::libs::lang::shared::license::scan_util::{finalize_scan, home_dir};
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};
use crate::libs::lang::shared::license::xml::{
    parse_csproj_package_references, parse_nuspec_license,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);

    let csproj_files = find_csproj_files(root);
    if csproj_files.is_empty() {
        return Err(LicenseError::NoManifest(crate::vstr!(
            "No .csproj file found in project directory"
        )));
    }

    let project_name = csproj_files
        .first()
        .and_then(|p| p.file_stem())
        .map(|s| s.to_vstring())
        .unwrap_or_else(|| crate::vstr!("unnamed"));

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

fn find_csproj_files(root: &Path) -> Vec<crate::core::volkiwithstds::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "csproj" {
                    files.push(path.to_path_buf());
                }
            }
        }
    }
    files
}

fn find_nuget_license(
    name: &str,
    version: &str,
    nuget_cache: &Option<crate::core::volkiwithstds::path::PathBuf>,
) -> (String, LicenseSource) {
    let Some(cache) = nuget_cache else {
        return (crate::vstr!("UNKNOWN"), LicenseSource::NotFound);
    };

    if !cache.exists() {
        return (crate::vstr!("UNKNOWN"), LicenseSource::NotFound);
    }

    // NuGet cache uses lowercase package names
    let lower_name = name.to_lowercase();
    let pkg_dir = cache.join(&lower_name).join(version);

    if !pkg_dir.is_dir() {
        return (crate::vstr!("UNKNOWN"), LicenseSource::NotFound);
    }

    let nuspec_path = pkg_dir.join(&crate::vformat!("{lower_name}.nuspec"));
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

    (crate::vstr!("UNKNOWN"), LicenseSource::NotFound)
}
