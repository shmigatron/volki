//! db:web — launch the web-based database editor.

use crate::core::cli::command::{Command, OptionSpec};
use crate::core::cli::error::CliError;
use crate::core::cli::parser::ParsedArgs;
use crate::core::volkiwithstds::collections::Vec;
use crate::core::volkiwithstds::path::{Path, PathBuf};
use crate::libs::web::cli::dynamic_runtime::{run_dynamic_runtime, DynamicRuntimeOptions, EmptyRoutesPolicy};

pub struct WebEditorCommand;

impl Command for WebEditorCommand {
    fn name(&self) -> &str {
        "db:web"
    }

    fn description(&self) -> &str {
        "Launch web-based database table editor"
    }

    fn long_description(&self) -> &str {
        "Starts the DB web editor through Volki web runtime (dynamic .volki routes + web server)."
    }

    fn options(&self) -> Vec<OptionSpec> {
        let mut opts = Vec::new();
        opts.push(OptionSpec {
            name: "port",
            description: "Port to listen on",
            takes_value: true,
            required: false,
            default_value: Some("4000"),
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
        false
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let host = args.get_option("host").unwrap_or("127.0.0.1");
        let port_str = args.get_option("port").unwrap_or("4000");
        let port: u16 = port_str.parse().map_err(|_| {
            CliError::InvalidUsage(crate::core::volkiwithstds::collections::String::from(
                "invalid port number",
            ))
        })?;

        let source_dir = find_web_editor_root().ok_or_else(|| {
            CliError::InvalidUsage(crate::core::volkiwithstds::collections::String::from(
                "could not locate src/libs/db/web_editor from current directory",
            ))
        })?;

        run_dynamic_runtime(DynamicRuntimeOptions {
            host,
            port,
            source_dir: source_dir.as_path(),
            title: "volki db editor",
            scan_prefix: Some("db:web"),
            show_routes: true,
            show_summary: true,
            show_source_dir: true,
            empty_routes: EmptyRoutesPolicy::Error(
                "no page.volki routes found under src/libs/db/web_editor/app",
            ),
        })
    }
}

fn find_web_editor_root() -> Option<PathBuf> {
    let cwd = crate::core::volkiwithstds::env::current_dir().ok()?;
    let mut current = cwd;
    loop {
        let candidate = current.as_path().join("src/libs/db/web_editor");
        if candidate.as_path().join("app/page.volki").as_path().exists() {
            return Some(candidate);
        }
        let parent = current.as_path().parent()?;
        if parent == current.as_path() || parent == Path::new("") {
            return None;
        }
        current = parent.to_path_buf();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_editor_name() {
        assert_eq!(WebEditorCommand.name(), "db:web");
    }

    #[test]
    fn test_web_editor_does_not_require_config() {
        assert!(!WebEditorCommand.requires_config());
    }
}
