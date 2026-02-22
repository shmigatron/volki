//! TLS stream helpers — free functions operating on raw `*mut SSL`.

use crate::core::volkiwithstds::io::error::IoError;
use crate::core::volkiwithstds::sys::openssl;
use super::error::{TlsError, get_openssl_error};

/// Bind an SSL object to a socket file descriptor.
pub fn ssl_set_fd(ssl: *mut openssl::SSL, fd: i32) -> Result<(), TlsError> {
    unsafe {
        if openssl::SSL_set_fd(ssl, fd) != 1 {
            return Err(TlsError::InitFailed);
        }
    }
    Ok(())
}

/// Perform a non-blocking TLS server handshake.
/// Returns `Ok(true)` when the handshake is complete,
/// `Err(WantRead)` or `Err(WantWrite)` when it needs to be retried.
pub fn ssl_accept(ssl: *mut openssl::SSL) -> Result<bool, TlsError> {
    unsafe {
        openssl::ERR_clear_error();
        let ret = openssl::SSL_accept(ssl);
        if ret == 1 {
            return Ok(true);
        }
        let err = openssl::SSL_get_error(ssl, ret);
        match err {
            openssl::SSL_ERROR_WANT_READ => Err(TlsError::WantRead),
            openssl::SSL_ERROR_WANT_WRITE => Err(TlsError::WantWrite),
            openssl::SSL_ERROR_ZERO_RETURN => Err(TlsError::ConnectionClosed),
            openssl::SSL_ERROR_SYSCALL => {
                let io_err = IoError::last_os_error();
                Err(TlsError::SyscallError(io_err))
            }
            _ => Err(TlsError::HandshakeFailed(get_openssl_error())),
        }
    }
}

/// Read decrypted data from a TLS connection.
/// Returns number of bytes read, or a TLS error.
pub fn ssl_read(ssl: *mut openssl::SSL, buf: &mut [u8]) -> Result<usize, TlsError> {
    unsafe {
        openssl::ERR_clear_error();
        let ret = openssl::SSL_read(
            ssl,
            buf.as_mut_ptr() as *mut openssl::c_void,
            buf.len() as i32,
        );
        if ret > 0 {
            return Ok(ret as usize);
        }
        let err = openssl::SSL_get_error(ssl, ret);
        match err {
            openssl::SSL_ERROR_WANT_READ => Err(TlsError::WantRead),
            openssl::SSL_ERROR_WANT_WRITE => Err(TlsError::WantWrite),
            openssl::SSL_ERROR_ZERO_RETURN => Err(TlsError::ConnectionClosed),
            openssl::SSL_ERROR_SYSCALL => {
                if ret == 0 {
                    Err(TlsError::ConnectionClosed)
                } else {
                    Err(TlsError::SyscallError(IoError::last_os_error()))
                }
            }
            _ => Err(TlsError::HandshakeFailed(get_openssl_error())),
        }
    }
}

/// Write data through a TLS connection (encrypt + send).
/// Returns number of bytes written, or a TLS error.
pub fn ssl_write(ssl: *mut openssl::SSL, buf: &[u8]) -> Result<usize, TlsError> {
    unsafe {
        openssl::ERR_clear_error();
        let ret = openssl::SSL_write(
            ssl,
            buf.as_ptr() as *const openssl::c_void,
            buf.len() as i32,
        );
        if ret > 0 {
            return Ok(ret as usize);
        }
        let err = openssl::SSL_get_error(ssl, ret);
        match err {
            openssl::SSL_ERROR_WANT_READ => Err(TlsError::WantRead),
            openssl::SSL_ERROR_WANT_WRITE => Err(TlsError::WantWrite),
            openssl::SSL_ERROR_ZERO_RETURN => Err(TlsError::ConnectionClosed),
            openssl::SSL_ERROR_SYSCALL => {
                Err(TlsError::SyscallError(IoError::last_os_error()))
            }
            _ => Err(TlsError::HandshakeFailed(get_openssl_error())),
        }
    }
}

/// Shut down a TLS connection (sends close_notify).
pub fn ssl_shutdown(ssl: *mut openssl::SSL) {
    unsafe {
        openssl::SSL_shutdown(ssl);
    }
}

/// Free an SSL object.
pub fn ssl_free(ssl: *mut openssl::SSL) {
    if !ssl.is_null() {
        unsafe {
            openssl::SSL_free(ssl);
        }
    }
}
