use crate::core::volkiwithstds::io::traits::Write;
use crate::veprint;

pub fn hide_cursor() {
    veprint!("\x1b[?25l");
    flush();
}

pub fn show_cursor() {
    veprint!("\x1b[?25h");
    flush();
}

pub fn move_up(n: usize) {
    if n > 0 {
        veprint!("\x1b[{n}A");
    }
}

pub fn move_down(n: usize) {
    if n > 0 {
        veprint!("\x1b[{n}B");
    }
}

pub fn move_to_col(n: usize) {
    veprint!("\x1b[{n}G");
}

pub fn erase_line() {
    veprint!("\x1b[2K");
}

pub fn erase_lines(n: usize) {
    for i in 0..n {
        erase_line();
        if i < n - 1 {
            move_up(1);
        }
    }
    veprint!("\r");
    flush();
}

pub fn flush() {
    let _ = crate::core::volkiwithstds::io::stderr().flush();
}

// Testable versions that write to a buffer instead of stderr.
#[cfg(test)]
pub mod testable {
    use crate::core::volkiwithstds::collections::String;

    pub fn hide_cursor_str() -> &'static str {
        "\x1b[?25l"
    }

    pub fn show_cursor_str() -> &'static str {
        "\x1b[?25h"
    }

    pub fn move_up_str(n: usize) -> String {
        if n > 0 {
            crate::vformat!("\x1b[{n}A")
        } else {
            String::new()
        }
    }

    pub fn move_down_str(n: usize) -> String {
        if n > 0 {
            crate::vformat!("\x1b[{n}B")
        } else {
            String::new()
        }
    }

    pub fn move_to_col_str(n: usize) -> String {
        crate::vformat!("\x1b[{n}G")
    }

    pub fn erase_line_str() -> &'static str {
        "\x1b[2K"
    }
}

#[cfg(test)]
mod tests {
    use super::testable::*;

    #[test]
    fn hide_cursor_escape_code() {
        assert_eq!(hide_cursor_str(), "\x1b[?25l");
    }

    #[test]
    fn show_cursor_escape_code() {
        assert_eq!(show_cursor_str(), "\x1b[?25h");
    }

    #[test]
    fn move_up_escape_code() {
        assert_eq!(move_up_str(3).as_str(), "\x1b[3A");
    }

    #[test]
    fn move_up_zero_is_empty() {
        assert_eq!(move_up_str(0).as_str(), "");
    }

    #[test]
    fn move_down_escape_code() {
        assert_eq!(move_down_str(2).as_str(), "\x1b[2B");
    }

    #[test]
    fn move_to_col_escape_code() {
        assert_eq!(move_to_col_str(5).as_str(), "\x1b[5G");
    }

    #[test]
    fn erase_line_escape_code() {
        assert_eq!(erase_line_str(), "\x1b[2K");
    }
}
