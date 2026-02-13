use super::error::CliError;
use super::parser::ParsedArgs;

#[allow(dead_code)]
pub struct OptionSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub takes_value: bool,
    pub required: bool,
    pub default_value: Option<&'static str>,
    pub short: Option<char>,
}

pub trait Command {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn long_description(&self) -> &str {
        self.description()
    }
    fn options(&self) -> Vec<OptionSpec> {
        Vec::new()
    }
    fn requires_config(&self) -> bool {
        true
    }
    fn execute(&self, args: &ParsedArgs) -> Result<(), CliError>;
}
