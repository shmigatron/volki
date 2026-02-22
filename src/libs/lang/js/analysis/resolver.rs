use crate::core::volkiwithstds::path::{Path, PathBuf};

const EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mjs", "cjs"];
const INDEX_FILES: &[&str] = &["index.ts", "index.tsx", "index.js", "index.jsx"];

/// Resolve a relative import specifier to an absolute file path.
/// Returns None for bare specifiers (npm packages like "react").
pub fn resolve_import(source: &str, from_file: &Path) -> Option<PathBuf> {
    if !source.starts_with(".") && !source.starts_with("/") {
        return None;
    }

    let base_dir = from_file.parent()?;
    let target = base_dir.join(source);

    // Try exact path first
    if target.is_file() {
        return Some(normalize(&target));
    }

    // Try adding extensions
    for ext in EXTENSIONS {
        let with_ext = target.with_extension(ext);
        if with_ext.is_file() {
            return Some(normalize(&with_ext));
        }
    }

    // Try as directory with index file
    if target.is_dir() {
        for index in INDEX_FILES {
            let index_path = target.join(index);
            if index_path.is_file() {
                return Some(normalize(&index_path));
            }
        }
    }

    None
}

fn normalize(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::fs;

    fn make_temp_dir(name: &str) -> PathBuf {
        let dir = crate::core::volkiwithstds::env::temp_dir().join(&crate::vformat!(
            "volki_resolver_{}_{name}",
            crate::core::volkiwithstds::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bare_specifier_returns_none() {
        let dir = make_temp_dir("bare");
        let from = dir.join("index.ts");
        fs::write(&from, "").unwrap();
        assert!(resolve_import("react", &from).is_none());
        cleanup(&dir);
    }

    #[test]
    fn resolve_with_extension() {
        let dir = make_temp_dir("ext");
        let from = dir.join("index.ts");
        let target = dir.join("utils.ts");
        fs::write(&from, "").unwrap();
        fs::write(&target, "").unwrap();

        let resolved = resolve_import("./utils", &from);
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        assert!(resolved.ends_with("utils.ts"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_exact_file() {
        let dir = make_temp_dir("exact");
        let from = dir.join("index.ts");
        let target = dir.join("data.json");
        fs::write(&from, "").unwrap();
        fs::write(&target, "{}").unwrap();

        let resolved = resolve_import("./data.json", &from);
        assert!(resolved.is_some());
        cleanup(&dir);
    }

    #[test]
    fn resolve_directory_index() {
        let dir = make_temp_dir("diridx");
        let from = dir.join("index.ts");
        let sub = dir.join("components");
        fs::create_dir_all(&sub).unwrap();
        fs::write(&from, "").unwrap();
        fs::write(sub.join("index.ts"), "").unwrap();

        let resolved = resolve_import("./components", &from);
        assert!(resolved.is_some());
        let resolved = resolved.unwrap();
        assert!(resolved.contains("index.ts"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_nonexistent_returns_none() {
        let dir = make_temp_dir("nonex");
        let from = dir.join("index.ts");
        fs::write(&from, "").unwrap();

        assert!(resolve_import("./nonexistent", &from).is_none());
        cleanup(&dir);
    }

    #[test]
    fn resolve_js_extension() {
        let dir = make_temp_dir("jsext");
        let from = dir.join("index.ts");
        let target = dir.join("helper.js");
        fs::write(&from, "").unwrap();
        fs::write(&target, "").unwrap();

        let resolved = resolve_import("./helper", &from);
        assert!(resolved.is_some());
        cleanup(&dir);
    }

    #[test]
    fn resolve_tsx_extension() {
        let dir = make_temp_dir("tsxext");
        let from = dir.join("index.ts");
        let target = dir.join("App.tsx");
        fs::write(&from, "").unwrap();
        fs::write(&target, "").unwrap();

        let resolved = resolve_import("./App", &from);
        assert!(resolved.is_some());
        cleanup(&dir);
    }
}
