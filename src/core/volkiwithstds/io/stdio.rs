//! Stdout, Stderr, Stdin — standard I/O handles.

use super::error::{IoError, Result};
use super::traits::{BufRead, Read, Write};
use crate::core::volkiwithstds::sys::errno;
use crate::core::volkiwithstds::sys::syscalls;

/// Standard output (fd 1).
pub struct Stdout;

/// Standard error (fd 2).
pub struct Stderr;

/// Standard input (fd 0).
pub struct Stdin;

/// Locked stdin with buffering.
pub struct StdinLock {
    buf: [u8; 4096],
    pos: usize,
    filled: usize,
}

impl Stdout {
    pub fn new() -> Self {
        Stdout
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        write_fd(syscalls::STDOUT_FILENO, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl core::fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Write::write_all(self, s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

impl Stderr {
    pub fn new() -> Self {
        Stderr
    }
}

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        write_fd(syscalls::STDERR_FILENO, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl core::fmt::Write for Stderr {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Write::write_all(self, s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

impl Stdin {
    pub fn new() -> Self {
        Stdin
    }

    /// Get a locked, buffered stdin.
    pub fn lock(&self) -> StdinLock {
        StdinLock {
            buf: [0u8; 4096],
            pos: 0,
            filled: 0,
        }
    }
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        read_fd(syscalls::STDIN_FILENO, buf)
    }
}

impl Read for StdinLock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let available = &self.buf[self.pos..self.filled];
        if !available.is_empty() {
            let n = buf.len().min(available.len());
            buf[..n].copy_from_slice(&available[..n]);
            self.pos += n;
            return Ok(n);
        }
        // Read directly if buffer is empty
        read_fd(syscalls::STDIN_FILENO, buf)
    }
}

impl BufRead for StdinLock {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.pos >= self.filled {
            let n = read_fd(syscalls::STDIN_FILENO, &mut self.buf)?;
            self.pos = 0;
            self.filled = n;
        }
        Ok(&self.buf[self.pos..self.filled])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = (self.pos + amt).min(self.filled);
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn write_fd(fd: i32, buf: &[u8]) -> Result<usize> {
    loop {
        let ret =
            unsafe { syscalls::write(fd, buf.as_ptr() as *const syscalls::c_void, buf.len()) };
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

fn read_fd(fd: i32, buf: &mut [u8]) -> Result<usize> {
    loop {
        let ret =
            unsafe { syscalls::read(fd, buf.as_mut_ptr() as *mut syscalls::c_void, buf.len()) };
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

/// Convenience functions matching std API.
pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}

pub fn stdin() -> Stdin {
    Stdin::new()
}
