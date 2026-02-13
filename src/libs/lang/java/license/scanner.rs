use std::fs;
use std::path::{Path, PathBuf};

use crate::libs::lang::shared::license::parsers::xml_extract::{
    parse_maven_dependencies, parse_pom_license,
};
use crate::libs::lang::shared::license::scan_util::{finalize_scan, home_dir};
use crate::libs::lang::shared::license::types::{
    LicenseCategory, LicenseError, LicenseSource, PackageLicense, ScanConfig, ScanResult,
};

pub fn scan(config: &ScanConfig) -> Result<ScanResult, LicenseError> {
    let root = Path::new(&config.path);

    let is_maven = root.join("pom.xml").exists();
    let is_gradle = root.join("build.gradle").exists() || root.join("build.gradle.kts").exists();

    if !is_maven && !is_gradle {
        return Err(LicenseError::NoManifest(
            "No pom.xml or build.gradle found in project directory".to_string(),
        ));
    }

    let m2_repo = home_dir().map(|h| h.join(".m2").join("repository"));
    let gradle_cache = home_dir()
        .map(|h| h.join(".gradle").join("caches").join("modules-2").join("files-2.1"));

    if is_maven {
        scan_maven(root, config, &m2_repo, &gradle_cache)
    } else {
        scan_gradle(root, config, &m2_repo, &gradle_cache)
    }
}

fn scan_maven(
    root: &Path,
    config: &ScanConfig,
    m2_repo: &Option<PathBuf>,
    gradle_cache: &Option<PathBuf>,
) -> Result<ScanResult, LicenseError> {
    let pom_path = root.join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)?;

    let project_name = extract_maven_project_name(&pom_content);
    let deps = parse_maven_dependencies(&pom_content);

    let mut packages = Vec::new();

    for (group_id, artifact_id, version) in &deps {
        let (license, source, resolved_version) =
            find_java_license(group_id, artifact_id, version, m2_repo, gradle_cache);
        let category = LicenseCategory::from_license_str(&license);

        packages.push(PackageLicense {
            name: format!("{group_id}:{artifact_id}"),
            version: resolved_version,
            license,
            category,
            source,
        });
    }

    Ok(finalize_scan(project_name, packages, config))
}

fn scan_gradle(
    root: &Path,
    config: &ScanConfig,
    m2_repo: &Option<PathBuf>,
    gradle_cache: &Option<PathBuf>,
) -> Result<ScanResult, LicenseError> {
    let gradle_path = if root.join("build.gradle.kts").exists() {
        root.join("build.gradle.kts")
    } else {
        root.join("build.gradle")
    };

    let content = fs::read_to_string(&gradle_path)?;
    let project_name = read_gradle_project_name(root);
    let deps = parse_gradle_dependencies(&content, config.include_dev);

    let mut packages = Vec::new();

    for (group_id, artifact_id, version) in &deps {
        let (license, source, resolved_version) =
            find_java_license(group_id, artifact_id, version, m2_repo, gradle_cache);
        let category = LicenseCategory::from_license_str(&license);

        packages.push(PackageLicense {
            name: format!("{group_id}:{artifact_id}"),
            version: resolved_version,
            license,
            category,
            source,
        });
    }

    Ok(finalize_scan(project_name, packages, config))
}

/// Search for a POM file containing license info across both caches.
/// Returns (license, source, resolved_version).
fn find_java_license(
    group_id: &str,
    artifact_id: &str,
    version: &str,
    m2_repo: &Option<PathBuf>,
    gradle_cache: &Option<PathBuf>,
) -> (String, LicenseSource, String) {
    if let Some(result) = find_in_gradle_cache(group_id, artifact_id, version, gradle_cache) {
        return result;
    }

    if let Some(result) = find_in_m2_repo(group_id, artifact_id, version, m2_repo) {
        return result;
    }

    (
        "UNKNOWN".to_string(),
        LicenseSource::NotFound,
        version.to_string(),
    )
}

/// Search ~/.gradle/caches/modules-2/files-2.1/group/artifact/version/hash/*.pom
fn find_in_gradle_cache(
    group_id: &str,
    artifact_id: &str,
    version: &str,
    gradle_cache: &Option<PathBuf>,
) -> Option<(String, LicenseSource, String)> {
    let cache = gradle_cache.as_ref()?;
    if !cache.exists() {
        return None;
    }

    let artifact_dir = cache.join(group_id).join(artifact_id);
    if !artifact_dir.is_dir() {
        return None;
    }

    // If we have a version, look directly; otherwise pick the latest available
    let version_dir = if !version.is_empty() {
        let dir = artifact_dir.join(version);
        if dir.is_dir() {
            dir
        } else {
            return None;
        }
    } else {
        pick_latest_version_dir(&artifact_dir)?
    };

    let resolved_version = version_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if let Ok(hash_entries) = fs::read_dir(&version_dir) {
        for hash_entry in hash_entries.flatten() {
            let hash_dir = hash_entry.path();
            if !hash_dir.is_dir() {
                continue;
            }
            if let Ok(files) = fs::read_dir(&hash_dir) {
                for file in files.flatten() {
                    let path = file.path();
                    if path.extension().is_some_and(|e| e == "pom") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Some(license) = parse_pom_license(&content) {
                                return Some((license, LicenseSource::MetadataFile, resolved_version));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Search ~/.m2/repository/group/path/artifact/version/artifact-version.pom
fn find_in_m2_repo(
    group_id: &str,
    artifact_id: &str,
    version: &str,
    m2_repo: &Option<PathBuf>,
) -> Option<(String, LicenseSource, String)> {
    let repo = m2_repo.as_ref()?;
    if !repo.exists() {
        return None;
    }

    let group_path = group_id.replace('.', "/");
    let artifact_dir = repo.join(&group_path).join(artifact_id);

    if !artifact_dir.is_dir() {
        return None;
    }

    let version_str;
    if !version.is_empty() {
        version_str = version.to_string();
    } else {
        let dir = pick_latest_version_dir(&artifact_dir);
        match dir {
            Some(d) => {
                version_str = d
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
            }
            None => return None,
        }
    }

    let pom_path = artifact_dir
        .join(&version_str)
        .join(format!("{artifact_id}-{version_str}.pom"));

    if let Ok(content) = fs::read_to_string(&pom_path) {
        if let Some(license) = parse_pom_license(&content) {
            return Some((license, LicenseSource::MetadataFile, version_str));
        }
    }

    None
}

/// Given a directory containing version subdirectories, pick the "latest".
/// Uses simple lexicographic sorting which works well for semver.
fn pick_latest_version_dir(artifact_dir: &Path) -> Option<PathBuf> {
    let Ok(entries) = fs::read_dir(artifact_dir) else {
        return None;
    };

    let mut versions: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect();

    if versions.is_empty() {
        return None;
    }

    versions.sort();
    versions.pop()
}

fn extract_maven_project_name(pom_content: &str) -> String {
    use crate::libs::lang::shared::license::parsers::xml_extract::extract_tag_contents;

    let names = extract_tag_contents(pom_content, "name");
    if let Some(name) = names.first() {
        if !name.is_empty() && !name.contains("${") {
            return name.clone();
        }
    }

    let artifacts = extract_tag_contents(pom_content, "artifactId");
    if let Some(id) = artifacts.first() {
        return id.clone();
    }

    "unnamed".to_string()
}

/// Parse Gradle dependency declarations from build.gradle or build.gradle.kts.
///
/// Handles:
/// - Groovy DSL:  `implementation 'group:artifact:version'`
/// - Kotlin DSL:  `implementation("group:artifact:version")`
/// - Deps without version: `implementation 'group:artifact'` (BOM-managed)
/// - Test configurations filtered by `include_dev`
fn parse_gradle_dependencies(content: &str, include_dev: bool) -> Vec<(String, String, String)> {
    let mut deps = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        let is_test = trimmed.starts_with("test")
            || trimmed.starts_with("androidTest");

        if is_test && !include_dev {
            continue;
        }

        let dep_prefixes = [
            "implementation",
            "api",
            "compile",
            "runtimeOnly",
            "compileOnly",
            "testImplementation",
            "testCompile",
            "testRuntimeOnly",
            "androidTestImplementation",
        ];

        for prefix in &dep_prefixes {
            if !trimmed.starts_with(prefix) {
                continue;
            }

            let rest = &trimmed[prefix.len()..];
            // Must be followed by whitespace, '(' or nothing meaningful
            let first_char = rest.chars().next().unwrap_or(' ');
            if first_char != ' ' && first_char != '(' {
                continue;
            }

            let rest = rest.trim();

            let rest = rest.trim_start_matches('(');
            let dep_str = if let Some(s) = extract_quoted_string(rest) {
                s
            } else {
                continue;
            };

            // Parse "group:artifact:version" or "group:artifact"
            let parts: Vec<&str> = dep_str.split(':').collect();
            if parts.len() >= 3 {
                deps.push((
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                ));
            } else if parts.len() == 2 {
                deps.push((parts[0].to_string(), parts[1].to_string(), String::new()));
            }
            break;
        }
    }

    deps
}

/// Extract the first single- or double-quoted string from a line fragment.
fn extract_quoted_string(s: &str) -> Option<String> {
    let s = s.trim();
    let (quote, start) = if let Some(pos) = s.find('"') {
        ('"', pos)
    } else if let Some(pos) = s.find('\'') {
        ('\'', pos)
    } else {
        return None;
    };

    let after = &s[start + 1..];
    let end = after.find(quote)?;
    Some(after[..end].to_string())
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_gradle_dependencies ---

    #[test]
    fn gradle_single_quote() {
        let content = "    implementation 'com.google.guava:guava:33.0.0-jre'";
        let deps = parse_gradle_dependencies(content, false);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], ("com.google.guava".to_string(), "guava".to_string(), "33.0.0-jre".to_string()));
    }

    #[test]
    fn gradle_double_quote() {
        let content = "    implementation \"com.google.guava:guava:33.0.0\"";
        let deps = parse_gradle_dependencies(content, false);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].1, "guava");
    }

    #[test]
    fn gradle_kotlin_dsl() {
        let content = "    implementation(\"com.google.guava:guava:33.0.0\")";
        let deps = parse_gradle_dependencies(content, false);
        assert_eq!(deps.len(), 1);
    }

    #[test]
    fn gradle_no_version() {
        let content = "    implementation 'com.google.guava:guava'";
        let deps = parse_gradle_dependencies(content, false);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].2, ""); // empty version
    }

    #[test]
    fn gradle_api_config() {
        let content = "    api 'com.google.guava:guava:33.0.0'";
        let deps = parse_gradle_dependencies(content, false);
        assert_eq!(deps.len(), 1);
    }

    #[test]
    fn gradle_compile_config() {
        let content = "    compile 'com.google.guava:guava:33.0.0'";
        let deps = parse_gradle_dependencies(content, false);
        assert_eq!(deps.len(), 1);
    }

    #[test]
    fn gradle_runtime_only() {
        let content = "    runtimeOnly 'com.h2database:h2:2.2.224'";
        let deps = parse_gradle_dependencies(content, false);
        assert_eq!(deps.len(), 1);
    }

    #[test]
    fn gradle_test_excluded() {
        let content = "    testImplementation 'junit:junit:4.13.2'";
        let deps = parse_gradle_dependencies(content, false);
        assert!(deps.is_empty());
    }

    #[test]
    fn gradle_test_included_with_dev() {
        let content = "    testImplementation 'junit:junit:4.13.2'";
        let deps = parse_gradle_dependencies(content, true);
        assert_eq!(deps.len(), 1);
    }

    #[test]
    fn gradle_multiple_deps() {
        let content = "    implementation 'com.a:b:1.0'\n    implementation 'com.c:d:2.0'";
        let deps = parse_gradle_dependencies(content, false);
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn gradle_non_dep_lines_ignored() {
        let content = "plugins {\n    id 'java'\n}\n\nrepositories {\n    mavenCentral()\n}\n\ndependencies {\n    implementation 'com.a:b:1.0'\n}";
        let deps = parse_gradle_dependencies(content, false);
        assert_eq!(deps.len(), 1);
    }

    // --- extract_quoted_string ---

    #[test]
    fn extract_double_quoted() {
        assert_eq!(extract_quoted_string("\"hello\""), Some("hello".to_string()));
    }

    #[test]
    fn extract_single_quoted() {
        assert_eq!(extract_quoted_string("'hello'"), Some("hello".to_string()));
    }

    #[test]
    fn extract_no_quotes() {
        assert_eq!(extract_quoted_string("hello"), None);
    }

    #[test]
    fn extract_with_prefix() {
        assert_eq!(extract_quoted_string("= \"value\""), Some("value".to_string()));
    }
}

fn read_gradle_project_name(root: &Path) -> String {
    let settings = root.join("settings.gradle");
    let settings_kts = root.join("settings.gradle.kts");

    let path = if settings_kts.exists() {
        settings_kts
    } else if settings.exists() {
        settings
    } else {
        return root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());
    };

    if let Ok(content) = fs::read_to_string(&path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("rootProject.name") {
                if let Some(name) = extract_quoted_string(trimmed) {
                    return name;
                }
            }
        }
    }

    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string())
}
