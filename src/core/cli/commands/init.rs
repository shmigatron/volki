use crate::core::volkiwithstds::path::Path;

use crate::veprintln;

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
        let dir = args.positional().first().map(|s| s.as_str()).unwrap_or(".");

        let dir_path = Path::new(dir);
        log_debug!("init target: {}", dir_path.as_str());

        let projects = detector::detect(dir_path)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("{e}")))?;

        let path = VolkiConfig::init(dir_path, &projects)
            .map_err(|e| CliError::InvalidUsage(crate::vformat!("{e}")))?;

        output::print_item(
            &style::green(style::CHECK),
            &crate::vformat!("created {}", path.as_str()),
        );

        if let Some(project) = projects.first() {
            output::print_item(
                &style::dim(style::ARROW),
                &crate::vformat!(
                    "ecosystem: {}",
                    style::bold(&crate::vformat!("{}", project.ecosystem))
                ),
            );
            output::print_item(
                &style::dim(style::ARROW),
                &crate::vformat!(
                    "manager: {}",
                    style::bold(&crate::vformat!("{}", project.manager))
                ),
            );
            if let Some(ref fw) = project.framework {
                output::print_item(
                    &style::dim(style::ARROW),
                    &crate::vformat!("framework: {}", style::bold(&crate::vformat!("{fw}"))),
                );
            }
        }

        veprintln!();
        output::print_hint("run volki status to check your project");
        veprintln!();
        Ok(())
    }
}
