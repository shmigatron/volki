use std::fmt;

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
}

impl CliError {
    pub fn hint(&self) -> Option<String> {
        match self {
            CliError::UnknownCommand(_) => {
                Some(format!("run {} to see available commands", style::bold("volki --help")))
            }
            CliError::MissingArgument(arg) => {
                Some(format!("provide a value with {}", style::bold(&format!("--{arg} <value>"))))
            }
            CliError::UnknownFlag(_) => {
                Some(format!("run with {} to see valid options", style::bold("--help")))
            }
            CliError::MissingValue(flag) => {
                Some(format!("provide a value: {}", style::bold(&format!("--{flag} <value>"))))
            }
            CliError::InvalidUsage(_) => None,
            CliError::ConfigRequired => {
                Some(format!("run {} to initialize your project", style::bold("volki init")))
            }
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
        }
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_unknown_command() {
        let err = CliError::UnknownCommand("foo".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("unknown command"));
        assert!(msg.contains("foo"));
    }

    #[test]
    fn display_missing_argument() {
        let err = CliError::MissingArgument("name".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("missing required"));
        assert!(msg.contains("name"));
    }

    #[test]
    fn display_unknown_flag() {
        let err = CliError::UnknownFlag("verbose".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("unknown flag"));
        assert!(msg.contains("verbose"));
    }

    #[test]
    fn display_missing_value() {
        let err = CliError::MissingValue("path".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("requires a value"));
        assert!(msg.contains("path"));
    }

    #[test]
    fn display_invalid_usage() {
        let err = CliError::InvalidUsage("bad input".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("bad input"));
    }

    #[test]
    fn implements_std_error() {
        let err = CliError::UnknownCommand("x".to_string());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn hint_unknown_command() {
        let err = CliError::UnknownCommand("foo".to_string());
        assert!(err.hint().is_some());
    }

    #[test]
    fn hint_invalid_usage_is_none() {
        let err = CliError::InvalidUsage("whatever".to_string());
        assert!(err.hint().is_none());
    }
}
