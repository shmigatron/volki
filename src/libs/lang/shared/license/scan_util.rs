use crate::core::volkiwithstds::collections::HashMap;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::path::PathBuf;

use super::types::{LicenseCategory, PackageLicense, RiskLevel, ScanConfig, ScanResult};

/// Apply filters, sort, and build grouped maps from a raw list of packages.
pub fn finalize_scan(
    project_name: String,
    mut packages: Vec<PackageLicense>,
    config: &ScanConfig,
) -> ScanResult {
    if let Some(ref filter) = config.filter {
        let filter_upper = filter.to_uppercase();
        packages.retain(|p: &PackageLicense| p.license.to_uppercase().contains(&filter_upper));
    }
    if let Some(ref exclude) = config.exclude {
        let exclude_upper = exclude.to_uppercase();
        packages.retain(|p: &PackageLicense| !p.license.to_uppercase().contains(&exclude_upper));
    }
    if config.risk_level != RiskLevel::High {
        packages.retain(|p| config.risk_level.allows(p.category));
    }

    packages.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let mut by_license: HashMap<String, Vec<String>> = HashMap::new();
    let mut by_category: HashMap<LicenseCategory, Vec<String>> = HashMap::new();

    for pkg in &packages {
        let label = crate::vformat!("{}@{}", pkg.name, pkg.version);
        by_license
            .entry(pkg.license.clone())
            .or_default()
            .push(label.clone());
        by_category.entry(pkg.category).or_default().push(label);
    }

    let total_packages = packages.len();

    ScanResult {
        project_name,
        total_packages,
        packages,
        by_license,
        by_category,
    }
}

/// Resolve the user's home directory from $HOME.
pub fn home_dir() -> Option<PathBuf> {
    crate::core::volkiwithstds::env::var("HOME").map(|s| PathBuf::from(s.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::collections::ToString;
    use crate::libs::lang::shared::license::types::{LicenseSource, PackageLicense};
    use crate::vvec;

    fn make_pkg(name: &str, version: &str, license: &str) -> PackageLicense {
        PackageLicense {
            name: name.to_vstring(),
            version: version.to_vstring(),
            license: license.to_vstring(),
            category: LicenseCategory::from_license_str(license),
            source: LicenseSource::ManifestField,
        }
    }

    fn default_config(filter: Option<&str>, exclude: Option<&str>, risk: RiskLevel) -> ScanConfig {
        ScanConfig {
            path: crate::vstr!("."),
            include_dev: false,
            filter: filter.map(|s| s.to_vstring()),
            exclude: exclude.map(|s| s.to_vstring()),
            risk_level: risk,
        }
    }

    // --- finalize_scan ---

    #[test]
    fn finalize_empty_packages() {
        let config = default_config(None, None, RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), vvec![], &config);
        assert_eq!(result.total_packages, 0);
        assert!(result.packages.is_empty());
        assert!(result.by_license.is_empty());
        assert!(result.by_category.is_empty());
    }

    #[test]
    fn finalize_alphabetical_sorting() {
        let pkgs = vvec![
            make_pkg("zlib", "1.0", "MIT"),
            make_pkg("alpha", "1.0", "MIT"),
            make_pkg("middle", "1.0", "MIT"),
        ];
        let config = default_config(None, None, RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.packages[0].name, "alpha");
        assert_eq!(result.packages[1].name, "middle");
        assert_eq!(result.packages[2].name, "zlib");
    }

    #[test]
    fn finalize_by_license_map() {
        let pkgs = vvec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "2.0", "MIT"),
            make_pkg("c", "1.0", "Apache-2.0"),
        ];
        let config = default_config(None, None, RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.by_license.get("MIT").unwrap().len(), 2);
        assert_eq!(result.by_license.get("Apache-2.0").unwrap().len(), 1);
    }

    #[test]
    fn finalize_by_category_map() {
        let pkgs = vvec![make_pkg("a", "1.0", "MIT"), make_pkg("b", "1.0", "GPL-3.0"),];
        let config = default_config(None, None, RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(
            result
                .by_category
                .get(&LicenseCategory::Permissive)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            result
                .by_category
                .get(&LicenseCategory::StrongCopyleft)
                .unwrap()
                .len(),
            1
        );
    }

    // --- Filter ---

    #[test]
    fn filter_retains_matching() {
        let pkgs = vvec![make_pkg("a", "1.0", "MIT"), make_pkg("b", "1.0", "GPL-3.0"),];
        let config = default_config(Some("MIT"), None, RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 1);
        assert_eq!(result.packages[0].license, "MIT");
    }

    #[test]
    fn filter_case_insensitive() {
        let pkgs = vvec![make_pkg("a", "1.0", "MIT")];
        let config = default_config(Some("mit"), None, RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 1);
    }

    #[test]
    fn filter_partial_match() {
        let pkgs = vvec![
            make_pkg("a", "1.0", "Apache-2.0"),
            make_pkg("b", "1.0", "MIT"),
        ];
        let config = default_config(Some("Apache"), None, RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 1);
    }

    // --- Exclude ---

    #[test]
    fn exclude_removes_matching() {
        let pkgs = vvec![make_pkg("a", "1.0", "MIT"), make_pkg("b", "1.0", "GPL-3.0"),];
        let config = default_config(None, Some("GPL"), RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 1);
        assert_eq!(result.packages[0].license, "MIT");
    }

    #[test]
    fn exclude_case_insensitive() {
        let pkgs = vvec![make_pkg("a", "1.0", "MIT"), make_pkg("b", "1.0", "GPL-3.0"),];
        let config = default_config(None, Some("gpl"), RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 1);
    }

    // --- Risk level ---

    #[test]
    fn risk_high_keeps_all() {
        let pkgs = vvec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "1.0", "GPL-3.0"),
            make_pkg("c", "1.0", "UNKNOWN"),
        ];
        let config = default_config(None, None, RiskLevel::High);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 3);
    }

    #[test]
    fn risk_low_only_permissive() {
        let pkgs = vvec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "1.0", "GPL-3.0"),
            make_pkg("c", "1.0", "LGPL-2.1"),
        ];
        let config = default_config(None, None, RiskLevel::Low);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 1);
        assert_eq!(result.packages[0].license, "MIT");
    }

    #[test]
    fn risk_medium_keeps_weak_copyleft() {
        let pkgs = vvec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "1.0", "LGPL-2.1"),
            make_pkg("c", "1.0", "GPL-3.0"),
        ];
        let config = default_config(None, None, RiskLevel::Medium);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 2);
    }

    #[test]
    fn risk_low_removes_unknown() {
        let pkgs = vvec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "1.0", "SomethingWeird"),
        ];
        let config = default_config(None, None, RiskLevel::Low);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 1);
    }

    // --- Combined ---

    #[test]
    fn combined_filter_and_risk() {
        let pkgs = vvec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "1.0", "Apache-2.0"),
            make_pkg("c", "1.0", "GPL-3.0"),
        ];
        let config = default_config(Some("MIT"), None, RiskLevel::Low);
        let result = finalize_scan(crate::vstr!("test"), pkgs, &config);
        assert_eq!(result.total_packages, 1);
        assert_eq!(result.packages[0].name, "a");
    }
}
