//! SslContext — safe wrapper around `SSL_CTX*`.

use crate::core::volkiwithstds::path::CString;
use crate::core::volkiwithstds::sys::openssl;
use super::error::{TlsError, get_openssl_error};

/// Wraps an `SSL_CTX*` with RAII cleanup.
pub struct SslContext {
    ctx: *mut openssl::SSL_CTX,
}

unsafe impl Send for SslContext {}
unsafe impl Sync for SslContext {}

impl SslContext {
    /// Create a new server-side SSL context with TLSv1.2+ only.
    pub fn new_server() -> Result<Self, TlsError> {
        unsafe {
            // Initialize OpenSSL
            openssl::OPENSSL_init_ssl(
                openssl::OPENSSL_INIT_LOAD_SSL_STRINGS
                    | openssl::OPENSSL_INIT_LOAD_CRYPTO_STRINGS,
                core::ptr::null(),
            );

            let method = openssl::TLS_server_method();
            if method.is_null() {
                return Err(TlsError::InitFailed);
            }

            let ctx = openssl::SSL_CTX_new(method);
            if ctx.is_null() {
                return Err(TlsError::InitFailed);
            }

            // Disable old protocols — only TLSv1.2+ allowed
            openssl::SSL_CTX_set_options(
                ctx,
                openssl::SSL_OP_NO_SSLv2
                    | openssl::SSL_OP_NO_SSLv3
                    | openssl::SSL_OP_NO_TLSv1
                    | openssl::SSL_OP_NO_TLSv1_1,
            );

            Ok(Self { ctx })
        }
    }

    /// Load a PEM certificate file.
    pub fn load_cert_file(&self, path: &str) -> Result<(), TlsError> {
        let c_path = CString::new(path);
        unsafe {
            openssl::ERR_clear_error();
            let ret = openssl::SSL_CTX_use_certificate_file(
                self.ctx,
                c_path.as_ptr(),
                openssl::SSL_FILETYPE_PEM,
            );
            if ret != 1 {
                return Err(TlsError::CertLoadFailed(get_openssl_error()));
            }
        }
        Ok(())
    }

    /// Load a PEM private key file.
    pub fn load_key_file(&self, path: &str) -> Result<(), TlsError> {
        let c_path = CString::new(path);
        unsafe {
            openssl::ERR_clear_error();
            let ret = openssl::SSL_CTX_use_PrivateKey_file(
                self.ctx,
                c_path.as_ptr(),
                openssl::SSL_FILETYPE_PEM,
            );
            if ret != 1 {
                return Err(TlsError::KeyLoadFailed(get_openssl_error()));
            }
        }
        Ok(())
    }

    /// Verify that the loaded private key matches the certificate.
    pub fn check_private_key(&self) -> Result<(), TlsError> {
        unsafe {
            if openssl::SSL_CTX_check_private_key(self.ctx) != 1 {
                return Err(TlsError::KeyMismatch);
            }
        }
        Ok(())
    }

    /// Create a new per-connection SSL object from this context.
    pub fn new_ssl(&self) -> Result<*mut openssl::SSL, TlsError> {
        unsafe {
            openssl::ERR_clear_error();
            let ssl = openssl::SSL_new(self.ctx);
            if ssl.is_null() {
                return Err(TlsError::InitFailed);
            }
            Ok(ssl)
        }
    }

    /// Convenience: create a context, load cert + key, verify match.
    pub fn from_cert_and_key(cert_path: &str, key_path: &str) -> Result<Self, TlsError> {
        let ctx = Self::new_server()?;
        ctx.load_cert_file(cert_path)?;
        ctx.load_key_file(key_path)?;
        ctx.check_private_key()?;
        Ok(ctx)
    }
}

impl Drop for SslContext {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe {
                openssl::SSL_CTX_free(self.ctx);
            }
        }
    }
}
