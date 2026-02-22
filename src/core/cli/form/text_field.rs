use crate::core::cli::error::CliError;
use crate::core::volkiwithstds::collections::{Box, String, Vec};
use crate::{vbox, veprintln};

use super::ansi;
use super::key::{self, Key};
use super::raw_mode::RawModeGuard;
use super::render;

pub struct TextField {
    label: String,
    default: Option<String>,
    validator: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
}

impl TextField {
    pub fn new(label: &str) -> Self {
        TextField {
            label: String::from(label),
            default: None,
            validator: None,
        }
    }

    pub fn default_value(mut self, val: &str) -> Self {
        self.default = Some(String::from(val));
        self
    }

    pub fn validate<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + 'static,
    {
        self.validator = Some(vbox!(f => dyn Fn(&str) -> Result<(), String>));
        self
    }

    pub fn run(&self) -> Result<String, CliError> {
        let _guard = RawModeGuard::enter()?;

        let mut value: Vec<char> = self.default.as_deref().unwrap_or("").chars().collect();
        let mut cursor: usize = value.len();
        let mut error_showing = false;

        // Initial render: prompt line + input line
        render::render_prompt(&self.label);
        veprintln!();
        render::render_input(&value.iter().copied().collect::<String>());
        self.position_cursor(cursor);
        ansi::flush();

        loop {
            let k = key::read_key();
            match k {
                Key::Char(c) => {
                    if error_showing {
                        self.clear_error();
                        error_showing = false;
                    }
                    value.insert(cursor, c);
                    cursor += 1;
                    self.redraw_input(&value, cursor);
                }
                Key::Backspace => {
                    if cursor > 0 {
                        if error_showing {
                            self.clear_error();
                            error_showing = false;
                        }
                        cursor -= 1;
                        value.remove(cursor);
                        self.redraw_input(&value, cursor);
                    }
                }
                Key::Left => {
                    if cursor > 0 {
                        cursor -= 1;
                        self.position_cursor(cursor);
                        ansi::flush();
                    }
                }
                Key::Right => {
                    if cursor < value.len() {
                        cursor += 1;
                        self.position_cursor(cursor);
                        ansi::flush();
                    }
                }
                Key::Space => {
                    if error_showing {
                        self.clear_error();
                        error_showing = false;
                    }
                    value.insert(cursor, ' ');
                    cursor += 1;
                    self.redraw_input(&value, cursor);
                }
                Key::Enter => {
                    let val_str: String = value.iter().copied().collect();
                    if let Some(ref validator) = self.validator {
                        if let Err(msg) = validator(&val_str) {
                            if error_showing {
                                self.clear_error();
                            }
                            veprintln!();
                            render::render_error(&msg);
                            // Move back up to input line
                            ansi::move_up(1);
                            self.position_cursor(cursor);
                            ansi::flush();
                            error_showing = true;
                            continue;
                        }
                    }
                    // Clear interactive lines and show answered state
                    let lines_to_clear = if error_showing { 3 } else { 2 };
                    self.clear_all(lines_to_clear);
                    render::render_answered(&self.label, &val_str);
                    return Ok(val_str);
                }
                Key::CtrlC => {
                    let lines_to_clear = if error_showing { 3 } else { 2 };
                    self.clear_all(lines_to_clear);
                    return Err(CliError::InvalidUsage(String::from("cancelled")));
                }
                _ => {}
            }
        }
    }

    fn redraw_input(&self, value: &[char], cursor: usize) {
        ansi::erase_line();
        let val_str: String = value.iter().copied().collect();
        render::render_input(&val_str);
        self.position_cursor(cursor);
        ansi::flush();
    }

    fn position_cursor(&self, cursor: usize) {
        // "  → " prefix is 4 visible chars, then the cursor position within the value
        ansi::move_to_col(5 + cursor);
    }

    fn clear_error(&self) {
        // Move down to error line, erase it, move back up
        ansi::move_down(1);
        ansi::erase_line();
        ansi::move_up(1);
    }

    fn clear_all(&self, lines: usize) {
        // Move to bottom of our output, then erase upward
        if lines > 2 {
            ansi::move_down(1); // from input line to error line
        }
        ansi::erase_lines(lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_field_builds() {
        let tf = TextField::new("Database name")
            .default_value("mydb")
            .validate(|v| {
                if v.is_empty() {
                    Err(String::from("must not be empty"))
                } else {
                    Ok(())
                }
            });
        assert_eq!(tf.label.as_str(), "Database name");
        assert_eq!(tf.default.as_deref(), Some("mydb"));
        assert!(tf.validator.is_some());
    }

    #[test]
    fn text_field_default_none() {
        let tf = TextField::new("Name");
        assert!(tf.default.is_none());
        assert!(tf.validator.is_none());
    }

    #[test]
    fn validator_accepts_valid() {
        let tf = TextField::new("Name").validate(|v| {
            if v.chars().all(|c| c.is_alphanumeric()) {
                Ok(())
            } else {
                Err(String::from("invalid"))
            }
        });
        let v = tf.validator.as_ref().unwrap();
        assert!(v("hello").is_ok());
    }

    #[test]
    fn validator_rejects_invalid() {
        let tf = TextField::new("Name").validate(|v| {
            if v.is_empty() {
                Err(String::from("empty"))
            } else {
                Ok(())
            }
        });
        let v = tf.validator.as_ref().unwrap();
        assert_eq!(v("").unwrap_err().as_str(), "empty");
    }
}
