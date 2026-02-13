use std::process::Command;

fn volki() -> Command {
    Command::new(env!("CARGO_BIN_EXE_volki"))
}

#[test]
fn no_args_shows_help() {
    let output = volki().output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.to_lowercase().contains("usage") || combined.to_lowercase().contains("help") || combined.contains("volki"),
        "Expected help text, got: {combined}"
    );
    assert!(output.status.success());
}

#[test]
fn help_flag() {
    let output = volki().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.to_lowercase().contains("usage") || combined.contains("volki") || combined.to_lowercase().contains("help"));
}

#[test]
fn version_flag() {
    let output = volki().arg("--version").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    // Should either print version or show help (depends on implementation)
    assert!(output.status.success() || combined.contains("volki"));
}

#[test]
fn unknown_command_error() {
    let output = volki().arg("nonexistent").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "Expected failure for unknown command");
    assert!(stderr.to_lowercase().contains("unknown") || stderr.to_lowercase().contains("error"));
}

#[test]
fn license_help() {
    let output = volki().args(["license", "--help"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.to_lowercase().contains("license") || combined.to_lowercase().contains("scan"));
}

#[test]
fn init_help() {
    let output = volki().args(["init", "--help"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.to_lowercase().contains("init") || combined.to_lowercase().contains("name"));
}

#[test]
fn init_creates_config() {
    let dir = std::env::temp_dir().join(format!("volki_cli_init_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let output = volki().args(["init", dir.to_str().unwrap()]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(output.status.success(), "init failed: {stderr}");
    assert!(combined.to_lowercase().contains("created"), "Expected 'created', got: {combined}");
    assert!(dir.join("volki.toml").is_file());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn init_fails_if_config_exists() {
    let dir = std::env::temp_dir().join(format!("volki_cli_init_dup_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("volki.toml"), "").unwrap();

    let output = volki().args(["init", dir.to_str().unwrap()]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"), "Expected 'already exists', got: {stderr}");

    let _ = std::fs::remove_dir_all(&dir);
}

fn dir_with_config(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("volki_cli_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("volki.toml"), "[volki]\n").unwrap();
    dir
}

#[test]
fn license_nonexistent_path() {
    let dir = dir_with_config("lic_nopath");
    let output = volki()
        .current_dir(&dir)
        .args(["license", "--path", "/tmp/volki_nonexistent_path_xyz"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn license_unknown_ecosystem() {
    let dir = dir_with_config("lic_unkeco");
    let output = volki()
        .current_dir(&dir)
        .args(["license", "--ecosystem", "cobol"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.to_lowercase().contains("unknown") || stderr.to_lowercase().contains("cobol"),
        "Expected error about unknown ecosystem, got: {stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn config_required_for_commands() {
    let dir = std::env::temp_dir().join(format!("volki_cli_noconfig_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let output = volki()
        .current_dir(&dir)
        .arg("status")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("volki.toml") && stderr.contains("volki init"),
        "Expected config-required error, got: {stderr}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
