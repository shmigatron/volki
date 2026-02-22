use crate::core::volkiwithstds::collections::ToString;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fs::FileType;
use crate::core::volkiwithstds::io;
use crate::core::volkiwithstds::path::{Path, PathBuf};
use crate::vvec;

const JS_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs"];

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    ".hg",
    ".svn",
    "dist",
    "build",
    "coverage",
    ".next",
    ".nuxt",
    ".cache",
    "target",
];

pub struct WalkConfig {
    pub extensions: Vec<String>,
    pub skip_dirs: Vec<String>,
}

impl Default for WalkConfig {
    fn default() -> Self {
        Self {
            extensions: JS_EXTENSIONS.iter().map(|s| s.to_vstring()).collect(),
            skip_dirs: SKIP_DIRS.iter().map(|s| s.to_vstring()).collect(),
        }
    }
}

pub fn walk_files(root: &Path, config: &WalkConfig) -> Result<Vec<PathBuf>, io::IoError> {
    if root.is_file() {
        return Ok(vvec![root.to_path_buf()]);
    }

    let mut results = Vec::new();
    walk_recursive(root, config, &mut results)?;
    results.sort();
    Ok(results)
}

fn walk_recursive(
    dir: &Path,
    config: &WalkConfig,
    results: &mut Vec<PathBuf>,
) -> Result<(), io::IoError> {
    let entries = match crate::core::volkiwithstds::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::IoErrorKind::PermissionDenied => return Ok(()),
        Err(e) => return Err(e),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type();

        if file_type == FileType::Directory {
            let dir_name = entry.file_name();
            let name = dir_name;
            if !config.skip_dirs.iter().any(|s| s.as_str() == name) {
                walk_recursive(&path, config, results)?;
            }
        } else if file_type == FileType::File {
            if let Some(ext) = path.extension() {
                let ext = ext;
                if config.extensions.iter().any(|e| e.as_str() == ext) {
                    results.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::fs;

    #[test]
    fn walk_single_file() {
        let dir = crate::core::volkiwithstds::env::temp_dir().join("volki_walk_test_single");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("test.js");
        fs::write(&file, "// test").unwrap();

        let config = WalkConfig::default();
        let files = walk_files(&file, &config).unwrap();
        assert_eq!(files, vvec![file.clone()]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn walk_dir_finds_js_files() {
        let dir = crate::core::volkiwithstds::env::temp_dir().join("volki_walk_test_dir");
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
            let ext = f.extension().unwrap();
            JS_EXTENSIONS.contains(&ext.as_ref())
        }));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn walk_skips_node_modules() {
        let dir = crate::core::volkiwithstds::env::temp_dir().join("volki_walk_test_skip");
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
        let dir = crate::core::volkiwithstds::env::temp_dir().join("volki_walk_test_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let config = WalkConfig::default();
        let files = walk_files(&dir, &config).unwrap();
        assert!(files.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn results_are_sorted() {
        let dir = crate::core::volkiwithstds::env::temp_dir().join("volki_walk_test_sort");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("c.js"), "").unwrap();
        fs::write(dir.join("a.js"), "").unwrap();
        fs::write(dir.join("b.js"), "").unwrap();

        let config = WalkConfig::default();
        let files = walk_files(&dir, &config).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_vstring())
            .collect();
        assert_eq!(
            names,
            vvec![
                crate::vstr!("a.js"),
                crate::vstr!("b.js"),
                crate::vstr!("c.js")
            ]
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
