//! IoError and IoErrorKind — I/O error types.

use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::sys::errno;
use core::fmt;

/// Kinds of I/O errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    AlreadyExists,
    InvalidInput,
    InvalidData,
    TimedOut,
    Interrupted,
    WouldBlock,
    BrokenPipe,
    AddrInUse,
    AddrNotAvailable,
    IsADirectory,
    NotADirectory,
    DirectoryNotEmpty,
    UnexpectedEof,
    Other,
}

impl fmt::Display for IoErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IoErrorKind::NotFound => "entity not found",
            IoErrorKind::PermissionDenied => "permission denied",
            IoErrorKind::ConnectionRefused => "connection refused",
            IoErrorKind::ConnectionReset => "connection reset",
            IoErrorKind::AlreadyExists => "entity already exists",
            IoErrorKind::InvalidInput => "invalid input parameter",
            IoErrorKind::InvalidData => "invalid data",
            IoErrorKind::TimedOut => "timed out",
            IoErrorKind::Interrupted => "operation interrupted",
            IoErrorKind::WouldBlock => "operation would block",
            IoErrorKind::BrokenPipe => "broken pipe",
            IoErrorKind::AddrInUse => "address in use",
            IoErrorKind::AddrNotAvailable => "address not available",
            IoErrorKind::IsADirectory => "is a directory",
            IoErrorKind::NotADirectory => "not a directory",
            IoErrorKind::DirectoryNotEmpty => "directory not empty",
            IoErrorKind::UnexpectedEof => "unexpected end of file",
            IoErrorKind::Other => "other error",
        };
        f.write_str(s)
    }
}

/// An I/O error.
pub struct IoError {
    kind: IoErrorKind,
    message: String,
}

impl IoError {
    /// Create a new IoError.
    pub fn new(kind: IoErrorKind, msg: &str) -> Self {
        Self {
            kind,
            message: String::from(msg),
        }
    }

    /// Create an IoError from an errno value.
    pub fn from_errno(err: i32) -> Self {
        let kind = match err {
            errno::ENOENT => IoErrorKind::NotFound,
            errno::EACCES => IoErrorKind::PermissionDenied,
            errno::EEXIST => IoErrorKind::AlreadyExists,
            errno::EINTR => IoErrorKind::Interrupted,
            errno::EINVAL => IoErrorKind::InvalidInput,
            errno::EPIPE => IoErrorKind::BrokenPipe,
            errno::EISDIR => IoErrorKind::IsADirectory,
            errno::ENOTDIR => IoErrorKind::NotADirectory,
            errno::ENOTEMPTY => IoErrorKind::DirectoryNotEmpty,
            errno::ECONNREFUSED => IoErrorKind::ConnectionRefused,
            errno::ETIMEDOUT => IoErrorKind::TimedOut,
            errno::EADDRINUSE => IoErrorKind::AddrInUse,
            errno::EADDRNOTAVAIL => IoErrorKind::AddrNotAvailable,
            errno::EAGAIN => IoErrorKind::WouldBlock,
            errno::ECONNRESET => IoErrorKind::ConnectionReset,
            errno::ENOTCONN => IoErrorKind::ConnectionReset,
            _ => IoErrorKind::Other,
        };
        let mut msg = String::from("errno ");
        // Simple integer-to-string for the errno value
        let mut buf = [0u8; 20];
        let s = int_to_str(err, &mut buf);
        msg.push_str(s);
        Self { kind, message: msg }
    }

    /// Create an IoError from the current errno.
    pub fn last_os_error() -> Self {
        Self::from_errno(errno::get_errno())
    }

    /// Returns the error kind.
    pub fn kind(&self) -> IoErrorKind {
        self.kind
    }
}

impl fmt::Debug for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IoError({:?}, \"{}\")", self.kind, self.message)
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl Clone for IoError {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            message: self.message.clone(),
        }
    }
}

/// Simple i32 to string conversion.
fn int_to_str(mut val: i32, buf: &mut [u8; 20]) -> &str {
    let negative = val < 0;
    if negative {
        val = -val;
    }
    let mut pos = 20;
    if val == 0 {
        pos -= 1;
        buf[pos] = b'0';
    } else {
        while val > 0 {
            pos -= 1;
            buf[pos] = b'0' + (val % 10) as u8;
            val /= 10;
        }
    }
    if negative {
        pos -= 1;
        buf[pos] = b'-';
    }
    unsafe { core::str::from_utf8_unchecked(&buf[pos..]) }
}

/// Type alias for I/O Results.
pub type Result<T> = core::result::Result<T, IoError>;
