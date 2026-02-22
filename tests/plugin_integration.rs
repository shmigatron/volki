use std::fs;
use std::path::{Path, PathBuf};

fn tmp(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("volki_plugin_int_{}_{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn has_node() -> bool {
    std::process::Command::new("node")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn plugin_transforms_tokens() {
    if !has_node() {
        eprintln!("skipping plugin_transforms_tokens: node not available");
        return;
    }

    let dir = tmp("transform");

    // Create a Node plugin that uppercases all identifiers
    let plugin_dir = dir.join("node_modules/volki-plugin-upper");
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::write(
        plugin_dir.join("volki-plugin.js"),
        r#"
let input = "";
process.stdin.on("data", (chunk) => (input += chunk));
process.stdin.on("end", () => {
    const req = JSON.parse(input);
    if (req.hook !== "formatter.before_all") {
        process.stdout.write(JSON.stringify({ version: 1, status: "skip" }));
        return;
    }
    const tokens = req.data.tokens.map((t) => {
        if (t.kind === "Identifier") {
            return { ...t, text: t.text.toUpperCase() };
        }
        return t;
    });
    process.stdout.write(JSON.stringify({ version: 1, status: "ok", data: { tokens } }));
});
"#,
    )
    .unwrap();

    // Create volki.toml
    fs::write(
        dir.join("volki.toml"),
        "[volki]\n\n[plugins]\nlist = [\"volki-plugin-upper\"]\n",
    )
    .unwrap();

    // Create a JS file to format
    fs::write(dir.join("test.js"), "const hello = 1\n").unwrap();

    // Run the formatter with plugin support
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_volki"))
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stdout: {stdout}");
    eprintln!("stderr: {stderr}");

    // Read the formatted file — identifiers should be uppercased
    let result = fs::read_to_string(dir.join("test.js")).unwrap();
    assert!(
        result.contains("CONST") && result.contains("HELLO"),
        "expected uppercased identifiers, got: {result}"
    );

    cleanup(&dir);
}

#[test]
fn plugin_skip_is_noop() {
    if !has_node() {
        eprintln!("skipping plugin_skip_is_noop: node not available");
        return;
    }

    let dir = tmp("skip");

    let plugin_dir = dir.join("node_modules/volki-plugin-noop");
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::write(
        plugin_dir.join("volki-plugin.js"),
        r#"
let input = "";
process.stdin.on("data", (chunk) => (input += chunk));
process.stdin.on("end", () => {
    process.stdout.write(JSON.stringify({ version: 1, status: "skip" }));
});
"#,
    )
    .unwrap();

    fs::write(
        dir.join("volki.toml"),
        "[volki]\n\n[plugins]\nlist = [\"volki-plugin-noop\"]\n",
    )
    .unwrap();

    // Pre-formatted input (semi + trailing newline)
    let source = "const x = 1;\n";
    fs::write(dir.join("test.js"), source).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_volki"))
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!("stdout: {stdout}");

    let result = fs::read_to_string(dir.join("test.js")).unwrap();
    assert_eq!(result, source, "skip plugin should not change anything");

    cleanup(&dir);
}

#[test]
fn missing_plugin_warns_but_continues() {
    let dir = tmp("missing");

    fs::write(
        dir.join("volki.toml"),
        "[volki]\n\n[plugins]\nlist = [\"volki-plugin-nonexistent\"]\n",
    )
    .unwrap();

    let source = "const x = 1\n";
    fs::write(dir.join("test.js"), source).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_volki"))
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();

    // Should still succeed (missing plugin is non-fatal)
    assert!(
        output.status.success(),
        "format should succeed even with missing plugin"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning") || stderr.contains("not found"),
        "expected warning about missing plugin, got stderr: {stderr}"
    );

    // File should still be formatted normally
    let result = fs::read_to_string(dir.join("test.js")).unwrap();
    assert!(
        result.contains("const x = 1;"),
        "file should be formatted: {result}"
    );

    cleanup(&dir);
}

#[test]
fn plugin_receives_options() {
    if !has_node() {
        eprintln!("skipping plugin_receives_options: node not available");
        return;
    }

    let dir = tmp("options");

    let plugin_dir = dir.join("node_modules/volki-plugin-opts");
    fs::create_dir_all(&plugin_dir).unwrap();
    // Plugin that prepends a comment with the option value
    fs::write(
        plugin_dir.join("volki-plugin.js"),
        r#"
let input = "";
process.stdin.on("data", (chunk) => (input += chunk));
process.stdin.on("end", () => {
    const req = JSON.parse(input);
    if (req.hook !== "formatter.before_all") {
        process.stdout.write(JSON.stringify({ version: 1, status: "skip" }));
        return;
    }
    const prefix = req.plugin_options.prefix || "DEFAULT";
    const comment = { kind: "LineComment", text: "// " + prefix, line: 0, col: 0 };
    const newline = { kind: "Newline", text: "\n", line: 0, col: 0 };
    const tokens = [comment, newline, ...req.data.tokens];
    process.stdout.write(JSON.stringify({ version: 1, status: "ok", data: { tokens } }));
});
"#,
    )
    .unwrap();

    fs::write(
        dir.join("volki.toml"),
        "[volki]\n\n[plugins]\nlist = [\"volki-plugin-opts\"]\n\n[plugins.volki-plugin-opts]\nprefix = \"CUSTOM_VALUE\"\n",
    )
    .unwrap();

    fs::write(dir.join("test.js"), "const x = 1;\n").unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_volki"))
        .current_dir(&dir)
        .args(["format", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!("stdout: {stdout}");

    let result = fs::read_to_string(dir.join("test.js")).unwrap();
    assert!(
        result.contains("// CUSTOM_VALUE"),
        "expected plugin option value in output, got: {result}"
    );

    cleanup(&dir);
}
