use crate::core::volkiwithstds::collections::ToString;
use crate::core::volkiwithstds::collections::xml::Xml;
use crate::core::volkiwithstds::collections::{String, Vec};

/// Parse Maven pom.xml `<dependency>` blocks into (groupId, artifactId, version).
pub fn parse_maven_dependencies(raw: &str) -> Vec<(String, String, String)> {
    let mut deps = Vec::new();
    let mut search_from = 0;

    while let Some(start) = raw[search_from..].find("<dependency>") {
        let dep_start = search_from + start;
        if let Some(end) = raw[dep_start..].find("</dependency>") {
            let block = &raw[dep_start..dep_start + end + "</dependency>".len()];
            let xml = Xml::new(block);

            let group_id = xml.first_tag_content("groupId").unwrap_or_default();
            let artifact_id = xml.first_tag_content("artifactId").unwrap_or_default();
            let version = xml.first_tag_content("version").unwrap_or_default();

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
pub fn parse_csproj_package_references(raw: &str) -> Vec<(String, String)> {
    let mut packages = Vec::new();
    let mut search_from = 0;

    while let Some(start) = raw[search_from..].find("<PackageReference") {
        let tag_start = search_from + start;
        if let Some(end_offset) = raw[tag_start..].find('>') {
            let tag_str = &raw[tag_start..tag_start + end_offset + 1];
            let name =
                Xml::attr_value(tag_str, "Include").or_else(|| Xml::attr_value(tag_str, "include"));
            let version =
                Xml::attr_value(tag_str, "Version").or_else(|| Xml::attr_value(tag_str, "version"));

            if let Some(name) = name {
                packages.push((name, version.unwrap_or_default()));
            }

            search_from = tag_start + end_offset + 1;
        } else {
            break;
        }
    }

    packages
}

/// Extract license name from Maven pom.xml `<licenses><license><name>` section.
/// Normalizes common POM license names to SPDX identifiers.
pub fn parse_pom_license(raw: &str) -> Option<String> {
    let lic_start = raw.find("<licenses>")?;
    let lic_end = raw[lic_start..].find("</licenses>")?;
    let block = &raw[lic_start..lic_start + lic_end];
    let xml = Xml::new(block);

    let name = xml.first_tag_content("name")?;
    let url = xml.first_tag_content("url");

    Some(normalize_pom_license(&name, url.as_deref()))
}

fn normalize_pom_license(name: &str, url: Option<&str>) -> String {
    let lower = name.to_lowercase();

    if lower.contains("apache") {
        if lower.contains("2") {
            return crate::vstr!("Apache-2.0");
        }
        if lower.contains("1.1") {
            return crate::vstr!("Apache-1.1");
        }
        return crate::vstr!("Apache-2.0");
    }

    if lower.contains("mit") {
        return crate::vstr!("MIT");
    }

    if lower.contains("bsd") {
        if lower.contains("2") || lower.contains("simplified") {
            return crate::vstr!("BSD-2-Clause");
        }
        if lower.contains("3") || lower.contains("new") || lower.contains("revised") {
            return crate::vstr!("BSD-3-Clause");
        }
        return crate::vstr!("BSD-3-Clause");
    }

    if lower.contains("gnu") || lower.contains("gpl") {
        if lower.contains("lesser") || lower.contains("lgpl") || lower.contains("library") {
            if lower.contains("3") {
                return crate::vstr!("LGPL-3.0");
            }
            return crate::vstr!("LGPL-2.1");
        }
        if lower.contains("affero") || lower.contains("agpl") {
            return crate::vstr!("AGPL-3.0");
        }
        if lower.contains("3") {
            return crate::vstr!("GPL-3.0");
        }
        return crate::vstr!("GPL-2.0");
    }

    if lower.contains("mozilla") || lower.contains("mpl") {
        return crate::vstr!("MPL-2.0");
    }

    if lower.contains("eclipse") || lower.contains("epl") {
        if lower.contains("2") {
            return crate::vstr!("EPL-2.0");
        }
        return crate::vstr!("EPL-1.0");
    }

    if lower.contains("cddl") || lower.contains("common development and distribution") {
        return crate::vstr!("CDDL-1.0");
    }

    if lower.contains("isc") {
        return crate::vstr!("ISC");
    }

    if lower.contains("unlicense") || lower == "public domain" {
        return crate::vstr!("Unlicense");
    }

    if lower.contains("cc0") || lower.contains("creative commons") && lower.contains("zero") {
        return crate::vstr!("CC0-1.0");
    }

    if !name.contains(" ") && name.len() < 30 {
        return name.to_vstring();
    }

    if let Some(u) = url {
        if let Some(l) = license_from_url(u) {
            return l;
        }
    }

    name.to_vstring()
}

/// Extract license from NuGet .nuspec `<license>` tag.
pub fn parse_nuspec_license(raw: &str) -> Option<String> {
    let xml = Xml::new(raw);

    let contents = xml.tag_contents("license");
    if let Some(first) = contents.first() {
        if !first.is_empty() {
            return Some(first.clone());
        }
    }

    let urls = xml.tag_contents("licenseUrl");
    if let Some(url) = urls.first() {
        return license_from_url(url);
    }

    None
}

fn license_from_url(url: &str) -> Option<String> {
    let lower = url.to_lowercase();
    if lower.contains("mit") {
        Some(crate::vstr!("MIT"))
    } else if lower.contains("apache") {
        Some(crate::vstr!("Apache-2.0"))
    } else if lower.contains("bsd") {
        Some(crate::vstr!("BSD"))
    } else if lower.contains("lgpl") {
        Some(crate::vstr!("LGPL"))
    } else if lower.contains("gpl") {
        Some(crate::vstr!("GPL"))
    } else if lower.contains("mpl") {
        Some(crate::vstr!("MPL-2.0"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_maven_dependencies ---

    #[test]
    fn maven_single_dep() {
        let xml = "<dependencies><dependency><groupId>com.google</groupId><artifactId>guava</artifactId><version>33.0</version></dependency></dependencies>";
        let deps = parse_maven_dependencies(xml);
        assert_eq!(deps.len(), 1);
        assert_eq!(
            deps[0],
            (
                crate::vstr!("com.google"),
                crate::vstr!("guava"),
                crate::vstr!("33.0")
            )
        );
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
        assert_eq!(
            pkgs[0],
            (crate::vstr!("Newtonsoft.Json"), crate::vstr!("13.0.3"))
        );
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
        let xml =
            "<licenses><license><name>Apache License, Version 2.0</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("Apache-2.0")));
    }

    #[test]
    fn pom_license_mit() {
        let xml = "<licenses><license><name>MIT License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("MIT")));
    }

    #[test]
    fn pom_license_bsd2() {
        let xml = "<licenses><license><name>BSD 2-Clause License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("BSD-2-Clause")));
    }

    #[test]
    fn pom_license_bsd3() {
        let xml =
            "<licenses><license><name>BSD 3-Clause \"New\" License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("BSD-3-Clause")));
    }

    #[test]
    fn pom_license_gpl2() {
        let xml =
            "<licenses><license><name>GNU General Public License v2</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("GPL-2.0")));
    }

    #[test]
    fn pom_license_gpl3() {
        let xml =
            "<licenses><license><name>GNU General Public License v3</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("GPL-3.0")));
    }

    #[test]
    fn pom_license_lgpl() {
        let xml = "<licenses><license><name>GNU Lesser General Public License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("LGPL-2.1")));
    }

    #[test]
    fn pom_license_agpl() {
        let xml = "<licenses><license><name>GNU Affero General Public License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("AGPL-3.0")));
    }

    #[test]
    fn pom_license_mpl() {
        let xml = "<licenses><license><name>Mozilla Public License 2.0</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("MPL-2.0")));
    }

    #[test]
    fn pom_license_eclipse() {
        let xml = "<licenses><license><name>Eclipse Public License 2.0</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("EPL-2.0")));
    }

    #[test]
    fn pom_license_cddl() {
        let xml = "<licenses><license><name>CDDL 1.0</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("CDDL-1.0")));
    }

    #[test]
    fn pom_license_isc() {
        let xml = "<licenses><license><name>ISC License</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("ISC")));
    }

    #[test]
    fn pom_license_unlicense() {
        let xml = "<licenses><license><name>The Unlicense</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("Unlicense")));
    }

    #[test]
    fn pom_license_cc0() {
        let xml = "<licenses><license><name>CC0 1.0 Universal</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("CC0-1.0")));
    }

    #[test]
    fn pom_license_spdx_passthrough() {
        let xml = "<licenses><license><name>MIT</name></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("MIT")));
    }

    #[test]
    fn pom_license_url_fallback() {
        let xml = "<licenses><license><name>Some Weird License Name That Is Very Long And Unknown</name><url>https://opensource.org/licenses/MIT</url></license></licenses>";
        assert_eq!(parse_pom_license(xml), Some(crate::vstr!("MIT")));
    }

    #[test]
    fn pom_license_no_licenses_block() {
        let xml = "<project><name>myapp</name></project>";
        assert_eq!(parse_pom_license(xml), None);
    }

    // --- parse_nuspec_license ---

    #[test]
    fn nuspec_license_expression() {
        let xml = "<package><metadata><license>MIT</license></metadata></package>";
        assert_eq!(parse_nuspec_license(xml), Some(crate::vstr!("MIT")));
    }

    #[test]
    fn nuspec_license_url_fallback_mit() {
        let xml = "<package><metadata><licenseUrl>https://licenses.nuget.org/MIT</licenseUrl></metadata></package>";
        assert_eq!(parse_nuspec_license(xml), Some(crate::vstr!("MIT")));
    }

    #[test]
    fn nuspec_license_url_fallback_apache() {
        let xml = "<package><metadata><licenseUrl>https://www.apache.org/licenses/LICENSE-2.0</licenseUrl></metadata></package>";
        assert_eq!(parse_nuspec_license(xml), Some(crate::vstr!("Apache-2.0")));
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
        assert_eq!(parse_nuspec_license(xml), Some(crate::vstr!("MIT")));
    }
}
