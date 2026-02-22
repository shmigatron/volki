//! Security configuration — size limits, timeouts, rate limiting.

use crate::core::volkiwithstds::time::Duration;

/// Maximum allowed sizes for request components.
pub struct SizeLimits {
    pub max_header_size: usize,
    pub max_body_size: usize,
    pub max_uri_length: usize,
}

impl Default for SizeLimits {
    fn default() -> Self {
        Self {
            max_header_size: 8 * 1024,         // 8KB
            max_body_size: 10 * 1024 * 1024,   // 10MB
            max_uri_length: 8192,
        }
    }
}

/// Timeout durations for various connection phases.
pub struct TimeoutConfig {
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub keep_alive_timeout: Duration,
    pub handshake_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            keep_alive_timeout: Duration::from_secs(60),
            handshake_timeout: Duration::from_secs(10),
        }
    }
}

/// A rate limit rule: max requests within a time window.
pub struct RateLimit {
    pub requests: u32,
    pub window: Duration,
}

/// Rate limiting and connection limit configuration.
pub struct RateLimitConfig {
    pub global: Option<RateLimit>,
    pub max_connections: usize,
    pub max_connections_per_ip: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            global: None,
            max_connections: 1024,
            max_connections_per_ip: 64,
        }
    }
}

/// Top-level security configuration.
pub struct SecurityConfig {
    pub size_limits: SizeLimits,
    pub timeouts: TimeoutConfig,
    pub rate_limits: RateLimitConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            size_limits: SizeLimits::default(),
            timeouts: TimeoutConfig::default(),
            rate_limits: RateLimitConfig::default(),
        }
    }
}
