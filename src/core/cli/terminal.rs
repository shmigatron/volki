#[cfg(unix)]
pub fn is_tty() -> bool {
    unsafe { libc_isatty(2) != 0 }
}

#[cfg(not(unix))]
pub fn is_tty() -> bool {
    false
}

#[cfg(unix)]
unsafe extern "C" {
    #[link_name = "isatty"]
    fn libc_isatty(fd: i32) -> i32;
}

pub fn is_ci() -> bool {
    std::env::var_os("CI").is_some()
        || std::env::var_os("GITHUB_ACTIONS").is_some()
        || std::env::var_os("GITLAB_CI").is_some()
        || std::env::var_os("CIRCLECI").is_some()
        || std::env::var_os("TRAVIS").is_some()
        || std::env::var_os("JENKINS_URL").is_some()
        || std::env::var_os("BUILDKITE").is_some()
        || std::env::var_os("TF_BUILD").is_some()
}

pub fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

#[cfg(unix)]
pub fn terminal_width() -> usize {
    use std::mem::MaybeUninit;

    #[repr(C)]
    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16,
        ws_ypixel: u16,
    }

    // TIOCGWINSZ on macOS = 0x40087468, on Linux = 0x5413
    #[cfg(target_os = "macos")]
    const TIOCGWINSZ: u64 = 0x40087468;
    #[cfg(target_os = "linux")]
    const TIOCGWINSZ: u64 = 0x5413;

    unsafe {
        let mut ws = MaybeUninit::<Winsize>::uninit();
        if libc_ioctl(2, TIOCGWINSZ, ws.as_mut_ptr()) == 0 {
            let ws = ws.assume_init();
            if ws.ws_col > 0 {
                return ws.ws_col as usize;
            }
        }
    }
    80
}

#[cfg(not(unix))]
pub fn terminal_width() -> usize {
    80
}

#[cfg(unix)]
unsafe extern "C" {
    #[link_name = "ioctl"]
    fn libc_ioctl(fd: i32, request: u64, ...) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_width_is_reasonable() {
        let w = terminal_width();
        assert!(w >= 20 && w <= 500);
    }

    #[test]
    fn is_ci_returns_bool() {
        // Just ensure it doesn't panic
        let _ = is_ci();
    }

    #[test]
    fn no_color_returns_bool() {
        let _ = no_color();
    }

    #[test]
    fn is_tty_returns_bool() {
        let _ = is_tty();
    }
}
