pub mod build_cmd;
pub mod dev_cmd;
pub mod dynamic_runtime;
pub mod hub_cmd;
pub mod start_cmd;

pub use build_cmd::WebBuildCommand;
pub use dev_cmd::WebDevCommand;
pub use hub_cmd::WebHubCommand;
pub use start_cmd::WebStartCommand;

use crate::core::cli::error::CliError;
use crate::core::volkiwithstds::collections::String;

/// Verify that `volki.toml` contains a `[web]` section.
/// Called at the start of every web subcommand's `execute`.
pub fn require_web_section() -> Result<(), CliError> {
    let cwd = crate::core::volkiwithstds::env::current_dir().map_err(|e| {
        CliError::InvalidUsage(crate::vformat!("cannot determine working directory: {e}"))
    })?;
    let config_path = cwd.join("volki.toml");
    let content = crate::core::volkiwithstds::fs::read_to_string(config_path.as_path())
        .map_err(|_| CliError::ConfigRequired)?;
    let table = crate::core::config::parser::parse(content.as_str())
        .map_err(|e| CliError::InvalidUsage(crate::vformat!("failed to parse volki.toml: {e}")))?;
    if !table.has_section("web") {
        return Err(CliError::ConfigSectionRequired(String::from("web")));
    }
    Ok(())
}
