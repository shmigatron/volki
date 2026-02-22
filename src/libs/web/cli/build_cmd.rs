//! web:build — compile .volki files to Rust.

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::core::volkiwithstds::collections::Vec;
use crate::core::volkiwithstds::path::Path;
use crate::veprintln;

pub struct WebBuildCommand;

impl Command for WebBuildCommand {
    fn name(&self) -> &str {
        "web:build"
    }

    fn description(&self) -> &str {
        "Compile .volki files to Rust"
    }

    fn long_description(&self) -> &str {
        "Walks a directory for .volki files and compiles each one into a .rs file with equivalent Rust builder calls."
    }

    fn options(&self) -> Vec<OptionSpec> {
        let mut opts = Vec::new();
        opts.push(OptionSpec {
            name: "path",
            description: "Source directory to scan",
            takes_value: true,
            required: false,
            default_value: Some("."),
            short: None,
        });
        opts
    }

    fn requires_config(&self) -> bool {
        true
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        super::require_web_section()?;
        let dir = args.get_option("path").unwrap_or(".");
        let path = Path::new(dir);

        // Read [web] config from volki.toml
        let entrypoint = crate::libs::web::compiler::read_entrypoint_config(path);
        let dist = crate::libs::web::compiler::read_dist_config(path);

        let source_dir = if entrypoint.as_str() == "." {
            path.to_path_buf()
        } else {
            path.join(entrypoint.as_str())
        };

        veprintln!();
        veprintln!("  {} {}", style::dim("entrypoint:"), entrypoint);
        veprintln!("  {} {}", style::dim("output:"), dist);

        match crate::libs::web::compiler::compile_dir(source_dir.as_path(), dist.as_str()) {
            Ok(results) => {
                if results.is_empty() {
                    veprintln!("  {} no .volki files found", style::dim("result:"));
                } else {
                    let client_count = results.iter().filter(|r| r.client.is_some()).count();
                    veprintln!(
                        "  {} compiled {} file{}",
                        style::dim("result:"),
                        results.len(),
                        if results.len() == 1 { "" } else { "s" },
                    );
                    for result in &results {
                        veprintln!(
                            "    {} -> {}",
                            style::dim(result.source_path.display()),
                            result.output_path.display(),
                        );
                        for warning in result.warnings.iter() {
                            crate::core::cli::print_warn_trace(
                                warning.file.display(),
                                warning.line,
                                warning.col,
                                warning.message.as_str(),
                            );
                        }
                    }
                    if client_count > 0 {
                        veprintln!(
                            "  {} {} file{} with client-side WASM",
                            style::dim("client:"),
                            client_count,
                            if client_count == 1 { "" } else { "s" },
                        );
                    }
                }
                veprintln!();
                Ok(())
            }
            Err(e) => {
                Err(CliError::InvalidUsage(crate::vformat!(
                    "compilation failed\n\n  {}:{}:{}: {}",
                    e.file,
                    e.line,
                    e.col,
                    e.message,
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_name() {
        assert_eq!(WebBuildCommand.name(), "web:build");
    }

    #[test]
    fn test_build_requires_config() {
        assert!(WebBuildCommand.requires_config());
    }
}
