use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::style;

const BRAILLE_FRAMES: &[&str] = &[
    "\u{280B}", // ⠋
    "\u{2819}", // ⠙
    "\u{2839}", // ⠹
    "\u{2838}", // ⠸
    "\u{283C}", // ⠼
    "\u{2834}", // ⠴
    "\u{2826}", // ⠦
    "\u{2827}", // ⠧
    "\u{2807}", // ⠇
    "\u{280F}", // ⠏
];

pub struct Spinner {
    label: String,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn new(label: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let label_owned = label.to_string();

        let r = running.clone();
        let l = label_owned.clone();

        let handle = thread::spawn(move || {
            let mut frame_idx = 0;
            while r.load(Ordering::Relaxed) {
                let frame = BRAILLE_FRAMES[frame_idx % BRAILLE_FRAMES.len()];
                let spinner_char = style::purple(frame);
                eprint!("\r  {} {}", spinner_char, l);
                let _ = std::io::stderr().flush();
                frame_idx += 1;
                thread::sleep(Duration::from_millis(80));
            }
            eprint!("\r{}\r", " ".repeat(l.len() + 6));
            let _ = std::io::stderr().flush();
        });

        Spinner {
            label: label_owned,
            running,
            handle: Some(handle),
        }
    }

    pub fn stop_with(mut self, symbol: &str, message: &str) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        eprintln!("  {} {}", symbol, message);
    }

    pub fn fail(mut self, message: &str) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        eprintln!("  {} {}", style::red(style::CROSS), message);
    }

    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braille_frames_count() {
        assert_eq!(BRAILLE_FRAMES.len(), 10);
    }

    #[test]
    fn spinner_creates_and_stops() {
        let spinner = Spinner::new("test");
        // Give thread a moment to start
        thread::sleep(Duration::from_millis(50));
        spinner.stop_with(style::CHECK, "done");
    }

    #[test]
    fn spinner_fail_stops() {
        let spinner = Spinner::new("failing");
        thread::sleep(Duration::from_millis(50));
        spinner.fail("something went wrong");
    }
}
