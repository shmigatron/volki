//! errno access — platform-specific.

use super::syscalls::c_int;

unsafe extern "C" {
    #[cfg(target_os = "macos")]
    fn __error() -> *mut c_int;

    #[cfg(target_os = "linux")]
    fn __errno_location() -> *mut c_int;
}

/// Returns the current errno value.
pub fn get_errno() -> c_int {
    unsafe {
        #[cfg(target_os = "macos")]
        {
            *__error()
        }
        #[cfg(target_os = "linux")]
        {
            *__errno_location()
        }
    }
}

/// Sets errno to the given value.
pub fn set_errno(val: c_int) {
    unsafe {
        #[cfg(target_os = "macos")]
        {
            *__error() = val;
        }
        #[cfg(target_os = "linux")]
        {
            *__errno_location() = val;
        }
    }
}

pub const ENOENT: c_int = 2;
pub const EINTR: c_int = 4;
pub const EIO: c_int = 5;
pub const EACCES: c_int = 13;
pub const EEXIST: c_int = 17;
pub const ENOTDIR: c_int = 20;
pub const EISDIR: c_int = 21;
pub const EINVAL: c_int = 22;
pub const EPIPE: c_int = 32;
#[cfg(target_os = "macos")]
pub const ENOTEMPTY: c_int = 66;
#[cfg(target_os = "linux")]
pub const ENOTEMPTY: c_int = 39;

#[cfg(target_os = "macos")]
pub const ECONNREFUSED: c_int = 61;
#[cfg(target_os = "linux")]
pub const ECONNREFUSED: c_int = 111;

#[cfg(target_os = "macos")]
pub const ETIMEDOUT: c_int = 60;
#[cfg(target_os = "linux")]
pub const ETIMEDOUT: c_int = 110;

#[cfg(target_os = "macos")]
pub const EADDRINUSE: c_int = 48;
#[cfg(target_os = "linux")]
pub const EADDRINUSE: c_int = 98;

#[cfg(target_os = "macos")]
pub const EADDRNOTAVAIL: c_int = 49;
#[cfg(target_os = "linux")]
pub const EADDRNOTAVAIL: c_int = 99;

#[cfg(target_os = "macos")]
pub const EAGAIN: c_int = 35;
#[cfg(target_os = "linux")]
pub const EAGAIN: c_int = 11;

#[cfg(target_os = "macos")]
pub const ECONNRESET: c_int = 54;
#[cfg(target_os = "linux")]
pub const ECONNRESET: c_int = 104;

#[cfg(target_os = "macos")]
pub const ENOTCONN: c_int = 57;
#[cfg(target_os = "linux")]
pub const ENOTCONN: c_int = 107;
