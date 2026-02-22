//! Fd — owned file descriptor wrapper.

use super::error::{IoError, Result};
use super::traits::{Read, Write};
use crate::core::volkiwithstds::sys::errno;
use crate::core::volkiwithstds::sys::syscalls;

/// An owned file descriptor. Closes on drop.
pub struct Fd {
    raw: i32,
}

impl Fd {
    /// Wrap a raw file descriptor. Takes ownership.
    pub fn from_raw(fd: i32) -> Self {
        Self { raw: fd }
    }

    /// Returns the raw fd without consuming.
    pub fn as_raw(&self) -> i32 {
        self.raw
    }

    /// Consumes the Fd without closing.
    pub fn into_raw(self) -> i32 {
        let fd = self.raw;
        core::mem::forget(self);
        fd
    }
}

impl Read for Fd {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        loop {
            let ret = unsafe {
                syscalls::read(
                    self.raw,
                    buf.as_mut_ptr() as *mut syscalls::c_void,
                    buf.len(),
                )
            };
            if ret < 0 {
                let err = errno::get_errno();
                if err == errno::EINTR {
                    continue;
                }
                return Err(IoError::from_errno(err));
            }
            return Ok(ret as usize);
        }
    }
}

impl Write for Fd {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        loop {
            let ret = unsafe {
                syscalls::write(self.raw, buf.as_ptr() as *const syscalls::c_void, buf.len())
            };
            if ret < 0 {
                let err = errno::get_errno();
                if err == errno::EINTR {
                    continue;
                }
                return Err(IoError::from_errno(err));
            }
            return Ok(ret as usize);
        }
    }

    fn flush(&mut self) -> Result<()> {
        Ok(()) // raw fds don't buffer
    }
}

impl Drop for Fd {
    fn drop(&mut self) {
        if self.raw >= 0 {
            unsafe {
                syscalls::close(self.raw);
            }
        }
    }
}
