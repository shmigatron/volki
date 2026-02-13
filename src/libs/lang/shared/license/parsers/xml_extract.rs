/// Extract text content between `<tag>` and `</tag>` pairs.
pub fn extract_tag_contents(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(start) = xml[search_from..].find(&open) {
        let content_start = search_from + start + open.len();
        if let Some(end) = xml[content_start..].find(&close) {
            let content = xml[content_start..content_start + end].trim().to_string();
            if !content.is_empty() {
                results.push(content);
            }
            search_from = content_start + end + close.len();
        } else {
            break;
        }
    }

    results
}

/// Extract attribute values from tags like `<tag attr="value">` or `<tag attr="value" />`.
#[allow(dead_code)]
pub fn extract_tag_attribute(xml: &str, tag: &str, attr: &str) -> Vec<String> {
    let tag_open = format!("<{tag} ");
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(start) = xml[search_from..].find(&tag_open) {
        let tag_start = search_from + start;
        // Find the end of this tag (either > or />)
        if let Some(end_offset) = xml[tag_start..].find('>') {
            let tag_content = &xml[tag_start..tag_start + end_offset + 1];
            if let Some(val) = extract_attr_value(tag_content, attr) {
                results.push(val);
            }
            search_from = tag_start + end_offset + 1;
        } else {
            break;
        }
    }

    results
}

fn extract_attr_value(tag_str: &str, attr: &str) -> Option<String> {
    let patterns = [format!("{attr}=\""), format!("{attr}='")];
    for pattern in &patterns {
        if let Some(start) = tag_str.find(pattern.as_str()) {
            let val_start = start + pattern.len();
            let quote = tag_str.as_bytes()[start + pattern.len() - 1];
            if let Some(end) = tag_str[val_start..].find(quote as char) {
                return Some(tag_str[val_start..val_start + end].to_string());
            }
        }
    }
    None
}

/// Parse Maven pom.xml `<dependency>` blocks into (groupId, artifactId, version).
pub fn parse_maven_dependencies(xml: &str) -> Vec<(String, String, String)> {
    let mut deps = Vec::new();
    let mut search_from = 0;

    while let Some(start) = xml[search_from..].find("<dependency>") {
        let dep_start = search_from + start;
        if let Some(end) = xml[dep_start..].find("</dependency>") {
            let block = &xml[dep_start..dep_start + end + "</dependency>".len()];

            let group_id = first_tag_content(block, "groupId").unwrap_or_default();
            let artifact_id = first_tag_content(block, "artifactId").unwrap_or_default();
            let version = first_tag_content(block, "version").unwrap_or_default();

            if !group_id.is_empty() && !artifact_id.is_empty() {
                deps.push((group_id, artifact_id, version));
            }

            search_from = dep_start + end + "</dependency>".len();
        } else {
            break;
        }
    }

    deps
}

/// Parse .csproj `<PackageReference Include="name" Version="ver" />` entries.
pub fn parse_csproj_package_references(xml: &str) -> Vec<(String, String)> {
    let mut packages = Vec::new();

    // Match both self-closing and full tags
    let tag = "PackageReference";
    let mut search_from = 0;

    while let Some(start) = xml[search_from..].find("<PackageReference") {
        let tag_start = search_from + start;
        if let Some(end_offset) = xml[tag_start..].find('>') {
            let tag_str = &xml[tag_start..tag_start + end_offset + 1];
            let name = extract_attr_value(tag_str, "Include")
                .or_else(|| extract_attr_value(tag_str, "include"));
            let version = extract_attr_value(tag_str, "Version")
                .or_else(|| extract_attr_value(tag_str, "version"));

            if let Some(name) = name {
                packages.push((name, version.unwrap_or_default()));
            }

            search_from = tag_start + end_offset + 1;
        } else {
            break;
        }
    }

    let _ = tag; // suppress unused warning

    packages
}

/// Extract license name from Maven pom.xml `<licenses><license><name>` section.
/// Normalizes common POM license names to SPDX identifiers.
pub fn parse_pom_license(xml: &str) -> Option<String> {
    // Find <licenses> block
    let lic_start = xml.find("<licenses>")?;
    let lic_end = xml[lic_start..].find("</licenses>")?;
    let block = &xml[lic_start..lic_start + lic_end];

    let raw = first_tag_content(block, "name")?;

    // Try to get a URL as fallback context
    let url = first_tag_content(block, "url");

    Some(normalize_pom_license(&raw, url.as_deref()))
}

fn normalize_pom_license(name: &str, url: Option<&str>) -> String {
    let lower = name.to_lowercase();

    // Apache variants
    if lower.contains("apache") {
        if lower.contains("2") {
            return "Apache-2.0".to_string();
        }
        if lower.contains("1.1") {
            return "Apache-1.1".to_string();
        }
        return "Apache-2.0".to_string();
    }

    // MIT
    if lower.contains("mit") {
        return "MIT".to_string();
    }

    // BSD variants
    if lower.contains("bsd") {
        if lower.contains("2") || lower.contains("simplified") {
            return "BSD-2-Clause".to_string();
        }
        if lower.contains("3") || lower.contains("new") || lower.contains("revised") {
            return "BSD-3-Clause".to_string();
        }
        return "BSD-3-Clause".to_string();
    }

    // GPL variants
    if lower.contains("gnu") || lower.contains("gpl") {
        if lower.contains("lesser") || lower.contains("lgpl") || lower.contains("library") {
            if lower.contains("3") {
                return "LGPL-3.0".to_string();
            }
            return "LGPL-2.1".to_string();
        }
        if lower.contains("affero") || lower.contains("agpl") {
            return "AGPL-3.0".to_string();
        }
        if lower.contains("3") {
            return "GPL-3.0".to_string();
        }
        return "GPL-2.0".to_string();
    }

    // MPL
    if lower.contains("mozilla") || lower.contains("mpl") {
        return "MPL-2.0".to_string();
    }

    // Eclipse
    if lower.contains("eclipse") || lower.contains("epl") {
        if lower.contains("2") {
            return "EPL-2.0".to_string();
        }
        return "EPL-1.0".to_string();
    }

    // CDDL
    if lower.contains("cddl") || lower.contains("common development and distribution") {
        return "CDDL-1.0".to_string();
    }

    // ISC
    if lower.contains("isc") {
        return "ISC".to_string();
    }

    // Unlicense
    if lower.contains("unlicense") || lower == "public domain" {
        return "Unlicense".to_string();
    }

    // CC0
    if lower.contains("cc0") || lower.contains("creative commons") && lower.contains("zero") {
        return "CC0-1.0".to_string();
    }

    // If the name looks like it's already an SPDX identifier, return as-is
    if !name.contains(' ') && name.len() < 30 {
        return name.to_string();
    }

    // Last resort: try URL
    if let Some(u) = url {
        if let Some(l) = license_from_url(u) {
            return l;
        }
    }

    name.to_string()
}

/// Extract license from NuGet .nuspec `<license>` tag.
pub fn parse_nuspec_license(xml: &str) -> Option<String> {
    // Try <license type="expression">MIT</license>
    let contents = extract_tag_contents(xml, "license");
    if let Some(first) = contents.first() {
        if !first.is_empty() {
            return Some(first.clone());
        }
    }

    // Try <licenseUrl> as fallback
    let urls = extract_tag_contents(xml, "licenseUrl");
    if let Some(url) = urls.first() {
        return license_from_url(url);
    }

    None
}

fn first_tag_content(xml: &str, tag: &str) -> Option<String> {
    let contents = extract_tag_contents(xml, tag);
    contents.into_iter().next()
}

fn license_from_url(url: &str) -> Option<String> {
    let lower = url.to_lowercase();
    if lower.contains("mit") {
        Some("MIT".to_string())
    } else if lower.contains("apache") {
        Some("Apache-2.0".to_string())
    } else if lower.contains("bsd") {
        Some("BSD".to_string())
    } else if lower.contains("lgpl") {
        Some("LGPL".to_string())
    } else if lower.contains("gpl") {
        Some("GPL".to_string())
    } else if lower.contains("mpl") {
        Some("MPL-2.0".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- extract_tag_contents ---

    #[test]
    fn tag_contents_simple() {
        let result = extract_tag_contents("<name>hello</name>", "name");
        assert_eq!(result, vec!["hello".to_string()]);
    }

    #[test]
    fn tag_contents_multiple() {
        let xml = "<item>a</item><item>b</item>";
        let result = extract_tag_contents(xml, "item");
        assert_eq!(result, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn tag_contents_empty_filtered() {
        let result = extract_tag_contents("<name>  </name>", "name");
        assert!(result.is_empty());
    }

    #[test]
    fn tag_contents_whitespace_trimmed() {
        let result = extract_tag_contents("<name>  hello  </name>", "name");
        assert_eq!(result, vec!["hello".to_string()]);
    }

    #[test]
    fn tag_contents_no_match() {
        let result = extract_tag_contents("<other>val</other>", "name");
        assert!(result.is_empty());
    }

    #[test]
    fn tag_contents_unclosed() {
        let result = extract_tag_contents("<name>hello", "name");
        assert!(result.is_empty());
    }

    // --- parse_maven_dependencies ---

    #[test]
    fn maven_single_dep() {
        let xml = "<dependencies><dependency><groupId>com.google</groupId><artifactId>guava</artifactId><version>33.0</version></dependency></dependencies>";
        let deps = parse_maven_dependencies(xml);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], ("com.google".to_string(), "guava".to_string(), "33.0".to_string()));
    }

    #[test]
    fn maven_multiple_deps() {
        let xml = "<dependencies>\
            <dependency><groupId>com.a</groupId><artifactId>x</artifactId><version>1</version></dependency>\
            <dependency><groupId>com.b</groupId><artifactId>y</artifactId><version>2</version></dependency>\
            </dependencies>";
        let deps = parse_maven_dependencies(xml);
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn maven_no_version() {
        let xml = "<dependency><groupId>com.a</groupId><artifactId>x</artifactId></dependency>";
        let deps = parse_maven_dependencies(xml);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].2, ""); // empty version
    }

    #[test]
    fn maven_incomplete_skipped() {
        let xml = "<dependency><groupId>com.a</groupId></dependency>";
        let deps = parse_maven_dependencies(xml);
        assert!(deps.is_empty()); // no artifactId
    }

    #[test]
    fn maven_real_pom() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>com.example</groupId>
  <artifactId>myapp</artifactId>
  <version>1.0.0</version>
  <dependencies>
    <dependency>
      <groupId>org.slf4j</groupId>
      <artifactId>slf4j-api</artifactId>
      <version>2.0.12</version>
    </dependency>
    <dependency>
      <groupId>junit</groupId>
      <artifactId>junit</artifactId>
      <version>4.13.2</version>
      <scope>test</scope>
    </dependency>
  </dependencies>
</project>"#;
        let deps = parse_maven_dependencies(xml);
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].1, "slf4j-api");
        assert_eq!(deps[1].1, "junit");
    }

    // --- parse_csproj_package_references ---

    #[test]
    fn csproj_self_closing() {
        let xml = r#"<PackageReference Include="Newtonsoft.Json" Version="13.0.3" />"#;
        let pkgs = parse_csproj_package_references(xml);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0], ("Newtonsoft.Json".to_string(), "13.0.3".to_string()));
    }

    #[test]
    fn csproj_no_version() {
        let xml = r#"<PackageReference Include="SomePackage" />"#;
        let pkgs = parse_csproj_package_references(xml);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].1, ""); // empty version
    }

    #[test]
    fn csproj_multiple_refs() {
        let xml = r#"<ItemGroup>
    <PackageReference Include="A" Version="1.0" />
    <PackageReference Include="B" Version="2.0" />
</ItemGroup>"#;
        let pkgs = parse_csproj_package_references(xml);
        assert_eq!(pkgs.len(), 2);
    }

    #[test]
    fn csproj_real_format() {
        let xml = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
    <PackageReference Include="Serilog" Version="3.1.1" />
  </ItemGroup>
</Project>"#;
        let pkgs = parse_csproj_package_references(xml);
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].0, "Newtonsoft.Json");
        assert_eq!(pkgs[1].0, "Serilog");
    }

    // --- parse_pom_license ---

    #[test]
    fn pom_license_apache() {
        let xml = "<licenses><license><name>Apache License, Version 2.0</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("Apache-2.0".to_string()));
    }

    #[test]
    fn pom_license_mit() {
        let xml = "<licenses><license><name>MIT License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("MIT".to_string()));
    }

    #[test]
    fn pom_license_bsd2() {
        let xml = "<licenses><license><name>BSD 2-Clause License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("BSD-2-Clause".to_string()));
    }

    #[test]
    fn pom_license_bsd3() {
        let xml = "<licenses><license><name>BSD 3-Clause \"New\" License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("BSD-3-Clause".to_string()));
    }

    #[test]
    fn pom_license_gpl2() {
        let xml = "<licenses><license><name>GNU General Public License v2</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("GPL-2.0".to_string()));
    }

    #[test]
    fn pom_license_gpl3() {
        let xml = "<licenses><license><name>GNU General Public License v3</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("GPL-3.0".to_string()));
    }

    #[test]
    fn pom_license_lgpl() {
        let xml = "<licenses><license><name>GNU Lesser General Public License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("LGPL-2.1".to_string()));
    }

    #[test]
    fn pom_license_agpl() {
        let xml = "<licenses><license><name>GNU Affero General Public License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("AGPL-3.0".to_string()));
    }

    #[test]
    fn pom_license_mpl() {
        let xml = "<licenses><license><name>Mozilla Public License 2.0</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("MPL-2.0".to_string()));
    }

    #[test]
    fn pom_license_eclipse() {
        let xml = "<licenses><license><name>Eclipse Public License 2.0</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("EPL-2.0".to_string()));
    }

    #[test]
    fn pom_license_cddl() {
        let xml = "<licenses><license><name>CDDL 1.0</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("CDDL-1.0".to_string()));
    }

    #[test]
    fn pom_license_isc() {
        let xml = "<licenses><license><name>ISC License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("ISC".to_string()));
    }

    #[test]
    fn pom_license_unlicense() {
        let xml = "<licenses><license><name>The Unlicense</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("Unlicense".to_string()));
    }

    #[test]
    fn pom_license_cc0() {
        let xml = "<licenses><license><name>CC0 1.0 Universal</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("CC0-1.0".to_string()));
    }

    #[test]
    fn pom_license_spdx_passthrough() {
        let xml = "<licenses><license><name>MIT</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("MIT".to_string()));
    }

    #[test]
    fn pom_license_url_fallback() {
        let xml = "<licenses><license><name>Some Weird License Name That Is Very Long And Unknown</name><url>https://opensource.org/licenses/MIT</url></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some("MIT".to_string()));
    }

    #[test]
    fn pom_license_no_licenses_block() {
        let xml = "<project><name>myapp</name></project>";
        assert_eq!(parse_pom_license(xml), None);
    }

    // --- parse_nuspec_license ---

    #[test]
    fn nuspec_license_expression() {
        // extract_tag_contents matches <license> exactly, not <license type="...">
        let xml = "<package><metadata><license>MIT</license></metadata></package>";
        assert_eq!(parse_nuspec_license(xml), Some("MIT".to_string()));
    }

    #[test]
    fn nuspec_license_url_fallback_mit() {
        let xml = "<package><metadata><licenseUrl>https://licenses.nuget.org/MIT</licenseUrl></metadata></package>";
        assert_eq!(parse_nuspec_license(xml), Some("MIT".to_string()));
    }

    #[test]
    fn nuspec_license_url_fallback_apache() {
        let xml = "<package><metadata><licenseUrl>https://www.apache.org/licenses/LICENSE-2.0</licenseUrl></metadata></package>";
        assert_eq!(parse_nuspec_license(xml), Some("Apache-2.0".to_string()));
    }

    #[test]
    fn nuspec_license_none() {
        let xml = "<package><metadata><id>test</id></metadata></package>";
        assert_eq!(parse_nuspec_license(xml), None);
    }

    #[test]
    fn nuspec_real_format() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd">
  <metadata>
    <id>Newtonsoft.Json</id>
    <version>13.0.3</version>
    <license>MIT</license>
  </metadata>
</package>"#;
        assert_eq!(parse_nuspec_license(xml), Some("MIT".to_string()));
    }
}
