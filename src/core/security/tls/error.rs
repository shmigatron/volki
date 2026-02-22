//! TLS error types.

use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::io::error::IoError;
use crate::core::volkiwithstds::sys::openssl;
use core::fmt;

/// Errors that can occur during TLS operations.
pub enum TlsError {
    InitFailed,
    CertLoadFailed(String),
    KeyLoadFailed(String),
    KeyMismatch,
    HandshakeFailed(String),
    WantRead,
    WantWrite,
    ConnectionClosed,
    SyscallError(IoError),
}

impl fmt::Debug for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlsError::InitFailed => f.write_str("TlsError::InitFailed"),
            TlsError::CertLoadFailed(s) => write!(f, "TlsError::CertLoadFailed(\"{}\")", s),
            TlsError::KeyLoadFailed(s) => write!(f, "TlsError::KeyLoadFailed(\"{}\")", s),
            TlsError::KeyMismatch => f.write_str("TlsError::KeyMismatch"),
            TlsError::HandshakeFailed(s) => write!(f, "TlsError::HandshakeFailed(\"{}\")", s),
            TlsError::WantRead => f.write_str("TlsError::WantRead"),
            TlsError::WantWrite => f.write_str("TlsError::WantWrite"),
            TlsError::ConnectionClosed => f.write_str("TlsError::ConnectionClosed"),
            TlsError::SyscallError(e) => write!(f, "TlsError::SyscallError({:?})", e),
        }
    }
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlsError::InitFailed => f.write_str("TLS initialization failed"),
            TlsError::CertLoadFailed(s) => write!(f, "failed to load certificate: {}", s),
            TlsError::KeyLoadFailed(s) => write!(f, "failed to load private key: {}", s),
            TlsError::KeyMismatch => f.write_str("private key does not match certificate"),
            TlsError::HandshakeFailed(s) => write!(f, "TLS handshake failed: {}", s),
            TlsError::WantRead => f.write_str("TLS wants read"),
            TlsError::WantWrite => f.write_str("TLS wants write"),
            TlsError::ConnectionClosed => f.write_str("TLS connection closed"),
            TlsError::SyscallError(e) => write!(f, "TLS syscall error: {}", e),
        }
    }
}

/// Drain the OpenSSL error queue and return a human-readable string.
pub fn get_openssl_error() -> String {
    let code = unsafe { openssl::ERR_get_error() };
    if code == 0 {
        return String::from("unknown error");
    }
    let mut buf = [0i8; 256];
    unsafe {
        openssl::ERR_error_string_n(code, buf.as_mut_ptr(), buf.len());
    }
    // Convert the C string to our String
    let mut len = 0usize;
    while len < buf.len() && buf[len] != 0 {
        len += 1;
    }
    let slice = unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const u8, len) };
    let s = unsafe { core::str::from_utf8_unchecked(slice) };
    String::from(s)
}
