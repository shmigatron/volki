//! Thread sleep via nanosleep.

use crate::core::volkiwithstds::sys::syscalls;
use crate::core::volkiwithstds::time::Duration;

/// Sleep for the given duration.
pub fn sleep(duration: Duration) {
    let req = syscalls::timespec {
        tv_sec: duration.as_secs() as syscalls::c_long,
        tv_nsec: duration.subsec_nanos() as syscalls::c_long,
    };
    unsafe {
        syscalls::nanosleep(&req, core::ptr::null_mut());
    }
}
