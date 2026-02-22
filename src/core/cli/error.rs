use core::fmt;

use crate::core::volkiwithstds::collections::String;

use super::style;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CliError {
    UnknownCommand(String),
    MissingArgument(String),
    UnknownFlag(String),
    MissingValue(String),
    InvalidUsage(String),
    ConfigRequired,
    ConfigSectionRequired(String),
}

impl CliError {
    pub fn hint(&self) -> Option<String> {
        match self {
            CliError::UnknownCommand(_) => Some(crate::vformat!(
                "run {} to see available commands",
                style::bold("volki --help")
            )),
            CliError::MissingArgument(arg) => Some(crate::vformat!(
                "provide a value with {}",
                style::bold(&crate::vformat!("--{arg} <value>"))
            )),
            CliError::UnknownFlag(_) => Some(crate::vformat!(
                "run with {} to see valid options",
                style::bold("--help")
            )),
            CliError::MissingValue(flag) => Some(crate::vformat!(
                "provide a value: {}",
                style::bold(&crate::vformat!("--{flag} <value>"))
            )),
            CliError::InvalidUsage(_) => None,
            CliError::ConfigRequired => Some(crate::vformat!(
                "run {} to initialize your project",
                style::bold("volki init")
            )),
            CliError::ConfigSectionRequired(section) => Some(crate::vformat!(
                "add a [{}] section to your volki.toml",
                section
            )),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UnknownCommand(cmd) => {
                write!(f, "unknown command '{cmd}'")
            }
            CliError::MissingArgument(arg) => {
                write!(f, "missing required option '--{arg}'")
            }
            CliError::UnknownFlag(flag) => {
                write!(f, "unknown flag '--{flag}'")
            }
            CliError::MissingValue(flag) => {
                write!(f, "flag '--{flag}' requires a value")
            }
            CliError::InvalidUsage(msg) => {
                write!(f, "{msg}")
            }
            CliError::ConfigRequired => {
                write!(f, "volki.toml not found")
            }
            CliError::ConfigSectionRequired(section) => {
                write!(f, "[{section}] section not found in volki.toml")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_unknown_command() {
        let err = CliError::UnknownCommand(String::from("foo"));
        let msg = crate::vformat!("{err}");
        assert!(msg.contains("unknown command"));
        assert!(msg.contains("foo"));
    }

    #[test]
    fn display_missing_argument() {
        let err = CliError::MissingArgument(String::from("name"));
        let msg = crate::vformat!("{err}");
        assert!(msg.contains("missing required"));
        assert!(msg.contains("name"));
    }

    #[test]
    fn display_unknown_flag() {
        let err = CliError::UnknownFlag(String::from("verbose"));
        let msg = crate::vformat!("{err}");
        assert!(msg.contains("unknown flag"));
        assert!(msg.contains("verbose"));
    }

    #[test]
    fn display_missing_value() {
        let err = CliError::MissingValue(String::from("path"));
        let msg = crate::vformat!("{err}");
        assert!(msg.contains("requires a value"));
        assert!(msg.contains("path"));
    }

    #[test]
    fn display_invalid_usage() {
        let err = CliError::InvalidUsage(String::from("bad input"));
        let msg = crate::vformat!("{err}");
        assert!(msg.contains("bad input"));
    }

    #[test]
    fn hint_unknown_command() {
        let err = CliError::UnknownCommand(String::from("foo"));
        assert!(err.hint().is_some());
    }

    #[test]
    fn hint_invalid_usage_is_none() {
        let err = CliError::InvalidUsage(String::from("whatever"));
        assert!(err.hint().is_none());
    }
}
