use std::path::Path;

use crate::core::cli::command::Command;
use crate::core::cli::error::CliError;
use crate::core::cli::output;
use crate::core::cli::parser::ParsedArgs;
use crate::core::cli::style;
use crate::core::config::VolkiConfig;
use crate::core::package::detect::detector;
use crate::log_debug;

pub struct InitCommand;

impl Command for InitCommand {
    fn name(&self) -> &str {
        "init"
    }

    fn description(&self) -> &str {
        "Initialize a new volki project"
    }

    fn long_description(&self) -> &str {
        "Create a volki.toml config file in the target directory."
    }

    fn requires_config(&self) -> bool {
        false
    }

    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError> {
        let dir = args
            .positional()
            .first()
            .map(|s| s.as_str())
            .unwrap_or(".");

        let dir_path = Path::new(dir);
        log_debug!("init target: {}", dir_path.display());

        let projects = detector::detect(dir_path)
            .map_err(|e| CliError::InvalidUsage(e.to_string()))?;

        let path = VolkiConfig::init(dir_path, &projects)
            .map_err(|e| CliError::InvalidUsage(e.to_string()))?;

        output::print_item(
            &style::green(style::CHECK),
            &format!("created {}", path.display()),
        );

        if let Some(project) = projects.first() {
            output::print_item(
                &style::dim(style::ARROW),
                &format!("ecosystem: {}", style::bold(&project.ecosystem.to_string())),
            );
            output::print_item(
                &style::dim(style::ARROW),
                &format!("manager: {}", style::bold(&project.manager.to_string())),
            );
            if let Some(ref fw) = project.framework {
                output::print_item(
                    &style::dim(style::ARROW),
                    &format!("framework: {}", style::bold(&fw.to_string())),
                );
            }
        }

        eprintln!();
        output::print_hint("run volki status to check your project");
        eprintln!();
        Ok(())
    }
}
