use std::fs;
use std::process::Command;

fn volki() -> Command {
    Command::new(env!("CARGO_BIN_EXE_volki"))
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("volki_fmt_integration_{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("volki.toml"), "[volki]\n").unwrap();
    dir
}

#[test]
fn format_no_files_clean_exit() {
    let dir = temp_dir("no_files");
    let output = volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_check_no_files_clean_exit() {
    let dir = temp_dir("check_no_files");
    let output = volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap(), "--check"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_writes_file() {
    let dir = temp_dir("writes");
    let file = dir.join("test.js");
    fs::write(&file, "const x = 'hello'\n").unwrap();

    let output = volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());

    let result = fs::read_to_string(&file).unwrap();
    assert!(
        result.contains("\"hello\""),
        "Expected double quotes, got: {result}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_check_detects_changes() {
    let dir = temp_dir("check_changes");
    let file = dir.join("test.js");
    fs::write(&file, "const x = 'hello'\n").unwrap();

    let output = volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap(), "--check"])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "Check should fail when files need formatting"
    );

    // File should NOT be modified in check mode
    let result = fs::read_to_string(&file).unwrap();
    assert!(
        result.contains("'hello'"),
        "Check mode should not modify files"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_check_passes_when_formatted() {
    let dir = temp_dir("check_pass");
    let file = dir.join("test.js");
    // First format the file
    fs::write(&file, "const x = 'hello'\n").unwrap();
    volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();

    // Then check — should pass
    let output = volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap(), "--check"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Check should pass on already-formatted files"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_inserts_semicolons() {
    let dir = temp_dir("semi");
    let file = dir.join("test.js");
    fs::write(&file, "const x = 1\nconst y = 2\n").unwrap();

    volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let result = fs::read_to_string(&file).unwrap();
    assert!(
        result.contains("const x = 1;"),
        "Expected semicolons, got: {result}"
    );
    assert!(
        result.contains("const y = 2;"),
        "Expected semicolons, got: {result}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_normalizes_indentation() {
    let dir = temp_dir("indent");
    let file = dir.join("test.js");
    fs::write(&file, "if (x) {\nfoo();\n}\n").unwrap();

    volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let result = fs::read_to_string(&file).unwrap();
    assert!(
        result.contains("  foo();"),
        "Expected 2-space indent, got: {result}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_single_file_path() {
    let dir = temp_dir("single_file");
    let file = dir.join("test.js");
    fs::write(&file, "const x = 'hi'\n").unwrap();

    let output = volki()
        .current_dir(&dir)
        .args(["format", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("\"hi\""));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_skips_non_js_files() {
    let dir = temp_dir("skip_non_js");
    let txt_file = dir.join("readme.txt");
    let js_file = dir.join("app.js");
    fs::write(&txt_file, "hello world").unwrap();
    fs::write(&js_file, "const x = 'test'\n").unwrap();

    volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let txt_content = fs::read_to_string(&txt_file).unwrap();
    assert_eq!(
        txt_content, "hello world",
        "Non-JS files should not be modified"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn format_typescript_file() {
    let dir = temp_dir("typescript");
    let file = dir.join("test.ts");
    fs::write(&file, "const x = 'hello'\n").unwrap();

    let output = volki()
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("\"hello\""));
    let _ = fs::remove_dir_all(&dir);
}
