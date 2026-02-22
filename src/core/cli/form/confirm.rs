use crate::core::cli::error::CliError;
use crate::core::volkiwithstds::collections::String;
use crate::veprint;

use super::ansi;
use super::key::{self, Key};
use super::raw_mode::RawModeGuard;
use super::render;

pub struct Confirm {
    label: String,
    default: Option<bool>,
}

impl Confirm {
    pub fn new(label: &str) -> Self {
        Confirm {
            label: String::from(label),
            default: None,
        }
    }

    pub fn default_yes(mut self) -> Self {
        self.default = Some(true);
        self
    }

    pub fn default_no(mut self) -> Self {
        self.default = Some(false);
        self
    }

    pub fn run(&self) -> Result<bool, CliError> {
        let _guard = RawModeGuard::enter()?;

        ansi::hide_cursor();

        let hint = match self.default {
            Some(true) => "(Y/n)",
            Some(false) => "(y/N)",
            None => "(y/n)",
        };

        veprint!("  ");
        render::render_prompt(&self.label);
        veprint!(" {hint}");
        ansi::flush();

        loop {
            let k = key::read_key();
            match k {
                Key::Char('y') | Key::Char('Y') => {
                    ansi::erase_line();
                    veprint!("\r");
                    ansi::show_cursor();
                    render::render_answered(&self.label, "yes");
                    return Ok(true);
                }
                Key::Char('n') | Key::Char('N') => {
                    ansi::erase_line();
                    veprint!("\r");
                    ansi::show_cursor();
                    render::render_answered(&self.label, "no");
                    return Ok(false);
                }
                Key::Enter => {
                    if let Some(default) = self.default {
                        let answer = if default { "yes" } else { "no" };
                        ansi::erase_line();
                        veprint!("\r");
                        ansi::show_cursor();
                        render::render_answered(&self.label, answer);
                        return Ok(default);
                    }
                    // No default set, ignore Enter
                }
                Key::CtrlC => {
                    ansi::erase_line();
                    veprint!("\r");
                    ansi::show_cursor();
                    return Err(CliError::InvalidUsage(String::from("cancelled")));
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_builds() {
        let c = Confirm::new("Drop database?");
        assert_eq!(c.label.as_str(), "Drop database?");
        assert!(c.default.is_none());
    }

    #[test]
    fn confirm_default_yes() {
        let c = Confirm::new("Proceed?").default_yes();
        assert_eq!(c.default, Some(true));
    }

    #[test]
    fn confirm_default_no() {
        let c = Confirm::new("Proceed?").default_no();
        assert_eq!(c.default, Some(false));
    }
}
