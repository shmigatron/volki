use crate::core::cli::style;
use crate::core::volkiwithstds::collections::String;
use crate::{veprint, veprintln};

pub fn render_prompt(label: &str) {
    veprint!("  {} {}", style::purple("?"), style::bold(label));
}

pub fn render_input(value: &str) {
    veprint!("\r  {} {}", style::cyan(style::ARROW), value);
}

pub fn render_answered(label: &str, answer: &str) {
    veprintln!(
        "  {} {}  {}",
        style::green(style::CHECK),
        style::bold(label),
        style::cyan(answer)
    );
}

pub fn render_option(label: &str, selected: bool) {
    if selected {
        veprintln!("    {} {}", style::cyan(style::BULLET), label);
    } else {
        veprintln!("    {} {}", style::dim(style::PENDING), style::dim(label));
    }
}

pub fn render_error(msg: &str) {
    veprintln!("    {} {}", style::red(style::CROSS), style::red(msg));
}

// Testable versions returning strings instead of printing.
pub fn format_prompt(label: &str) -> String {
    crate::vformat!("  {} {}", style::purple("?"), style::bold(label))
}

pub fn format_answered(label: &str, answer: &str) -> String {
    crate::vformat!(
        "  {} {}  {}",
        style::green(style::CHECK),
        style::bold(label),
        style::cyan(answer)
    )
}

pub fn format_option(label: &str, selected: bool) -> String {
    if selected {
        crate::vformat!("    {} {}", style::cyan(style::BULLET), label)
    } else {
        crate::vformat!("    {} {}", style::dim(style::PENDING), style::dim(label))
    }
}

pub fn format_error(msg: &str) -> String {
    crate::vformat!("    {} {}", style::red(style::CROSS), style::red(msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_contains_label() {
        let s = format_prompt("Database name");
        assert!(s.contains("Database name"));
        assert!(s.contains("?"));
    }

    #[test]
    fn answered_contains_both() {
        let s = format_answered("Database name", "my_db");
        assert!(s.contains("Database name"));
        assert!(s.contains("my_db"));
        assert!(s.contains(style::CHECK));
    }

    #[test]
    fn option_selected_has_bullet() {
        let s = format_option("postgres", true);
        assert!(s.contains("postgres"));
        assert!(s.contains(style::BULLET));
    }

    #[test]
    fn option_unselected_has_pending() {
        let s = format_option("mysql", false);
        assert!(s.contains("mysql"));
        assert!(s.contains(style::PENDING));
    }

    #[test]
    fn error_contains_message() {
        let s = format_error("must not be empty");
        assert!(s.contains("must not be empty"));
        assert!(s.contains(style::CROSS));
    }
}
