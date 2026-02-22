use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::io::traits::Write;
use crate::core::volkiwithstds::time::Instant;
use crate::{veprint, veprintln};

use super::style;

pub struct ProgressBar {
    total: u64,
    current: u64,
    label: String,
    bar_width: usize,
    last_drawn_pct: u64,
    last_drawn_at: Instant,
}

impl ProgressBar {
    pub fn new(total: u64, label: &str) -> Self {
        ProgressBar {
            total,
            current: 0,
            label: String::from(label),
            bar_width: 20,
            last_drawn_pct: u64::MAX,
            last_drawn_at: Instant::now(),
        }
    }

    pub fn set(&mut self, current: u64) {
        self.current = current.min(self.total);
        self.draw();
    }

    pub fn inc(&mut self, amount: u64) {
        self.current = (self.current + amount).min(self.total);
        self.draw();
    }

    pub fn finish(&self) {
        let pct = if self.total == 0 {
            100
        } else {
            100 * self.current / self.total
        };
        let filled = if self.total == 0 {
            self.bar_width
        } else {
            (self.bar_width as u64 * self.current / self.total) as usize
        };
        let empty = self.bar_width - filled;

        let bar_filled = String::from("\u{2588}").repeat(filled);
        let bar_empty = String::from("\u{2591}").repeat(empty);

        let bar_str = if style::use_color() {
            crate::vformat!("{}{}{}{bar_empty}", style::PURPLE, bar_filled, style::RESET)
        } else {
            crate::vformat!("{bar_filled}{bar_empty}")
        };

        veprintln!(
            "\r  {}  {}  {}/{}  {pct}% {}",
            self.label,
            bar_str,
            self.current,
            self.total,
            style::green(style::CHECK),
        );
    }

    pub fn finish_with_error(&self) {
        let pct = if self.total == 0 {
            100
        } else {
            100 * self.current / self.total
        };
        let filled = if self.total == 0 {
            self.bar_width
        } else {
            (self.bar_width as u64 * self.current / self.total) as usize
        };
        let empty = self.bar_width - filled;

        let bar_filled = String::from("\u{2588}").repeat(filled);
        let bar_empty = String::from("\u{2591}").repeat(empty);

        let bar_str = if style::use_color() {
            crate::vformat!("{}{}{}{bar_empty}", style::RED, bar_filled, style::RESET)
        } else {
            crate::vformat!("{bar_filled}{bar_empty}")
        };

        veprintln!(
            "\r  {}  {}  {}/{}  {pct}% {}",
            self.label,
            bar_str,
            self.current,
            self.total,
            style::red(style::CROSS),
        );
    }

    fn draw(&mut self) {
        let pct = if self.total == 0 {
            100
        } else {
            100 * self.current / self.total
        };

        // Rate limiting: skip if <1% change and <100ms elapsed
        let elapsed = self.last_drawn_at.elapsed().as_millis();
        if pct == self.last_drawn_pct && elapsed < 100 {
            return;
        }
        if pct != self.last_drawn_pct || elapsed >= 100 {
            self.last_drawn_pct = pct;
            self.last_drawn_at = Instant::now();
        }

        let filled = if self.total == 0 {
            self.bar_width
        } else {
            (self.bar_width as u64 * self.current / self.total) as usize
        };
        let empty = self.bar_width - filled;

        let bar_filled = String::from("\u{2588}").repeat(filled);
        let bar_empty = String::from("\u{2591}").repeat(empty);

        let bar_str = if style::use_color() {
            crate::vformat!("{}{}{}{bar_empty}", style::PURPLE, bar_filled, style::RESET)
        } else {
            crate::vformat!("{bar_filled}{bar_empty}")
        };

        veprint!(
            "\r  {}  {}  {}/{}  {pct}%",
            self.label,
            bar_str,
            self.current,
            self.total,
        );
        let _ = crate::core::volkiwithstds::io::stderr().flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero() {
        let pb = ProgressBar::new(100, "test");
        assert_eq!(pb.current, 0);
        assert_eq!(pb.total, 100);
    }

    #[test]
    fn bar_width_is_20() {
        let pb = ProgressBar::new(100, "test");
        assert_eq!(pb.bar_width, 20);
    }

    #[test]
    fn set_clamps_to_total() {
        let mut pb = ProgressBar::new(10, "test");
        pb.set(20);
        assert_eq!(pb.current, 10);
    }

    #[test]
    fn set_within_range() {
        let mut pb = ProgressBar::new(100, "test");
        pb.set(50);
        assert_eq!(pb.current, 50);
    }

    #[test]
    fn inc_accumulates() {
        let mut pb = ProgressBar::new(100, "test");
        pb.inc(10);
        pb.inc(20);
        assert_eq!(pb.current, 30);
    }

    #[test]
    fn inc_clamps() {
        let mut pb = ProgressBar::new(10, "test");
        pb.inc(5);
        pb.inc(10);
        assert_eq!(pb.current, 10);
    }

    #[test]
    fn zero_total_no_panic() {
        let pb = ProgressBar::new(0, "test");
        pb.finish(); // should not panic
    }

    #[test]
    fn finish_with_error_no_panic() {
        let pb = ProgressBar::new(10, "test");
        pb.finish_with_error();
    }
}
