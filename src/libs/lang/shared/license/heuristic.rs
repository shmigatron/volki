use std::fs;
use std::path::Path;

/// Detect license type from LICENSE file variants in a directory.
/// Reads the first 1000 characters and uses heuristic keyword matching.
pub fn detect_license_from_file(dir: &Path) -> Option<String> {
    let candidates = [
        "LICENSE",
        "LICENSE.md",
        "LICENSE.txt",
        "LICENCE",
        "LICENCE.md",
        "LICENCE.txt",
        "license",
        "license.md",
        "license.txt",
    ];

    for candidate in &candidates {
        let path = dir.join(candidate);
        if let Ok(content) = fs::read_to_string(&path) {
            // Only look at the first 1000 chars for heuristic matching
            let snippet: String = content.chars().take(1000).collect();
            let upper = snippet.to_uppercase();

            if upper.contains("MIT LICENSE") || upper.contains("PERMISSION IS HEREBY GRANTED") {
                return Some("MIT".to_string());
            }
            if upper.contains("APACHE LICENSE") {
                return Some("Apache-2.0".to_string());
            }
            if upper.contains("BSD 2-CLAUSE") || upper.contains("SIMPLIFIED BSD") {
                return Some("BSD-2-Clause".to_string());
            }
            if upper.contains("BSD 3-CLAUSE") || upper.contains("NEW BSD") {
                return Some("BSD-3-Clause".to_string());
            }
            if upper.contains("ISC LICENSE") {
                return Some("ISC".to_string());
            }
            if upper.contains("GNU GENERAL PUBLIC LICENSE") {
                if upper.contains("VERSION 3") {
                    return Some("GPL-3.0".to_string());
                }
                return Some("GPL-2.0".to_string());
            }
            if upper.contains("GNU LESSER GENERAL PUBLIC") {
                return Some("LGPL-2.1".to_string());
            }
            if upper.contains("MOZILLA PUBLIC LICENSE") {
                return Some("MPL-2.0".to_string());
            }
            if upper.contains("THE UNLICENSE") || upper.contains("UNLICENSE") {
                return Some("Unlicense".to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_dir_with_license(filename: &str, content: &str) -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("volki_test_heuristic_{}_{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join(filename), content).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detect_mit() {
        let dir = temp_dir_with_license("LICENSE", "MIT License\n\nCopyright (c) 2024\n\nPermission is hereby granted...");
        assert_eq!(detect_license_from_file(&dir), Some("MIT".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_apache() {
        let dir = temp_dir_with_license("LICENSE", "Apache License\nVersion 2.0, January 2004\n...");
        assert_eq!(detect_license_from_file(&dir), Some("Apache-2.0".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_bsd2() {
        let dir = temp_dir_with_license("LICENSE", "BSD 2-Clause License\n\nRedistribution and use...");
        assert_eq!(detect_license_from_file(&dir), Some("BSD-2-Clause".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_bsd3() {
        let dir = temp_dir_with_license("LICENSE", "BSD 3-Clause License\n\nRedistribution and use...");
        assert_eq!(detect_license_from_file(&dir), Some("BSD-3-Clause".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_isc() {
        let dir = temp_dir_with_license("LICENSE", "ISC License\n\nCopyright (c) 2024...");
        assert_eq!(detect_license_from_file(&dir), Some("ISC".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_gpl2() {
        let dir = temp_dir_with_license("LICENSE", "GNU General Public License\nVersion 2, June 1991...");
        assert_eq!(detect_license_from_file(&dir), Some("GPL-2.0".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_gpl3() {
        let dir = temp_dir_with_license("LICENSE", "GNU General Public License\nVersion 3, 29 June 2007...");
        assert_eq!(detect_license_from_file(&dir), Some("GPL-3.0".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_lgpl() {
        let dir = temp_dir_with_license("LICENSE", "GNU Lesser General Public License\nVersion 2.1...");
        assert_eq!(detect_license_from_file(&dir), Some("LGPL-2.1".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_mpl() {
        let dir = temp_dir_with_license("LICENSE", "Mozilla Public License Version 2.0\n...");
        assert_eq!(detect_license_from_file(&dir), Some("MPL-2.0".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_unlicense() {
        let dir = temp_dir_with_license("LICENSE", "This is free and unencumbered software released into the public domain.\n\nThe Unlicense...");
        assert_eq!(detect_license_from_file(&dir), Some("Unlicense".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_no_license_file() {
        let dir = std::env::temp_dir().join(format!("volki_test_heuristic_none_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        assert_eq!(detect_license_from_file(&dir), None);
        cleanup(&dir);
    }

    #[test]
    fn detect_licence_spelling() {
        let dir = temp_dir_with_license("LICENCE", "MIT License\n\nPermission is hereby granted...");
        assert_eq!(detect_license_from_file(&dir), Some("MIT".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_license_md() {
        let dir = temp_dir_with_license("LICENSE.md", "MIT License\n\nPermission is hereby granted...");
        assert_eq!(detect_license_from_file(&dir), Some("MIT".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_license_txt() {
        let dir = temp_dir_with_license("LICENSE.txt", "MIT License\n\nPermission is hereby granted...");
        assert_eq!(detect_license_from_file(&dir), Some("MIT".to_string()));
        cleanup(&dir);
    }

    #[test]
    fn detect_unrecognized_content() {
        let dir = temp_dir_with_license("LICENSE", "Some random proprietary license text that doesn't match anything.");
        assert_eq!(detect_license_from_file(&dir), None);
        cleanup(&dir);
    }

    #[test]
    fn detect_permission_hereby_granted_without_mit() {
        let dir = temp_dir_with_license("LICENSE", "Permission is hereby granted, free of charge, to any person...");
        assert_eq!(detect_license_from_file(&dir), Some("MIT".to_string()));
        cleanup(&dir);
    }
}
