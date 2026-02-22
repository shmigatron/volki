use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::path::Path;

use super::types::{PluginError, PluginRuntime, PluginSpec, ResolvedPlugin};
use crate::core::volkiwithstds::collections::json::extract_top_level;

pub fn resolve(spec: &PluginSpec, project_dir: &Path) -> Result<ResolvedPlugin, PluginError> {
    if let Some(ref rt) = spec.runtime {
        return match rt {
            PluginRuntime::Node => resolve_node(spec, project_dir),
            PluginRuntime::Python => resolve_python(spec, project_dir),
        };
    }

    if let Ok(p) = resolve_node(spec, project_dir) {
        return Ok(p);
    }
    if let Ok(p) = resolve_python(spec, project_dir) {
        return Ok(p);
    }

    Err(PluginError::NotFound(spec.name.clone()))
}

fn resolve_node(spec: &PluginSpec, project_dir: &Path) -> Result<ResolvedPlugin, PluginError> {
    let pkg_dir = project_dir.join("node_modules").join(&spec.name);
    if !pkg_dir.is_dir() {
        return Err(PluginError::NotFound(spec.name.clone()));
    }

    let entry = pkg_dir.join("volki-plugin.js");
    if entry.is_file() {
        return Ok(ResolvedPlugin {
            name: spec.name.clone(),
            runtime: PluginRuntime::Node,
            entry_point: entry,
            version: read_node_version(&pkg_dir),
        });
    }

    let pkg_json = pkg_dir.join("package.json");
    if pkg_json.is_file() {
        if let Ok(content) = crate::core::volkiwithstds::fs::read_to_string(&pkg_json) {
            let map = extract_top_level(&content);
            if let Some(main) = map.get(&String::from("main")).and_then(|v| v.as_str()) {
                let main_path = pkg_dir.join(main);
                if main_path.is_file() {
                    return Ok(ResolvedPlugin {
                        name: spec.name.clone(),
                        runtime: PluginRuntime::Node,
                        entry_point: main_path,
                        version: map
                            .get(&String::from("version"))
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    });
                }
            }
        }
    }

    let index = pkg_dir.join("index.js");
    if index.is_file() {
        return Ok(ResolvedPlugin {
            name: spec.name.clone(),
            runtime: PluginRuntime::Node,
            entry_point: index,
            version: read_node_version(&pkg_dir),
        });
    }

    Err(PluginError::NotFound(spec.name.clone()))
}

fn resolve_python(spec: &PluginSpec, project_dir: &Path) -> Result<ResolvedPlugin, PluginError> {
    let module_name = spec.name.replace("-", "_");
    let venv_lib = project_dir.join(".venv").join("lib");

    if venv_lib.is_dir() {
        if let Ok(entries) = crate::core::volkiwithstds::fs::read_dir(&venv_lib) {
            for entry in entries.flatten() {
                let site_pkg = entry.path().join("site-packages").join(&module_name);
                let plugin_file = site_pkg.join("volki_plugin.py");
                if plugin_file.is_file() {
                    return Ok(ResolvedPlugin {
                        name: spec.name.clone(),
                        runtime: PluginRuntime::Python,
                        entry_point: plugin_file,
                        version: None,
                    });
                }
            }
        }
    }

    Err(PluginError::NotFound(spec.name.clone()))
}

fn read_node_version(pkg_dir: &Path) -> Option<String> {
    let pkg_json = pkg_dir.join("package.json");
    let content = crate::core::volkiwithstds::fs::read_to_string(&pkg_json).ok()?;
    let map = extract_top_level(&content);
    map.get(&String::from("version"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::collections::Vec;
    use crate::core::volkiwithstds::fs;
    use crate::core::volkiwithstds::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        let dir = crate::core::volkiwithstds::env::temp_dir().join(&crate::vformat!(
            "volki_resolver_{}_{}",
            crate::core::volkiwithstds::process::id(),
            name
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn resolve_node_volki_plugin_js() {
        let dir = tmp("node_entry");
        let pkg = dir.join("node_modules/my-plugin");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(&pkg.join("volki-plugin.js"), "// plugin".as_bytes()).unwrap();

        let spec = PluginSpec {
            name: String::from("my-plugin"),
            runtime: None,
            options: Vec::new(),
        };
        let resolved = resolve(&spec, &dir).unwrap();
        assert_eq!(resolved.runtime, PluginRuntime::Node);
        assert!(resolved.entry_point.as_str().ends_with("volki-plugin.js"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_node_package_json_main() {
        let dir = tmp("node_main");
        let pkg = dir.join("node_modules/my-plugin");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(
            &pkg.join("package.json"),
            r#"{"main":"lib/entry.js","version":"1.0.0"}"#.as_bytes(),
        )
        .unwrap();
        fs::create_dir_all(&pkg.join("lib")).unwrap();
        fs::write(&pkg.join("lib/entry.js"), "// entry".as_bytes()).unwrap();

        let spec = PluginSpec {
            name: String::from("my-plugin"),
            runtime: None,
            options: Vec::new(),
        };
        let resolved = resolve(&spec, &dir).unwrap();
        assert!(resolved.entry_point.as_str().ends_with("lib/entry.js"));
        assert_eq!(resolved.version.as_deref(), Some("1.0.0"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_node_index_fallback() {
        let dir = tmp("node_index");
        let pkg = dir.join("node_modules/my-plugin");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(&pkg.join("index.js"), "// index".as_bytes()).unwrap();

        let spec = PluginSpec {
            name: String::from("my-plugin"),
            runtime: None,
            options: Vec::new(),
        };
        let resolved = resolve(&spec, &dir).unwrap();
        assert!(resolved.entry_point.as_str().ends_with("index.js"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_not_found() {
        let dir = tmp("not_found");
        let spec = PluginSpec {
            name: String::from("nonexistent"),
            runtime: None,
            options: Vec::new(),
        };
        assert!(matches!(
            resolve(&spec, &dir),
            Err(PluginError::NotFound(_))
        ));
        cleanup(&dir);
    }

    #[test]
    fn resolve_python_plugin() {
        let dir = tmp("python_entry");
        let site = dir.join(".venv/lib/python3.11/site-packages/my_plugin");
        fs::create_dir_all(&site).unwrap();
        fs::write(&site.join("volki_plugin.py"), "# plugin".as_bytes()).unwrap();

        let spec = PluginSpec {
            name: String::from("my-plugin"),
            runtime: None,
            options: Vec::new(),
        };
        let resolved = resolve(&spec, &dir).unwrap();
        assert_eq!(resolved.runtime, PluginRuntime::Python);
        assert!(resolved.entry_point.as_str().ends_with("volki_plugin.py"));
        cleanup(&dir);
    }
}
