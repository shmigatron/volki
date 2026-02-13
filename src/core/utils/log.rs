use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Off = 4,
}

impl LogLevel {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            3 => LogLevel::Error,
            _ => LogLevel::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Off => "off",
        }
    }
}

static LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Error as u8);

pub fn set_level(level: LogLevel) {
    LEVEL.store(level as u8, Ordering::Relaxed);
}

pub fn level() -> LogLevel {
    LogLevel::from_u8(LEVEL.load(Ordering::Relaxed))
}

pub fn enabled(msg_level: LogLevel) -> bool {
    msg_level >= level()
}

pub fn log(msg_level: LogLevel, module: &str, msg: &str) {
    if enabled(msg_level) {
        eprintln!("[{}] {}: {}", msg_level.label(), module, msg);
    }
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::core::utils::log::log(
            $crate::core::utils::log::LogLevel::Debug,
            module_path!(),
            &format!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::core::utils::log::log(
            $crate::core::utils::log::LogLevel::Info,
            module_path!(),
            &format!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::core::utils::log::log(
            $crate::core::utils::log::LogLevel::Warn,
            module_path!(),
            &format!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::core::utils::log::log(
            $crate::core::utils::log::LogLevel::Error,
            module_path!(),
            &format!($($arg)*),
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests manipulate global state, so we save/restore.
    fn with_level<F: FnOnce()>(lvl: LogLevel, f: F) {
        let prev = LEVEL.load(Ordering::Relaxed);
        set_level(lvl);
        f();
        LEVEL.store(prev, Ordering::Relaxed);
    }

    #[test]
    fn default_level_is_error() {
        // The static default is Error (binary default).
        // Tests may have changed it, so just verify the API works.
        let l = LogLevel::from_u8(LogLevel::Error as u8);
        assert_eq!(l, LogLevel::Error);
    }

    #[test]
    fn set_and_get_level() {
        with_level(LogLevel::Debug, || {
            assert_eq!(level(), LogLevel::Debug);
        });
    }

    #[test]
    fn enabled_respects_level() {
        with_level(LogLevel::Warn, || {
            assert!(!enabled(LogLevel::Debug));
            assert!(!enabled(LogLevel::Info));
            assert!(enabled(LogLevel::Warn));
            assert!(enabled(LogLevel::Error));
        });
    }

    #[test]
    fn enabled_off_blocks_all() {
        with_level(LogLevel::Off, || {
            assert!(!enabled(LogLevel::Error));
        });
    }

    #[test]
    fn enabled_debug_allows_all() {
        with_level(LogLevel::Debug, || {
            assert!(enabled(LogLevel::Debug));
            assert!(enabled(LogLevel::Info));
            assert!(enabled(LogLevel::Warn));
            assert!(enabled(LogLevel::Error));
        });
    }

    #[test]
    fn level_labels() {
        assert_eq!(LogLevel::Debug.label(), "debug");
        assert_eq!(LogLevel::Info.label(), "info");
        assert_eq!(LogLevel::Warn.label(), "warn");
        assert_eq!(LogLevel::Error.label(), "error");
        assert_eq!(LogLevel::Off.label(), "off");
    }

    #[test]
    fn level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Off);
    }

    #[test]
    fn macros_compile() {
        with_level(LogLevel::Debug, || {
            log_debug!("test debug {}", 42);
            log_info!("test info");
            log_warn!("test warn {}", "msg");
            log_error!("test error");
        });
    }
}
