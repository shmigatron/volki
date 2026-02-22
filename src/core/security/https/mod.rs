//! HTTPS configuration.

use crate::core::volkiwithstds::collections::String;

/// TLS configuration for the web server.
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}
