use std::io;
use std::path::{Path, PathBuf};

const JS_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs"];

const SKIP_DIRS: &[&str] = &[
    "node_modules", ".git", ".hg", ".svn", "dist", "build",
    "coverage", ".next", ".nuxt", ".cache", "target",
];

pub struct WalkConfig {
    pub extensions: Vec<String>,
    pub skip_dirs: Vec<String>,
}

impl Default for WalkConfig {
    fn default() -> Self {
        Self {
            extensions: JS_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
            skip_dirs: SKIP_DIRS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

pub fn walk_files(root: &Path, config: &WalkConfig) -> Result<Vec<PathBuf>, io::Error> {
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }

    let mut results = Vec::new();
    walk_recursive(root, config, &mut results)?;
    results.sort();
    Ok(results)
}

fn walk_recursive(dir: &Path, config: &WalkConfig, results: &mut Vec<PathBuf>) -> Result<(), io::Error> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Ok(()),
        Err(e) => return Err(e),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            let dir_name = entry.file_name();
            let name = dir_name.to_string_lossy();
            if !config.skip_dirs.iter().any(|s| s == name.as_ref()) {
                walk_recursive(&path, config, results)?;
            }
        } else if file_type.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy();
                if config.extensions.iter().any(|e| e == ext.as_ref()) {
                    results.push(path);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn walk_single_file() {
        let dir = std::env::temp_dir().join("volki_walk_test_single");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("test.js");
        fs::write(&file, "// test").unwrap();

        let config = WalkConfig::default();
        let files = walk_files(&file, &config).unwrap();
        assert_eq!(files, vec![file.clone()]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn walk_dir_finds_js_files() {
        let dir = std::env::temp_dir().join("volki_walk_test_dir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("a.js"), "").unwrap();
        fs::write(dir.join("b.ts"), "").unwrap();
        fs::write(dir.join("c.txt"), "").unwrap();
        fs::write(dir.join("sub/d.tsx"), "").unwrap();

        let config = WalkConfig::default();
        let files = walk_files(&dir, &config).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.iter().all(|f| {
            let ext = f.extension().unwrap().to_string_lossy();
            JS_EXTENSIONS.contains(&ext.as_ref())
        }));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn walk_skips_node_modules() {
        let dir = std::env::temp_dir().join("volki_walk_test_skip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("node_modules")).unwrap();
        fs::write(dir.join("node_modules/dep.js"), "").unwrap();
        fs::write(dir.join("app.js"), "").unwrap();

        let config = WalkConfig::default();
        let files = walk_files(&dir, &config).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("app.js"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn walk_empty_dir() {
        let dir = std::env::temp_dir().join("volki_walk_test_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let config = WalkConfig::default();
        let files = walk_files(&dir, &config).unwrap();
        assert!(files.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn results_are_sorted() {
        let dir = std::env::temp_dir().join("volki_walk_test_sort");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("c.js"), "").unwrap();
        fs::write(dir.join("a.js"), "").unwrap();
        fs::write(dir.join("b.js"), "").unwrap();

        let config = WalkConfig::default();
        let files = walk_files(&dir, &config).unwrap();
        let names: Vec<String> = files.iter().map(|f| f.file_name().unwrap().to_string_lossy().to_string()).collect();
        assert_eq!(names, vec!["a.js", "b.js", "c.js"]);

        let _ = fs::remove_dir_all(&dir);
    }
}
