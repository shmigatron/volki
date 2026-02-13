use std::fs;
use std::path::Path;

use crate::libs::lang::shared::license::parsers::key_value::get_rfc822_field;
use crate::libs::lang::shared::license::scan_util::finalize_scan;
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);

    let has_manifest = root.join("pyproject.toml").exists()
        || root.join("Pipfile").exists()
        || root.join("requirements.txt").exists()
        || root.join("setup.py").exists();

    if !has_manifest {
        return Err(LicenseError::NoManifest(
            "No Python project file found (pyproject.toml, Pipfile, requirements.txt, or setup.py)".to_string(),
        ));
    }

    let venv_dir = find_venv(root).ok_or_else(|| {
        LicenseError::NoDependencyDir(
            "No virtual environment found (.venv/, venv/, or env/). Create one and install dependencies first.".to_string(),
        )
    })?;

    let project_name = read_project_name(root);

    let mut packages = Vec::new();
    scan_site_packages(&venv_dir, &mut packages);

    Ok(finalize_scan(project_name, packages, config))
}

fn find_venv(root: &Path) -> Option<std::path::PathBuf> {
    let candidates = [".venv", "venv", "env"];
    for name in &candidates {
        let path = root.join(name);
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

fn scan_site_packages(venv_dir: &Path, packages: &mut Vec<PackageLicense>) {
    let lib_dir = venv_dir.join("lib");
    let Ok(entries) = fs::read_dir(&lib_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("python") {
            let site_packages = entry.path().join("site-packages");
            if site_packages.is_dir() {
                scan_dist_infos(&site_packages, packages);
            }
        }
    }
}

fn scan_dist_infos(site_packages: &Path, packages: &mut Vec<PackageLicense>) {
    let Ok(entries) = fs::read_dir(site_packages) else {
        return;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.ends_with(".dist-info") && entry.path().is_dir() {
            if let Some(pkg) = read_dist_info(&entry.path(), &name_str) {
                packages.push(pkg);
            }
        }
    }
}

fn read_dist_info(dir: &Path, dir_name: &str) -> Option<PackageLicense> {
    let base = dir_name.strip_suffix(".dist-info")?;
    let (name, version) = base.rsplit_once('-')?;

    let skip_packages = ["pip", "setuptools", "wheel", "pkg_resources", "_distutils_hack"];
    if skip_packages.contains(&name) {
        return None;
    }

    let name = name.replace('_', "-");

    let metadata_path = dir.join("METADATA");
    let (license, source) = if let Ok(content) = fs::read_to_string(&metadata_path) {
        if let Some(lic) = get_rfc822_field(&content, "License") {
            (lic, LicenseSource::MetadataFile)
        } else if let Some(classifier) = find_license_classifier(&content) {
            (classifier, LicenseSource::MetadataFile)
        } else {
            ("UNKNOWN".to_string(), LicenseSource::NotFound)
        }
    } else {
        ("UNKNOWN".to_string(), LicenseSource::NotFound)
    };

    let category = LicenseCategory::from_license_str(&license);

    Some(PackageLicense {
        name,
        version: version.to_string(),
        license,
        category,
        source,
    })
}

fn find_license_classifier(metadata: &str) -> Option<String> {
    for line in metadata.lines() {
        if let Some(rest) = line.strip_prefix("Classifier: License :: OSI Approved :: ") {
            let license = rest.trim().trim_end_matches(" License").to_string();
            if !license.is_empty() {
                return Some(license);
            }
        }
    }
    None
}

fn read_project_name(root: &Path) -> String {
    let pyproject = root.join("pyproject.toml");
    if let Ok(content) = fs::read_to_string(&pyproject) {
        let mut in_project = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[project]" {
                in_project = true;
                continue;
            }
            if trimmed.starts_with('[') {
                in_project = false;
                continue;
            }
            if in_project {
                if let Some(rest) = trimmed.strip_prefix("name") {
                    let rest = rest.trim();
                    if let Some(rest) = rest.strip_prefix('=') {
                        let val = rest.trim().trim_matches('"').trim_matches('\'');
                        if !val.is_empty() {
                            return val.to_string();
                        }
                    }
                }
            }
        }
    }
    "unnamed".to_string()
}
