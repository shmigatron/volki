use crate::core::volkiwithstds::collections::ToString;
use crate::core::volkiwithstds::collections::{String, Vec};
/// Extract a field value from RFC 822-style metadata (Python METADATA).
/// Looks for `Field: value` format.
pub fn get_rfc822_field(content: &str, field: &str) -> Option<String> {
    let prefix = crate::vformat!("{field}: ");
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix(prefix.as_str()) {
            let val = rest.trim().to_vstring();
            if !val.is_empty() && val != "UNKNOWN" {
                return Some(val);
            }
        }
    }
    None
}

/// Parse Go `go.mod` require block into (module_path, version) pairs.
pub fn parse_go_mod_requires(content: &str) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    let mut in_require = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("require (") || trimmed == "require (" {
            in_require = true;
            continue;
        }

        if in_require {
            if trimmed == ")" {
                in_require = false;
                continue;
            }

            // Skip indirect dependencies
            if trimmed.contains("// indirect") {
                // Still include them - they're real dependencies
            }

            // Format: module/path v1.2.3
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let module = parts[0].to_vstring();
                let version = parts[1].to_vstring();
                deps.push((module, version));
            }
            continue;
        }

        // Single-line require: `require module/path v1.2.3`
        if let Some(rest) = trimmed.strip_prefix("require ") {
            let rest = rest.trim();
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                deps.push((parts[0].to_vstring(), parts[1].to_vstring()));
            }
        }
    }

    deps
}

/// Parse Gemfile.lock GEM/specs section into (name, version) pairs.
pub fn parse_gemfile_lock_gems(content: &str) -> Vec<(String, String)> {
    let mut gems = Vec::new();
    let mut in_specs = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "specs:" {
            in_specs = true;
            continue;
        }

        if in_specs {
            // End of specs section (new section header or empty line followed by section)
            if !line.starts_with(" ") && !trimmed.is_empty() {
                in_specs = false;
                continue;
            }

            // Gem entries are indented with 4 spaces: "    gem_name (1.2.3)"
            // Sub-dependencies are indented with 6 spaces
            let indent = line.len() - line.trim_start().len();
            if indent == 4 && trimmed.contains("(") {
                if let Some((name, rest)) = trimmed.split_once(' ') {
                    let version = rest
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .to_vstring();
                    gems.push((name.to_vstring(), version));
                }
            }
        }
    }

    gems
}

/// Parse pubspec.lock packages into (name, version) pairs.
pub fn parse_pubspec_lock_packages(content: &str) -> Vec<(String, String)> {
    let mut packages = Vec::new();
    let mut current_name: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Package names are at 2-space indent: "  package_name:"
        if line.starts_with("  ") && !line.starts_with("    ") && trimmed.ends_with(':') {
            current_name = Some(trimmed.trim_end_matches(':').to_vstring());
            continue;
        }

        // Version is at 4-space indent: '    version: "1.2.3"'
        if line.starts_with("    ") && !line.starts_with("      ") {
            if let Some(ref name) = current_name {
                if let Some(rest) = trimmed.strip_prefix("version: ") {
                    let version = rest.trim_matches('"').to_vstring();
                    packages.push((name.clone(), version));
                    current_name = None;
                }
            }
        }
    }

    packages
}

/// Parse mix.lock entries into (name, version) pairs.
/// Format: `"dep_name": {:hex, :dep_name, "version", ...}`
pub fn parse_mix_lock_deps(content: &str) -> Vec<(String, String)> {
    let mut deps = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim().trim_end_matches(',');

        // Match: "name": {:hex, :name, "version", ...}
        if let Some(colon_pos) = trimmed.find("\":") {
            let name = trimmed[..colon_pos]
                .trim()
                .trim_start_matches('"')
                .to_vstring();

            if name.is_empty() || name.starts_with("%") {
                continue;
            }

            // Look for version string (third element in tuple)
            let rest = &trimmed[colon_pos + 2..];
            if let Some(version) = extract_mix_version(rest) {
                deps.push((name, version));
            }
        }
    }

    deps
}

fn extract_mix_version(tuple_str: &str) -> Option<String> {
    // The tuple looks like: {:hex, :name, "version", ...}
    // We want the first quoted string after the tuple start that looks like a version
    let mut in_quotes = false;
    let mut current = String::new();
    let mut found_strings = 0;

    for ch in tuple_str.chars() {
        if ch == '"' {
            if in_quotes {
                found_strings += 1;
                // The version is typically the first quoted string that contains a dot
                if current.contains(".") {
                    return Some(current);
                }
                current.clear();
                in_quotes = false;
            } else {
                in_quotes = true;
                current.clear();
            }
        } else if in_quotes {
            current.push(ch);
        }

        // Don't search too far
        if found_strings > 5 {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- get_rfc822_field ---

    #[test]
    fn rfc822_simple_field() {
        assert_eq!(
            get_rfc822_field("License: MIT", "License"),
            Some(crate::vstr!("MIT"))
        );
    }

    #[test]
    fn rfc822_full_metadata() {
        let content = "Metadata-Version: 2.1\nName: requests\nVersion: 2.31.0\nLicense: Apache-2.0\nAuthor: Kenneth Reitz";
        assert_eq!(
            get_rfc822_field(content, "License"),
            Some(crate::vstr!("Apache-2.0"))
        );
        assert_eq!(
            get_rfc822_field(content, "Name"),
            Some(crate::vstr!("requests"))
        );
    }

    #[test]
    fn rfc822_unknown_filtered() {
        assert_eq!(get_rfc822_field("License: UNKNOWN", "License"), None);
    }

    #[test]
    fn rfc822_empty_value() {
        assert_eq!(get_rfc822_field("License: ", "License"), None);
    }

    #[test]
    fn rfc822_missing_field() {
        assert_eq!(get_rfc822_field("Name: foo", "License"), None);
    }

    #[test]
    fn rfc822_case_sensitive() {
        // Field matching is case-sensitive (prefix match)
        assert_eq!(get_rfc822_field("license: MIT", "License"), None);
    }

    // --- parse_go_mod_requires ---

    #[test]
    fn go_mod_single_require() {
        let content = "require github.com/pkg/errors v0.9.1";
        let deps = parse_go_mod_requires(content);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, "github.com/pkg/errors");
        assert_eq!(deps[0].1, "v0.9.1");
    }

    #[test]
    fn go_mod_require_block() {
        let content = "require (\n\tgithub.com/a/b v1.0.0\n\tgithub.com/c/d v2.0.0\n)";
        let deps = parse_go_mod_requires(content);
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn go_mod_indirect_included() {
        let content = "require (\n\tgithub.com/a/b v1.0.0 // indirect\n)";
        let deps = parse_go_mod_requires(content);
        assert_eq!(deps.len(), 1);
    }

    #[test]
    fn go_mod_mixed_format() {
        let content = "module example.com/mymod\n\ngo 1.21\n\nrequire github.com/single/dep v1.0.0\n\nrequire (\n\tgithub.com/block/dep v2.0.0\n)";
        let deps = parse_go_mod_requires(content);
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn go_mod_empty() {
        assert!(parse_go_mod_requires("module example.com/foo\n\ngo 1.21\n").is_empty());
    }

    #[test]
    fn go_mod_with_module_line() {
        let content = "module example.com/mymod\n\nrequire (\n\tgolang.org/x/text v0.14.0\n)";
        let deps = parse_go_mod_requires(content);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, "golang.org/x/text");
    }

    // --- parse_gemfile_lock_gems ---

    #[test]
    fn gemfile_lock_simple() {
        let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.1.0)\n    rake (13.1.0)\n";
        let gems = parse_gemfile_lock_gems(content);
        assert_eq!(gems.len(), 2);
        assert_eq!(gems[0], (crate::vstr!("rails"), crate::vstr!("7.1.0")));
        assert_eq!(gems[1], (crate::vstr!("rake"), crate::vstr!("13.1.0")));
    }

    #[test]
    fn gemfile_lock_subdeps_filtered() {
        let content = "GEM\n  specs:\n    rails (7.1.0)\n      actioncable (= 7.1.0)\n      actionpack (= 7.1.0)\n    rake (13.1.0)\n";
        let gems = parse_gemfile_lock_gems(content);
        assert_eq!(gems.len(), 2);
        assert_eq!(gems[0].0, "rails");
        assert_eq!(gems[1].0, "rake");
    }

    #[test]
    fn gemfile_lock_empty_specs() {
        let content = "GEM\n  specs:\nPLATFORMS\n";
        let gems = parse_gemfile_lock_gems(content);
        assert!(gems.is_empty());
    }

    #[test]
    fn gemfile_lock_real_format() {
        let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    actioncable (7.1.3)\n      actionpack (= 7.1.3)\n      nio4r (~> 2.0)\n    actionpack (7.1.3)\n      rack (>= 2.2.4)\n\nPLATFORMS\n  ruby\n";
        let gems = parse_gemfile_lock_gems(content);
        assert_eq!(gems.len(), 2);
        assert_eq!(gems[0].0, "actioncable");
        assert_eq!(gems[1].0, "actionpack");
    }

    // --- parse_pubspec_lock_packages ---

    #[test]
    fn pubspec_lock_simple() {
        let content =
            "packages:\n  http:\n    version: \"1.2.0\"\n  path:\n    version: \"1.9.0\"\n";
        let pkgs = parse_pubspec_lock_packages(content);
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0], (crate::vstr!("http"), crate::vstr!("1.2.0")));
        assert_eq!(pkgs[1], (crate::vstr!("path"), crate::vstr!("1.9.0")));
    }

    #[test]
    fn pubspec_lock_quoted_version() {
        let content = "packages:\n  http:\n    version: \"1.2.0\"\n";
        let pkgs = parse_pubspec_lock_packages(content);
        assert_eq!(pkgs[0].1, "1.2.0");
    }

    #[test]
    fn pubspec_lock_no_version_skipped() {
        let content = "packages:\n  noversion:\n    description: something\n";
        let pkgs = parse_pubspec_lock_packages(content);
        assert!(pkgs.is_empty());
    }

    #[test]
    fn pubspec_lock_real_format() {
        let content = "packages:\n  async:\n    dependency: transitive\n    description:\n      name: async\n    source: hosted\n    version: \"2.11.0\"\n  collection:\n    dependency: direct main\n    version: \"1.18.0\"\n";
        let pkgs = parse_pubspec_lock_packages(content);
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].0, "async");
        assert_eq!(pkgs[1].0, "collection");
    }

    // --- parse_mix_lock_deps ---

    #[test]
    fn mix_lock_simple() {
        let content = r#"  "jason": {:hex, :jason, "1.4.1", "abc123"}"#;
        let deps = parse_mix_lock_deps(content);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, "jason");
        assert_eq!(deps[0].1, "1.4.1");
    }

    #[test]
    fn mix_lock_multiple() {
        let content = r#"  "jason": {:hex, :jason, "1.4.1", "abc"},
  "plug": {:hex, :plug, "1.15.0", "def"},"#;
        let deps = parse_mix_lock_deps(content);
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].0, "jason");
        assert_eq!(deps[1].0, "plug");
    }

    #[test]
    fn mix_lock_empty() {
        assert!(parse_mix_lock_deps("%{}\n").is_empty());
    }

    #[test]
    fn mix_lock_real_format() {
        let content = r#"%{
  "decimal": {:hex, :decimal, "2.1.1", "abc123", [:mix], [], "hexpm", "def456"},
  "ecto": {:hex, :ecto, "3.11.1", "ghi789", [:mix], [{:decimal, "~> 2.0", [hex: :decimal]}], "hexpm", "jkl012"},
}"#;
        let deps = parse_mix_lock_deps(content);
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].0, "decimal");
        assert_eq!(deps[0].1, "2.1.1");
        assert_eq!(deps[1].0, "ecto");
        assert_eq!(deps[1].1, "3.11.1");
    }
}
