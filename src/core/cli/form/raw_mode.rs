use crate::core::cli::error::CliError;
use crate::core::volkiwithstds::collections::String;

#[cfg(unix)]
mod platform {
    // termios struct layout differs between macOS and Linux.
    // macOS: tcflag_t = u64, cc_t = u8, speed_t = u64, NCCS = 20
    // Linux: tcflag_t = u32, cc_t = u8, speed_t = u32, NCCS = 32

    #[cfg(target_os = "macos")]
    pub const NCCS: usize = 20;
    #[cfg(target_os = "linux")]
    pub const NCCS: usize = 32;

    #[cfg(target_os = "macos")]
    type TcflagT = u64;
    #[cfg(target_os = "linux")]
    type TcflagT = u32;

    #[cfg(target_os = "macos")]
    type SpeedT = u64;
    #[cfg(target_os = "linux")]
    type SpeedT = u32;

    #[repr(C)]
    #[derive(Clone)]
    pub struct Termios {
        pub c_iflag: TcflagT,
        pub c_oflag: TcflagT,
        pub c_cflag: TcflagT,
        pub c_lflag: TcflagT,
        pub c_cc: [u8; NCCS],
        #[cfg(target_os = "macos")]
        pub c_ispeed: SpeedT,
        #[cfg(target_os = "macos")]
        pub c_ospeed: SpeedT,
        #[cfg(target_os = "linux")]
        pub c_line: u8,
        #[cfg(target_os = "linux")]
        pub c_ispeed: SpeedT,
        #[cfg(target_os = "linux")]
        pub c_ospeed: SpeedT,
    }

    // Local mode flags
    #[cfg(target_os = "macos")]
    pub const ICANON: TcflagT = 0x00000100;
    #[cfg(target_os = "linux")]
    pub const ICANON: TcflagT = 0x00000002;

    #[cfg(target_os = "macos")]
    pub const ECHO: TcflagT = 0x00000008;
    #[cfg(target_os = "linux")]
    pub const ECHO: TcflagT = 0x00000008;

    #[cfg(target_os = "macos")]
    pub const ISIG: TcflagT = 0x00000080;
    #[cfg(target_os = "linux")]
    pub const ISIG: TcflagT = 0x00000001;

    // cc indices
    pub const VMIN: usize = 16;
    #[cfg(target_os = "linux")]
    pub const VMIN_LINUX: usize = 6;

    pub const VTIME: usize = 17;
    #[cfg(target_os = "linux")]
    pub const VTIME_LINUX: usize = 5;

    // tcsetattr actions
    pub const TCSAFLUSH: i32 = 2;

    unsafe extern "C" {
        #[link_name = "tcgetattr"]
        pub fn tcgetattr(fd: i32, termios: *mut Termios) -> i32;
        #[link_name = "tcsetattr"]
        pub fn tcsetattr(fd: i32, action: i32, termios: *const Termios) -> i32;
    }

    pub fn vmin_idx() -> usize {
        #[cfg(target_os = "macos")]
        {
            VMIN
        }
        #[cfg(target_os = "linux")]
        {
            VMIN_LINUX
        }
    }

    pub fn vtime_idx() -> usize {
        #[cfg(target_os = "macos")]
        {
            VTIME
        }
        #[cfg(target_os = "linux")]
        {
            VTIME_LINUX
        }
    }
}

#[cfg(unix)]
pub struct RawModeGuard {
    original: platform::Termios,
    fd: i32,
}

#[cfg(unix)]
impl RawModeGuard {
    pub fn enter() -> Result<Self, CliError> {
        use core::mem::MaybeUninit;

        let fd = 0; // stdin
        let original = unsafe {
            let mut termios = MaybeUninit::<platform::Termios>::uninit();
            if platform::tcgetattr(fd, termios.as_mut_ptr()) != 0 {
                return Err(CliError::InvalidUsage(String::from(
                    "failed to get terminal attributes",
                )));
            }
            termios.assume_init()
        };

        let mut raw = original.clone();
        raw.c_lflag &= !(platform::ICANON | platform::ECHO | platform::ISIG);
        raw.c_cc[platform::vmin_idx()] = 1;
        raw.c_cc[platform::vtime_idx()] = 1; // 100ms timeout for escape sequences

        let result = unsafe { platform::tcsetattr(fd, platform::TCSAFLUSH, &raw) };
        if result != 0 {
            return Err(CliError::InvalidUsage(String::from(
                "failed to set raw mode",
            )));
        }

        Ok(RawModeGuard { original, fd })
    }

    fn restore(&self) {
        unsafe {
            platform::tcsetattr(self.fd, platform::TCSAFLUSH, &self.original);
        }
    }
}

#[cfg(unix)]
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        self.restore();
    }
}

#[cfg(not(unix))]
pub struct RawModeGuard;

#[cfg(not(unix))]
impl RawModeGuard {
    pub fn enter() -> Result<Self, CliError> {
        Err(CliError::InvalidUsage(String::from(
            "interactive forms require a unix terminal",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_mode_guard_type_exists() {
        // Ensure the type compiles and is usable
        let _: fn() -> Result<RawModeGuard, CliError> = RawModeGuard::enter;
    }

    #[cfg(unix)]
    #[test]
    fn platform_constants_are_nonzero() {
        assert!(platform::ICANON != 0);
        assert!(platform::ECHO != 0);
        assert!(platform::ISIG != 0);
    }

    #[cfg(unix)]
    #[test]
    fn vmin_vtime_indices_in_bounds() {
        assert!(platform::vmin_idx() < platform::NCCS);
        assert!(platform::vtime_idx() < platform::NCCS);
    }
}
