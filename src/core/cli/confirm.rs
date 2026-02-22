use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::io::traits::{BufRead, Write};
use crate::{veprint, veprintln};

use super::error::CliError;
use super::style;
use super::terminal;

#[derive(Debug, PartialEq, Eq)]
pub enum ConfirmResult {
    Confirmed,
    Cancelled,
}

pub fn confirm_destructive(
    action_description: &str,
    resource_name: &str,
    force: bool,
) -> Result<ConfirmResult, CliError> {
    if force {
        return Ok(ConfirmResult::Confirmed);
    }

    if !terminal::is_stdin_tty() {
        return Err(CliError::InvalidUsage(crate::vformat!(
            "destructive action requires confirmation but stdin is not a terminal\n\n  \
             use --force to skip confirmation in non-interactive environments"
        )));
    }

    let expected = crate::vformat!("sudo delete {resource_name}");

    veprintln!();
    veprintln!(
        "  {} {}",
        style::red(&crate::vformat!("{} destructive action:", style::WARN)),
        action_description
    );
    veprintln!();
    veprint!(
        "  type '{}' to continue, or press Ctrl+C to cancel:\n\n  {} ",
        style::bold(&expected),
        style::ARROW
    );
    crate::core::volkiwithstds::io::stderr().flush().ok();

    let mut input = String::new();
    crate::core::volkiwithstds::io::stdin()
        .lock()
        .read_line(&mut input)
        .map_err(|e| CliError::InvalidUsage(crate::vformat!("failed to read input: {e}")))?;

    let input = input.trim();

    if input == expected.as_str() {
        veprintln!();
        Ok(ConfirmResult::Confirmed)
    } else {
        veprintln!();
        veprintln!("  confirmation failed, action cancelled");
        veprintln!();
        Ok(ConfirmResult::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_skips_prompt() {
        let result = confirm_destructive("DROP DATABASE mydb", "mydb", true).unwrap();
        assert_eq!(result, ConfirmResult::Confirmed);
    }

    #[test]
    fn non_tty_without_force_errors() {
        super::terminal::set_stdin_tty_override(Some(false));
        let result = confirm_destructive("DROP DATABASE mydb", "mydb", false);
        super::terminal::set_stdin_tty_override(None);
        assert!(result.is_err());
        let msg = crate::vformat!("{}", result.unwrap_err());
        assert!(msg.contains("--force"));
    }
}
