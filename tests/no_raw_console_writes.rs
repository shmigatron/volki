use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn no_raw_console_writes_outside_core() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();
    scan_dir(&root, &mut violations);

    assert!(
        violations.is_empty(),
        "raw console writes are forbidden outside src/core; use core::cli logging helpers\n{}",
        violations.join("\n")
    );
}

fn scan_dir(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(path.as_path(), out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let rel = rel_path(path.as_path());
        if rel.starts_with("src/core/") {
            continue;
        }

        let Ok(content) = fs::read_to_string(path.as_path()) else { continue };
        check_forbidden(path.as_path(), content.as_str(), out);
    }
}

fn check_forbidden(path: &Path, content: &str, out: &mut Vec<String>) {
    let forbidden = [
        "core::volkiwithstds::io::stderr()",
        "core::volkiwithstds::io::stdout()",
        "crate::core::volkiwithstds::io::stderr()",
        "crate::core::volkiwithstds::io::stdout()",
        "volki::core::volkiwithstds::io::stderr()",
        "volki::core::volkiwithstds::io::stdout()",
    ];

    for (line_no, line) in content.lines().enumerate() {
        for needle in &forbidden {
            if line.contains(needle) {
                out.push(format!(
                    "{}:{}: contains forbidden console call `{}`",
                    rel_path(path),
                    line_no + 1,
                    needle
                ));
            }
        }
    }
}

fn rel_path(path: &Path) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.strip_prefix(root)
        .ok()
        .and_then(|p| p.to_str())
        .unwrap_or("<unknown>")
        .replace('\\', "/")
}
