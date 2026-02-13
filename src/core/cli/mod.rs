pub mod command;
pub mod commands;
pub mod error;
pub mod help;
pub mod output;
pub mod parser;
pub mod progress;
pub mod registry;
pub mod spinner;
pub mod style;
pub mod terminal;

use commands::deadcode::DeadCodeCommand;
use commands::duplicate::DuplicateCommand;
use commands::fix::FixCommand;
use commands::format::FormatCommand;
use commands::init::InitCommand;
use commands::license::LicenseCommand;
use commands::outdated::OutdatedCommand;
use commands::run::RunCommand;
use commands::status::StatusCommand;
use crate::libs::db::cli::{DbCommand, DbHubCommand, UserCommand, TableCommand};
use registry::CommandRegistry;

pub fn build_cli() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    registry.register(Box::new(DbHubCommand));
    registry.register(Box::new(DbCommand));
    registry.register(Box::new(DeadCodeCommand));
    registry.register(Box::new(DuplicateCommand));
    registry.register(Box::new(FixCommand));
    registry.register(Box::new(FormatCommand));
    registry.register(Box::new(InitCommand));
    registry.register(Box::new(LicenseCommand));
    registry.register(Box::new(OutdatedCommand));
    registry.register(Box::new(RunCommand));
    registry.register(Box::new(StatusCommand));
    registry.register(Box::new(TableCommand));
    registry.register(Box::new(UserCommand));
    registry
}
