//! web:dev — development server that interprets .volki files at runtime.

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::parser::ParsedArgs;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::libs::web::compiler;
use super::dynamic_runtime::{run_dynamic_runtime, DynamicRuntimeOptions, EmptyRoutesPolicy};

pub struct WebDevCommand;

impl Command for WebDevCommand {
    fn name(&self) -> &str {
        "web:dev"
    }

    fn description(&self) -> &str {
        "Start development server (interprets .volki at runtime)"
    }

    fn long_description(&self) -> &str {
        "Starts a development server that reads and interprets .volki files at runtime without a cargo build step. Pages are parsed and served immediately. Fragment functions are resolved, CSS is generated, metadata is extracted, and client interactivity is compiled to WASM at startup. For production builds, use web:build + web:start."
    }

    fn options(&self) -> Vec<OptionSpec> {
        let mut opts = Vec::new();
        opts.push(OptionSpec {
            name: "port",
            description: "Port to listen on",
            takes_value: true,
            required: false,
            default_value: Some("3000"),
            short: Some('p'),
        });
        opts.push(OptionSpec {
            name: "host",
            description: "Host to bind to",
            takes_value: true,
            required: false,
            default_value: Some("127.0.0.1"),
            short: None,
        });
        opts
    }

    fn requires_config(&self) -> bool {
        true
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        super::require_web_section()?;

        let host = args.get_option("host").unwrap_or("127.0.0.1");
        let port_str = args.get_option("port").unwrap_or("3000");
        let port: u16 = port_str.parse().map_err(|_| {
            CliError::InvalidUsage(String::from("invalid port number"))
        })?;

        // Find project root (where volki.toml is)
        let cwd = crate::core::volkiwithstds::env::current_dir().map_err(|e| {
            CliError::InvalidUsage(crate::vformat!("cannot determine working directory: {e}"))
        })?;

        // Read entrypoint config
        let entrypoint = compiler::read_entrypoint_config(cwd.as_path());
        let source_dir = if entrypoint.as_str() == "." {
            cwd.clone()
        } else {
            cwd.join(entrypoint.as_str())
        };

        run_dynamic_runtime(DynamicRuntimeOptions {
            host,
            port,
            source_dir: source_dir.as_path(),
            title: "volki dev server",
            scan_prefix: Some("dev"),
            show_routes: true,
            show_summary: true,
            show_source_dir: false,
            empty_routes: EmptyRoutesPolicy::WarnAndReturn,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_name() {
        assert_eq!(WebDevCommand.name(), "web:dev");
    }

    #[test]
    fn test_dev_requires_config() {
        assert!(WebDevCommand.requires_config());
    }

    #[test]
    fn test_dev_has_port_option() {
        let opts = WebDevCommand.options();
        assert!(opts.iter().any(|o| o.name == "port"));
    }

    #[test]
    fn test_dev_has_host_option() {
        let opts = WebDevCommand.options();
        assert!(opts.iter().any(|o| o.name == "host"));
    }
}
