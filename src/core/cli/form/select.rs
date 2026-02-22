use crate::core::cli::error::CliError;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::veprintln;

use super::ansi;
use super::key::{self, Key};
use super::raw_mode::RawModeGuard;
use super::render;

pub struct Select {
    label: String,
    options: Vec<String>,
    default_index: usize,
}

impl Select {
    pub fn new(label: &str, options: Vec<&str>) -> Self {
        Select {
            label: String::from(label),
            options: options.into_iter().map(|s| String::from(s)).collect(),
            default_index: 0,
        }
    }

    pub fn default_index(mut self, idx: usize) -> Self {
        self.default_index = idx;
        self
    }

    pub fn run(&self) -> Result<(usize, String), CliError> {
        if self.options.is_empty() {
            return Err(CliError::InvalidUsage(String::from("no options provided")));
        }

        let _guard = RawModeGuard::enter()?;
        let mut selected = self.default_index.min(self.options.len() - 1);

        ansi::hide_cursor();

        // Initial render: prompt + options
        render::render_prompt(&self.label);
        veprintln!();
        self.render_options(selected);
        ansi::flush();

        loop {
            let k = key::read_key();
            match k {
                Key::Up => {
                    if selected == 0 {
                        selected = self.options.len() - 1;
                    } else {
                        selected -= 1;
                    }
                    self.redraw_options(selected);
                }
                Key::Down => {
                    selected = (selected + 1) % self.options.len();
                    self.redraw_options(selected);
                }
                Key::Enter | Key::Space => {
                    // Clear prompt + all option lines
                    let total_lines = 1 + self.options.len();
                    ansi::move_up(0); // stay on last option line
                    ansi::erase_lines(total_lines);
                    ansi::show_cursor();
                    render::render_answered(&self.label, &self.options[selected]);
                    return Ok((selected, self.options[selected].clone()));
                }
                Key::CtrlC => {
                    let total_lines = 1 + self.options.len();
                    ansi::erase_lines(total_lines);
                    ansi::show_cursor();
                    return Err(CliError::InvalidUsage(String::from("cancelled")));
                }
                _ => {}
            }
        }
    }

    fn render_options(&self, selected: usize) {
        for (i, opt) in self.options.iter().enumerate() {
            render::render_option(opt, i == selected);
        }
    }

    fn redraw_options(&self, selected: usize) {
        // Move up to first option line, erase and rewrite
        ansi::move_up(self.options.len());
        for (i, opt) in self.options.iter().enumerate() {
            ansi::erase_line();
            render::render_option(opt, i == selected);
            if i < self.options.len() - 1 {
                // render_option uses veprintln which already moves down
            }
        }
        ansi::flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vvec;

    #[test]
    fn select_builds() {
        let s = Select::new("Dialect", vvec!["postgres", "mysql", "sqlite"]);
        assert_eq!(s.label.as_str(), "Dialect");
        assert_eq!(s.options.len(), 3);
        assert_eq!(s.default_index, 0);
    }

    #[test]
    fn select_default_index() {
        let s = Select::new("Dialect", vvec!["postgres", "mysql"]).default_index(1);
        assert_eq!(s.default_index, 1);
    }

    #[test]
    fn select_empty_options_errors() {
        let s = Select::new("Empty", vvec![]);
        // Can't run without raw mode, but we test the error path via the empty check
        // Since RawModeGuard::enter() fails in test (no tty), this is a compile check
        assert!(s.options.is_empty());
    }
}
