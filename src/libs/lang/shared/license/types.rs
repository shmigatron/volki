use std::collections::HashMap;
use std::fmt;
use std::io;

#[derive(Debug)]
pub struct ScanConfig {
    pub path: String,
    pub include_dev: bool,
    pub filter: Option<String>,
    pub exclude: Option<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub fn from_str(s: &str) -> Option<RiskLevel> {
        match s.to_lowercase().as_str() {
            "low" => Some(RiskLevel::Low),
            "medium" => Some(RiskLevel::Medium),
            "high" => Some(RiskLevel::High),
            _ => None,
        }
    }

    pub fn allows(self, category: LicenseCategory) -> bool {
        match self {
            RiskLevel::Low => category == LicenseCategory::Permissive,
            RiskLevel::Medium => matches!(
                category,
                LicenseCategory::Permissive | LicenseCategory::WeakCopyleft
            ),
            RiskLevel::High => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LicenseCategory {
    Permissive,
    WeakCopyleft,
    StrongCopyleft,
    Unknown,
}

impl LicenseCategory {
    pub fn from_license_str(license: &str) -> LicenseCategory {
        let license = license.trim();

        // Handle compound SPDX expressions like "(MIT OR Apache-2.0)"
        if license.contains(" OR ") || license.contains(" AND ") {
            let stripped = license.trim_start_matches('(').trim_end_matches(')');
            let parts: Vec<&str> = stripped
                .split(" OR ")
                .flat_map(|s| s.split(" AND "))
                .collect();

            let mut most_restrictive = LicenseCategory::Permissive;
            for part in parts {
                let cat = Self::classify_single(part.trim());
                if cat.restrictiveness() > most_restrictive.restrictiveness() {
                    most_restrictive = cat;
                }
            }
            return most_restrictive;
        }

        Self::classify_single(license)
    }

    fn classify_single(license: &str) -> LicenseCategory {
        let upper = license.to_uppercase();
        let upper = upper.trim();

        if upper.starts_with("MIT")
            || upper.starts_with("APACHE")
            || upper.starts_with("BSD")
            || upper.starts_with("ISC")
            || upper.starts_with("UNLICENSE")
            || upper.starts_with("CC0")
            || upper.starts_with("WTFPL")
            || upper.starts_with("0BSD")
            || upper.starts_with("ZLIB")
        {
            LicenseCategory::Permissive
        } else if upper.starts_with("LGPL") || upper.starts_with("MPL") {
            LicenseCategory::WeakCopyleft
        } else if upper.starts_with("GPL") || upper.starts_with("AGPL") {
            LicenseCategory::StrongCopyleft
        } else {
            LicenseCategory::Unknown
        }
    }

    fn restrictiveness(self) -> u8 {
        match self {
            LicenseCategory::Permissive => 0,
            LicenseCategory::WeakCopyleft => 1,
            LicenseCategory::StrongCopyleft => 2,
            LicenseCategory::Unknown => 3,
        }
    }
}

impl fmt::Display for LicenseCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LicenseCategory::Permissive => write!(f, "Permissive"),
            LicenseCategory::WeakCopyleft => write!(f, "Weak Copyleft"),
            LicenseCategory::StrongCopyleft => write!(f, "Strong Copyleft"),
            LicenseCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseSource {
    ManifestField,
    ManifestLegacy,
    LockfileField,
    MetadataFile,
    LicenseFile,
    NotFound,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PackageLicense {
    pub name: String,
    pub version: String,
    pub license: String,
    pub category: LicenseCategory,
    pub source: LicenseSource,
}

#[derive(Debug)]
pub struct ScanResult {
    pub project_name: String,
    pub total_packages: usize,
    pub packages: Vec<PackageLicense>,
    pub by_license: HashMap<String, Vec<String>>,
    pub by_category: HashMap<LicenseCategory, Vec<String>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum LicenseError {
    Io(io::Error),
    NoManifest(String),
    NoDependencyDir(String),
    ParseError(String),
}

impl fmt::Display for LicenseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LicenseError::Io(e) => write!(f, "IO error: {e}"),
            LicenseError::NoManifest(msg) => write!(f, "{msg}"),
            LicenseError::NoDependencyDir(msg) => write!(f, "{msg}"),
            LicenseError::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl From<io::Error> for LicenseError {
    fn from(e: io::Error) -> Self {
        LicenseError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- RiskLevel::from_str ---

    #[test]
    fn risk_level_from_str_low() {
        assert_eq!(RiskLevel::from_str("low"), Some(RiskLevel::Low));
    }

    #[test]
    fn risk_level_from_str_medium() {
        assert_eq!(RiskLevel::from_str("medium"), Some(RiskLevel::Medium));
    }

    #[test]
    fn risk_level_from_str_high() {
        assert_eq!(RiskLevel::from_str("high"), Some(RiskLevel::High));
    }

    #[test]
    fn risk_level_from_str_case_insensitive() {
        assert_eq!(RiskLevel::from_str("LOW"), Some(RiskLevel::Low));
        assert_eq!(RiskLevel::from_str("Medium"), Some(RiskLevel::Medium));
        assert_eq!(RiskLevel::from_str("HIGH"), Some(RiskLevel::High));
    }

    #[test]
    fn risk_level_from_str_invalid() {
        assert_eq!(RiskLevel::from_str("critical"), None);
    }

    #[test]
    fn risk_level_from_str_empty() {
        assert_eq!(RiskLevel::from_str(""), None);
    }

    // --- RiskLevel::allows ---

    #[test]
    fn risk_low_allows_permissive() {
        assert!(RiskLevel::Low.allows(LicenseCategory::Permissive));
    }

    #[test]
    fn risk_low_blocks_weak_copyleft() {
        assert!(!RiskLevel::Low.allows(LicenseCategory::WeakCopyleft));
    }

    #[test]
    fn risk_low_blocks_strong_copyleft() {
        assert!(!RiskLevel::Low.allows(LicenseCategory::StrongCopyleft));
    }

    #[test]
    fn risk_low_blocks_unknown() {
        assert!(!RiskLevel::Low.allows(LicenseCategory::Unknown));
    }

    #[test]
    fn risk_medium_allows_permissive() {
        assert!(RiskLevel::Medium.allows(LicenseCategory::Permissive));
    }

    #[test]
    fn risk_medium_allows_weak_copyleft() {
        assert!(RiskLevel::Medium.allows(LicenseCategory::WeakCopyleft));
    }

    #[test]
    fn risk_medium_blocks_strong_copyleft() {
        assert!(!RiskLevel::Medium.allows(LicenseCategory::StrongCopyleft));
    }

    #[test]
    fn risk_medium_blocks_unknown() {
        assert!(!RiskLevel::Medium.allows(LicenseCategory::Unknown));
    }

    #[test]
    fn risk_high_allows_all() {
        assert!(RiskLevel::High.allows(LicenseCategory::Permissive));
        assert!(RiskLevel::High.allows(LicenseCategory::WeakCopyleft));
        assert!(RiskLevel::High.allows(LicenseCategory::StrongCopyleft));
        assert!(RiskLevel::High.allows(LicenseCategory::Unknown));
    }

    // --- LicenseCategory::from_license_str single ---

    #[test]
    fn category_mit() {
        assert_eq!(LicenseCategory::from_license_str("MIT"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_apache() {
        assert_eq!(LicenseCategory::from_license_str("Apache-2.0"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_bsd_variants() {
        assert_eq!(LicenseCategory::from_license_str("BSD-2-Clause"), LicenseCategory::Permissive);
        assert_eq!(LicenseCategory::from_license_str("BSD-3-Clause"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_isc() {
        assert_eq!(LicenseCategory::from_license_str("ISC"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_unlicense() {
        assert_eq!(LicenseCategory::from_license_str("Unlicense"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_cc0() {
        assert_eq!(LicenseCategory::from_license_str("CC0-1.0"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_wtfpl() {
        assert_eq!(LicenseCategory::from_license_str("WTFPL"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_0bsd() {
        assert_eq!(LicenseCategory::from_license_str("0BSD"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_zlib() {
        assert_eq!(LicenseCategory::from_license_str("Zlib"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_lgpl() {
        assert_eq!(LicenseCategory::from_license_str("LGPL-2.1"), LicenseCategory::WeakCopyleft);
        assert_eq!(LicenseCategory::from_license_str("LGPL-3.0"), LicenseCategory::WeakCopyleft);
    }

    #[test]
    fn category_mpl() {
        assert_eq!(LicenseCategory::from_license_str("MPL-2.0"), LicenseCategory::WeakCopyleft);
    }

    #[test]
    fn category_gpl() {
        assert_eq!(LicenseCategory::from_license_str("GPL-2.0"), LicenseCategory::StrongCopyleft);
        assert_eq!(LicenseCategory::from_license_str("GPL-3.0"), LicenseCategory::StrongCopyleft);
    }

    #[test]
    fn category_agpl() {
        assert_eq!(LicenseCategory::from_license_str("AGPL-3.0"), LicenseCategory::StrongCopyleft);
    }

    #[test]
    fn category_unknown_license() {
        assert_eq!(LicenseCategory::from_license_str("CustomLicense"), LicenseCategory::Unknown);
    }

    #[test]
    fn category_empty() {
        assert_eq!(LicenseCategory::from_license_str(""), LicenseCategory::Unknown);
    }

    // --- Compound SPDX expressions ---

    #[test]
    fn category_or_both_permissive() {
        assert_eq!(LicenseCategory::from_license_str("MIT OR Apache-2.0"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_or_mixed_permissive_copyleft() {
        assert_eq!(LicenseCategory::from_license_str("MIT OR GPL-3.0"), LicenseCategory::StrongCopyleft);
    }

    #[test]
    fn category_and_mixed() {
        assert_eq!(LicenseCategory::from_license_str("MIT AND GPL-2.0"), LicenseCategory::StrongCopyleft);
    }

    #[test]
    fn category_or_with_parens() {
        assert_eq!(LicenseCategory::from_license_str("(MIT OR Apache-2.0)"), LicenseCategory::Permissive);
    }

    #[test]
    fn category_or_weak_copyleft_and_permissive() {
        assert_eq!(LicenseCategory::from_license_str("LGPL-2.1 OR MIT"), LicenseCategory::WeakCopyleft);
    }

    // --- Display impls ---

    #[test]
    fn category_display() {
        assert_eq!(format!("{}", LicenseCategory::Permissive), "Permissive");
        assert_eq!(format!("{}", LicenseCategory::WeakCopyleft), "Weak Copyleft");
        assert_eq!(format!("{}", LicenseCategory::StrongCopyleft), "Strong Copyleft");
        assert_eq!(format!("{}", LicenseCategory::Unknown), "Unknown");
    }

    // --- LicenseError ---

    #[test]
    fn license_error_display_io() {
        let err = LicenseError::Io(io::Error::new(io::ErrorKind::NotFound, "gone"));
        assert!(format!("{err}").contains("IO error"));
    }

    #[test]
    fn license_error_display_no_manifest() {
        let err = LicenseError::NoManifest("no pkg".to_string());
        assert_eq!(format!("{err}"), "no pkg");
    }

    #[test]
    fn license_error_display_parse() {
        let err = LicenseError::ParseError("bad".to_string());
        assert!(format!("{err}").contains("Parse error"));
    }

    #[test]
    fn license_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let err: LicenseError = io_err.into();
        assert!(matches!(err, LicenseError::Io(_)));
    }
}
