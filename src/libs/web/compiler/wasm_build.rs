//! WASM Build — compiles generated `_client.rs` files to `.wasm` using `rustc`.
//!
//! Shells out to:
//! ```text
//! rustc --target wasm32-unknown-unknown --crate-type cdylib -O --edition 2024 -o output.wasm input.rs
//! ```

use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::path::Path;
use crate::core::volkiwithstds::process::Command;

use super::CompileError;

/// Compile a generated `_client.rs` file to a `.wasm` binary.
///
/// - `client_rs` — path to the generated Rust source (e.g., `dist/app/page_client.rs`)
/// - `output_wasm` — path for the output `.wasm` file (e.g., `public/wasm/page_client.wasm`)
///
/// Returns `Ok(())` on success, or a `CompileError` with a helpful message if
/// the wasm target is missing or `rustc` fails.
pub fn compile_wasm(client_rs: &Path, output_wasm: &Path) -> Result<(), CompileError> {
    // Ensure output directory exists
    if let Some(parent) = output_wasm.parent() {
        crate::core::volkiwithstds::fs::create_dir_all(parent).map_err(|e| CompileError {
            file: output_wasm.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to create wasm output directory: {}", e),
        })?;
    }

    let output = Command::new("rustc")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--crate-type")
        .arg("cdylib")
        .arg("-O")
        .arg("--edition")
        .arg("2024")
        .arg("-o")
        .arg(output_wasm.as_str())
        .arg(client_rs.as_str())
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(())
            } else {
                let stderr = from_utf8_lossy(&result.stderr);
                // Check for common "target not installed" message
                if stderr.as_str().contains("wasm32-unknown-unknown") && stderr.as_str().contains("target") {
                    Err(CompileError {
                        file: client_rs.to_path_buf(),
                        line: 0,
                        col: 0,
                        message: String::from(
                            "wasm32-unknown-unknown target not installed.\n\n  \
                             Install it with: rustup target add wasm32-unknown-unknown"
                        ),
                    })
                } else {
                    Err(CompileError {
                        file: client_rs.to_path_buf(),
                        line: 0,
                        col: 0,
                        message: crate::vformat!("rustc failed:\n{}", stderr),
                    })
                }
            }
        }
        Err(e) => {
            Err(CompileError {
                file: client_rs.to_path_buf(),
                line: 0,
                col: 0,
                message: crate::vformat!(
                    "failed to run rustc: {}\n\n  \
                     Make sure rustc is installed and in your PATH.",
                    e
                ),
            })
        }
    }
}

/// Check if the wasm32-unknown-unknown target is available.
pub fn check_wasm_target() -> bool {
    let output = Command::new("rustup")
        .arg("target")
        .arg("list")
        .arg("--installed")
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout = from_utf8_lossy(&result.stdout);
                stdout.as_str().contains("wasm32-unknown-unknown")
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Helper to convert from UTF-8 bytes to a String, replacing invalid chars.
fn from_utf8_lossy(bytes: &[u8]) -> String {
    // Simple lossy conversion
    let s = core::str::from_utf8(bytes).unwrap_or("<invalid utf8>");
    String::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_wasm_missing_file() {
        let client_rs = Path::new("/tmp/__volki_nonexistent_test_12345.rs");
        let output = Path::new("/tmp/__volki_nonexistent_test_12345.wasm");
        let result = compile_wasm(client_rs, output);
        // Should fail because the file doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_check_wasm_target_runs() {
        // Just verify it doesn't panic — result depends on environment
        let _ = check_wasm_target();
    }
}
