use std::sync::atomic::{AtomicBool, Ordering};

use super::terminal;

static COLOR_DISABLED: AtomicBool = AtomicBool::new(false);

pub fn disable_color() {
    COLOR_DISABLED.store(true, Ordering::Relaxed);
}

pub fn use_color() -> bool {
    if COLOR_DISABLED.load(Ordering::Relaxed) {
        return false;
    }
    if terminal::no_color() {
        return false;
    }
    terminal::is_tty()
}

pub const PURPLE: &str = "\x1b[35m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";
pub const CYAN: &str = "\x1b[36m";
pub const DIM: &str = "\x1b[90m";
pub const BOLD: &str = "\x1b[1m";
pub const BOLD_CYAN: &str = "\x1b[1;36m";
pub const RESET: &str = "\x1b[0m";

pub const CHECK: &str = "\u{2713}";   // ✓
pub const CROSS: &str = "\u{2717}";   // ✗
pub const WARN: &str = "\u{26A0}";    // ⚠
pub const ARROW: &str = "\u{2192}";   // →
pub const BULLET: &str = "\u{25CF}";  // ●
pub const PENDING: &str = "\u{25CC}"; // ◌
pub const WOLF: &str = "\u{1F43A}";   // 🐺
pub const PAW: &str = "\u{1F43E}";    // 🐾
pub const SEARCH: &str = "\u{1F50D}"; // 🔍

pub const TREE_BRANCH: &str = "\u{251C}\u{2500}\u{2500}"; // ├──
pub const TREE_LAST: &str = "\u{2514}\u{2500}\u{2500}";   // └──
pub const TREE_PIPE: &str = "\u{2502}";                     // │

pub fn purple(s: &str) -> String {
    if use_color() { format!("{PURPLE}{s}{RESET}") } else { s.to_string() }
}

pub fn green(s: &str) -> String {
    if use_color() { format!("{GREEN}{s}{RESET}") } else { s.to_string() }
}

pub fn yellow(s: &str) -> String {
    if use_color() { format!("{YELLOW}{s}{RESET}") } else { s.to_string() }
}

pub fn red(s: &str) -> String {
    if use_color() { format!("{RED}{s}{RESET}") } else { s.to_string() }
}

pub fn cyan(s: &str) -> String {
    if use_color() { format!("{CYAN}{s}{RESET}") } else { s.to_string() }
}

pub fn dim(s: &str) -> String {
    if use_color() { format!("{DIM}{s}{RESET}") } else { s.to_string() }
}

pub fn bold(s: &str) -> String {
    if use_color() { format!("{BOLD}{s}{RESET}") } else { s.to_string() }
}

pub fn bold_cyan(s: &str) -> String {
    if use_color() { format!("{BOLD_CYAN}{s}{RESET}") } else { s.to_string() }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn banner() -> String {
    format!("{WOLF} volki v{VERSION}")
}

pub fn hint(msg: &str) -> String {
    format!("  {msg} {PAW}")
}

pub fn format_duration(ms: u128) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        let secs = ms as f64 / 1000.0;
        format!("{secs:.1}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_contains_version() {
        let b = banner();
        assert!(b.contains("volki"));
        assert!(b.contains("v"));
    }

    #[test]
    fn hint_has_paw() {
        let h = hint("try --help");
        assert!(h.contains("try --help"));
        assert!(h.contains(PAW));
    }

    #[test]
    fn format_duration_ms() {
        assert_eq!(format_duration(120), "120ms");
    }

    #[test]
    fn format_duration_secs() {
        assert_eq!(format_duration(1500), "1.5s");
    }

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0), "0ms");
    }

    #[test]
    fn disable_color_works() {
        // Save state
        let prev = COLOR_DISABLED.load(Ordering::Relaxed);
        disable_color();
        let txt = purple("hello");
        assert_eq!(txt, "hello");
        // Restore
        COLOR_DISABLED.store(prev, Ordering::Relaxed);
    }

    #[test]
    fn symbols_are_nonempty() {
        assert!(!CHECK.is_empty());
        assert!(!CROSS.is_empty());
        assert!(!WARN.is_empty());
        assert!(!ARROW.is_empty());
        assert!(!WOLF.is_empty());
        assert!(!PAW.is_empty());
    }
}
