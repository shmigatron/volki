pub mod command;
pub mod commands;
pub mod confirm;
pub mod error;
pub mod form;
pub mod help;
pub mod output;
pub mod parser;
pub mod progress;
pub mod registry;
pub mod spinner;
pub mod style;
pub mod terminal;
pub mod validate;

use commands::deadcode::DeadCodeCommand;
use commands::duplicate::DuplicateCommand;
use commands::fix::FixCommand;
use commands::format::FormatCommand;
use commands::init::InitCommand;
use commands::license::LicenseCommand;
use commands::outdated::OutdatedCommand;
use commands::run::RunCommand;
use commands::status::StatusCommand;
use crate::libs::db::cli::{DbCommand, DbHubCommand, UserCommand, TableCommand, WebEditorCommand};
use crate::libs::web::cli::{WebHubCommand, WebBuildCommand, WebStartCommand, WebDevCommand};
use crate::core::volkiwithstds::collections::String;
use registry::CommandRegistry;
use crate::vbox;
use crate::veprintln;
use error::CliError;

pub fn build_cli() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    registry.register(vbox!(DbHubCommand => dyn command::Command));
    registry.register(vbox!(DbCommand => dyn command::Command));
    registry.register(vbox!(DeadCodeCommand => dyn command::Command));
    registry.register(vbox!(DuplicateCommand => dyn command::Command));
    registry.register(vbox!(FixCommand => dyn command::Command));
    registry.register(vbox!(FormatCommand => dyn command::Command));
    registry.register(vbox!(InitCommand => dyn command::Command));
    registry.register(vbox!(LicenseCommand => dyn command::Command));
    registry.register(vbox!(OutdatedCommand => dyn command::Command));
    registry.register(vbox!(RunCommand => dyn command::Command));
    registry.register(vbox!(StatusCommand => dyn command::Command));
    registry.register(vbox!(TableCommand => dyn command::Command));
    registry.register(vbox!(UserCommand => dyn command::Command));
    registry.register(vbox!(WebEditorCommand => dyn command::Command));
    registry.register(vbox!(WebHubCommand => dyn command::Command));
    registry.register(vbox!(WebBuildCommand => dyn command::Command));
    registry.register(vbox!(WebStartCommand => dyn command::Command));
    registry.register(vbox!(WebDevCommand => dyn command::Command));
    registry
}

pub fn format_trace(file: &str, line: usize, col: usize) -> String {
    if line == 0 || col == 0 {
        crate::vformat!("{file}:?:?")
    } else {
        crate::vformat!("{file}:{line}:{col}")
    }
}

pub fn print_warn(message: &str) {
    veprintln!("  {} {}", style::yellow("warn"), style::yellow(message));
}

pub fn print_warn_trace(file: &str, line: usize, col: usize, message: &str) {
    let trace = format_trace(file, line, col);
    veprintln!(
        "  {} {}",
        style::yellow("warn"),
        style::yellow(message),
    );
    veprintln!("    {} {}", style::dim(style::ARROW), style::dim(trace.as_str()));
}

pub fn print_error(message: &str) {
    veprintln!("  {} {}", style::red("error"), style::red(message));
}

pub fn print_error_trace(file: &str, line: usize, col: usize, message: &str) {
    let trace = format_trace(file, line, col);
    veprintln!("  {} {}", style::red("error"), style::red(message));
    veprintln!("    {} {}", style::dim(style::ARROW), style::dim(trace.as_str()));
}

pub fn print_hint_line(message: &str) {
    veprintln!("    {} {}", style::dim(style::ARROW), message);
}

/// Render a CLI error with Volki-styled sections, traces, and hints.
pub fn print_cli_error(err: &CliError) {
    let rendered = crate::vformat!("{err}");
    let mut printed_primary = false;
    let mut error_count: usize = 0;
    let mut warning_count: usize = 0;

    for raw_line in rendered.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((file, ln, col, msg)) = parse_trace_prefix(line) {
            if msg.starts_with("warning:") || msg.starts_with("warn:") {
                let clean = msg
                    .strip_prefix("warning:")
                    .or_else(|| msg.strip_prefix("warn:"))
                    .map(|s| s.trim())
                    .unwrap_or(msg);
                print_warn_trace(file, ln, col, clean);
                warning_count += 1;
            } else {
                print_error_trace(file, ln, col, msg);
                error_count += 1;
            }
            printed_primary = true;
            continue;
        }

        if let Some(trace) = parse_arrow_trace(line) {
            veprintln!("    {} {}", style::dim(style::ARROW), style::dim(trace.as_str()));
            continue;
        }

        if line == "|" || line.starts_with("|") {
            continue;
        }

        if let Some(warn) = line
            .strip_prefix("warning:")
            .or_else(|| line.strip_prefix("warn:"))
        {
            print_warn(warn.trim());
            warning_count += 1;
            continue;
        }

        if let Some(help) = line.strip_prefix("= help:") {
            print_hint_line(help.trim());
            continue;
        }

        if let Some(help) = line.strip_prefix("help:") {
            print_hint_line(help.trim());
            continue;
        }

        if let Some(help) = line.strip_prefix("hint:") {
            print_hint_line(help.trim());
            continue;
        }

        let msg = line.strip_prefix("error:").map(|s| s.trim()).unwrap_or(line);
        if !printed_primary {
            print_error(msg);
            error_count += 1;
            printed_primary = true;
        } else {
            veprintln!("    {} {}", style::dim(style::ARROW), msg);
        }
    }

    if !printed_primary {
        print_error("unknown error");
        error_count += 1;
    }

    if let Some(hint) = err.hint() {
        print_hint_line(hint.as_str());
    }

    let error_label = if error_count == 1 { "error" } else { "errors" };
    let warning_label = if warning_count == 1 { "warning" } else { "warnings" };
    veprintln!(
        "  {} {} , {}",
        style::dim("totals:"),
        style::red(&crate::vformat!("{error_count} {error_label}")),
        style::yellow(&crate::vformat!("{warning_count} {warning_label}")),
    );
}

fn parse_arrow_trace(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("-->")?.trim();
    if rest.is_empty() {
        return None;
    }
    Some(String::from(rest))
}

fn parse_trace_prefix(line: &str) -> Option<(&str, usize, usize, &str)> {
    let i3 = line.rfind(':')?;
    let msg = line.get(i3 + 1..)?.trim();
    if msg.is_empty() {
        return None;
    }

    let head = line.get(..i3)?;
    let i2 = head.rfind(':')?;
    let col_s = head.get(i2 + 1..)?;
    let col = col_s.parse::<usize>().ok()?;

    let head2 = head.get(..i2)?;
    let i1 = head2.rfind(':')?;
    let line_s = head2.get(i1 + 1..)?;
    let ln = line_s.parse::<usize>().ok()?;

    let file = head2.get(..i1)?;
    if file.is_empty() {
        return None;
    }

    Some((file, ln, col, msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_trace_prefix_works() {
        let (file, line, col, msg) = parse_trace_prefix("src/app/page.volki:12:8: bad token").unwrap();
        assert_eq!(file, "src/app/page.volki");
        assert_eq!(line, 12);
        assert_eq!(col, 8);
        assert_eq!(msg, "bad token");
    }

    #[test]
    fn parse_trace_prefix_rejects_non_trace() {
        assert!(parse_trace_prefix("unknown command 'foo'").is_none());
    }
}
