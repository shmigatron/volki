pub mod ansi;
pub mod confirm;
pub mod key;
pub mod raw_mode;
pub mod render;
pub mod select;
pub mod text_field;

use crate::vformat;
use crate::core::volkiwithstds::collections::{String, Vec, HashMap};

use super::error::CliError;
use super::terminal;

pub use confirm::Confirm;
pub use select::Select;
pub use text_field::TextField;

pub struct Form {
    fields: Vec<FormField>,
}

pub enum FormField {
    Text { name: String, field: TextField },
    Select { name: String, field: Select },
    Confirm { name: String, field: Confirm },
}

#[derive(Debug)]
pub struct FormResult {
    values: HashMap<String, String>,
}

impl FormResult {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(&String::from(key)).map(|s| s.as_str())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(&String::from(key)).map(|v| v == "true")
    }
}

impl Form {
    pub fn new() -> Self {
        Form { fields: Vec::new() }
    }

    pub fn text(mut self, name: &str, field: TextField) -> Self {
        self.fields.push(FormField::Text {
            name: String::from(name),
            field,
        });
        self
    }

    pub fn select(mut self, name: &str, field: Select) -> Self {
        self.fields.push(FormField::Select {
            name: String::from(name),
            field,
        });
        self
    }

    pub fn confirm(mut self, name: &str, field: Confirm) -> Self {
        self.fields.push(FormField::Confirm {
            name: String::from(name),
            field,
        });
        self
    }

    pub fn run(self) -> Result<FormResult, CliError> {
        if !terminal::is_stdin_tty() {
            return Err(CliError::InvalidUsage(
                String::from("interactive form requires a terminal (stdin is not a TTY)"),
            ));
        }

        let mut values = HashMap::new();

        for field in self.fields {
            match field {
                FormField::Text { name, field } => {
                    let val = field.run()?;
                    values.insert(name, val);
                }
                FormField::Select { name, field } => {
                    let (_, val) = field.run()?;
                    values.insert(name, val);
                }
                FormField::Confirm { name, field } => {
                    let val = field.run()?;
                    values.insert(name, vformat!("{val}"));
                }
            }
        }

        Ok(FormResult { values })
    }
}

#[cfg(test)]
mod tests {
    use crate::vvec;
    use super::*;

    #[test]
    fn form_result_get() {
        let mut values = HashMap::new();
        values.insert(String::from("name"), String::from("mydb"));
        values.insert(String::from("confirm"), String::from("true"));
        let result = FormResult { values };

        assert_eq!(result.get("name"), Some("mydb"));
        assert_eq!(result.get("missing"), None);
    }

    #[test]
    fn form_result_get_bool() {
        let mut values = HashMap::new();
        values.insert(String::from("proceed"), String::from("true"));
        values.insert(String::from("skip"), String::from("false"));
        let result = FormResult { values };

        assert_eq!(result.get_bool("proceed"), Some(true));
        assert_eq!(result.get_bool("skip"), Some(false));
        assert_eq!(result.get_bool("missing"), None);
    }

    #[test]
    fn form_builder_chain() {
        let form = Form::new()
            .text("name", TextField::new("Database name"))
            .select("dialect", Select::new("Dialect", vvec!["postgres", "mysql"]))
            .confirm("proceed", Confirm::new("Create?").default_yes());

        assert_eq!(form.fields.len(), 3);
    }

    #[test]
    fn form_non_tty_returns_error() {
        terminal::set_stdin_tty_override(Some(false));
        let form = Form::new().text("name", TextField::new("Name"));
        let result = form.run();
        terminal::set_stdin_tty_override(None);
        assert!(result.is_err());
        let msg = vformat!("{}", result.unwrap_err());
        assert!(msg.contains("terminal") || msg.contains("TTY"));
    }
}
