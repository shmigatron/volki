//! OpenSSL FFI bindings — raw `extern "C"` declarations for `libssl` + `libcrypto`.

#![allow(non_camel_case_types, non_upper_case_globals, dead_code)]

use super::syscalls::{c_char, c_int, c_long, size_t};
pub use super::syscalls::c_void;

// ── Opaque types ────────────────────────────────────────────────────────────

#[repr(C)]
pub struct SSL_CTX {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct SSL {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct SSL_METHOD {
    _opaque: [u8; 0],
}

// ── Constants ───────────────────────────────────────────────────────────────

pub const SSL_FILETYPE_PEM: c_int = 1;

// SSL_get_error return values
pub const SSL_ERROR_NONE: c_int = 0;
pub const SSL_ERROR_SSL: c_int = 1;
pub const SSL_ERROR_WANT_READ: c_int = 2;
pub const SSL_ERROR_WANT_WRITE: c_int = 3;
pub const SSL_ERROR_WANT_X509_LOOKUP: c_int = 4;
pub const SSL_ERROR_SYSCALL: c_int = 5;
pub const SSL_ERROR_ZERO_RETURN: c_int = 6;
pub const SSL_ERROR_WANT_CONNECT: c_int = 7;
pub const SSL_ERROR_WANT_ACCEPT: c_int = 8;

// SSL_CTX_set_options flags — disable old protocols
pub const SSL_OP_NO_SSLv2: c_long = 0x01000000;
pub const SSL_OP_NO_SSLv3: c_long = 0x02000000;
pub const SSL_OP_NO_TLSv1: c_long = 0x04000000;
pub const SSL_OP_NO_TLSv1_1: c_long = 0x10000000;

// OPENSSL_init_ssl flags
pub const OPENSSL_INIT_LOAD_SSL_STRINGS: u64 = 0x00200000;
pub const OPENSSL_INIT_LOAD_CRYPTO_STRINGS: u64 = 0x00000002;

// ── extern "C" declarations ─────────────────────────────────────────────────

#[link(name = "ssl")]
#[link(name = "crypto")]
unsafe extern "C" {
    // Initialization
    pub fn OPENSSL_init_ssl(opts: u64, settings: *const c_void) -> c_int;

    // Method
    pub fn TLS_server_method() -> *const SSL_METHOD;

    // SSL_CTX
    pub fn SSL_CTX_new(method: *const SSL_METHOD) -> *mut SSL_CTX;
    pub fn SSL_CTX_free(ctx: *mut SSL_CTX);
    pub fn SSL_CTX_use_certificate_file(
        ctx: *mut SSL_CTX,
        file: *const c_char,
        typ: c_int,
    ) -> c_int;
    pub fn SSL_CTX_use_PrivateKey_file(
        ctx: *mut SSL_CTX,
        file: *const c_char,
        typ: c_int,
    ) -> c_int;
    pub fn SSL_CTX_check_private_key(ctx: *const SSL_CTX) -> c_int;
    pub fn SSL_CTX_set_options(ctx: *mut SSL_CTX, options: c_long) -> c_long;

    // SSL (per-connection)
    pub fn SSL_new(ctx: *mut SSL_CTX) -> *mut SSL;
    pub fn SSL_free(ssl: *mut SSL);
    pub fn SSL_set_fd(ssl: *mut SSL, fd: c_int) -> c_int;
    pub fn SSL_accept(ssl: *mut SSL) -> c_int;
    pub fn SSL_read(ssl: *mut SSL, buf: *mut c_void, num: c_int) -> c_int;
    pub fn SSL_write(ssl: *mut SSL, buf: *const c_void, num: c_int) -> c_int;
    pub fn SSL_shutdown(ssl: *mut SSL) -> c_int;
    pub fn SSL_get_error(ssl: *const SSL, ret: c_int) -> c_int;

    // Error queue
    pub fn ERR_get_error() -> c_long;
    pub fn ERR_error_string_n(e: c_long, buf: *mut c_char, len: size_t);
    pub fn ERR_clear_error();
}
