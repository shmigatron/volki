use crate::core::utils::log::{self as logger, LogLevel};
use crate::core::volkiwithstds::collections::{Box, String, Vec};
use crate::log_debug;
use crate::veprintln;

use super::command::Command;
use super::error::CliError;
use super::help;
use super::output;
use super::parser::{ParsedArgs, RawArgs};
use super::style;

pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        CommandRegistry {
            commands: Vec::new(),
        }
    }

    pub fn register(&mut self, cmd: Box<dyn Command>) {
        self.commands.push(cmd);
    }

    pub fn run(&self) -> Result<(), CliError> {
        let raw = RawArgs::from_env();

        // Detect --no-color early (before any output)
        if raw.tokens.iter().any(|t| t == "--no-color") {
            style::disable_color();
        }

        // Detect --verbose or VOLKI_LOG env for log level
        if raw.tokens.iter().any(|t| t == "--verbose") {
            logger::set_level(LogLevel::Debug);
        } else if let Some(val) = crate::core::volkiwithstds::env::var("VOLKI_LOG") {
            match val.as_str() {
                "debug" => logger::set_level(LogLevel::Debug),
                "info" => logger::set_level(LogLevel::Info),
                "warn" => logger::set_level(LogLevel::Warn),
                "error" => logger::set_level(LogLevel::Error),
                "off" => logger::set_level(LogLevel::Off),
                _ => {}
            }
        }

        // Top-level --help or no subcommand
        if raw.subcommand.is_none()
            || ParsedArgs::has_help_flag(&raw.tokens) && raw.subcommand.is_none()
        {
            let cmd_refs: Vec<&dyn Command> = self.commands.iter().map(|c| &**c).collect();
            help::print_top_level(&cmd_refs);
            return Ok(());
        }

        // Top-level --version
        if raw.tokens.iter().any(|t| t == "--version") && raw.subcommand.is_none() {
            veprintln!("{}", style::banner());
            return Ok(());
        }

        let sub = raw.subcommand.as_deref().unwrap();

        // Handle top-level flags passed as if they were subcommands
        if sub == "--help" || sub == "-h" {
            let cmd_refs: Vec<&dyn Command> = self.commands.iter().map(|c| &**c).collect();
            help::print_top_level(&cmd_refs);
            return Ok(());
        }
        if sub == "--version" {
            veprintln!("{}", style::banner());
            return Ok(());
        }

        let cmd = self
            .commands
            .iter()
            .find(|c| c.name() == sub)
            .ok_or_else(|| CliError::UnknownCommand(String::from(sub)))?;

        // Per-command --help
        if ParsedArgs::has_help_flag(&raw.tokens) {
            help::print_command_help(&**cmd);
            return Ok(());
        }

        // Config gate: require volki.toml unless the command opts out
        if cmd.requires_config() {
            let cwd = crate::core::volkiwithstds::env::current_dir().map_err(|e| {
                CliError::InvalidUsage(crate::vformat!("cannot determine working directory: {e}"))
            })?;
            if !cwd.join("volki.toml").is_file() {
                return Err(CliError::ConfigRequired);
            }
        }

        output::print_header(cmd.name());
        veprintln!();

        let specs = cmd.options();
        let parsed = ParsedArgs::resolve(&raw, &specs)?;

        Self::validate_required(&specs, &parsed)?;

        log_debug!("executing command '{}'", cmd.name());
        cmd.execute(&parsed)
    }

    fn validate_required(
        specs: &[super::command::OptionSpec],
        parsed: &ParsedArgs,
    ) -> Result<(), CliError> {
        for spec in specs {
            if spec.required && spec.takes_value && parsed.get_option(spec.name).is_none() {
                return Err(CliError::MissingArgument(String::from(spec.name)));
            }
        }
        Ok(())
    }
}
