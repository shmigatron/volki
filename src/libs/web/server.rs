//! Server — main user-facing API.

use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::net::TcpListener;
use crate::core::volkiwithstds::sync::Arc;
use crate::core::volkiwithstds::time::Duration;
use crate::core::security::https::TlsConfig;
use crate::core::security::tls::context::SslContext;
use crate::libs::web::html::document::HtmlDocument;
use crate::libs::web::html::metadata::MetadataFn;
use crate::libs::web::http::request::Request;
use crate::libs::web::http::response::Response;
use crate::libs::web::interpreter::DynamicPageData;
use crate::libs::web::reactor::event_loop::EventLoop;
use crate::libs::web::router::Router;
use crate::libs::web::router::file_route::FileRoute;
use crate::libs::web::security::{SecurityConfig, RateLimit};

pub struct Server {
    host: String,
    port: u16,
    router: Router,
    public_dir: Option<String>,
    num_workers: usize,
    tls_config: Option<TlsConfig>,
    security: SecurityConfig,
}

impl Server {
    pub fn new() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            port: 3000,
            router: Router::new(),
            public_dir: None,
            num_workers: 4,
            tls_config: None,
            security: SecurityConfig::default(),
        }
    }

    pub fn host(mut self, h: &str) -> Self {
        self.host = String::from(h);
        self
    }

    pub fn port(mut self, p: u16) -> Self {
        self.port = p;
        self
    }

    pub fn workers(mut self, n: usize) -> Self {
        self.num_workers = n;
        self
    }

    pub fn public_dir(mut self, path: &str) -> Self {
        self.public_dir = Some(String::from(path));
        self
    }

    /// Enable TLS with the given certificate and key file paths.
    pub fn tls(mut self, cert_path: &str, key_path: &str) -> Self {
        self.tls_config = Some(TlsConfig {
            cert_path: String::from(cert_path),
            key_path: String::from(key_path),
        });
        self
    }

    // ── Security builders ───────────────────────────────────────────────

    pub fn max_body_size(mut self, bytes: usize) -> Self {
        self.security.size_limits.max_body_size = bytes;
        self
    }

    pub fn max_header_size(mut self, bytes: usize) -> Self {
        self.security.size_limits.max_header_size = bytes;
        self
    }

    pub fn max_uri_length(mut self, bytes: usize) -> Self {
        self.security.size_limits.max_uri_length = bytes;
        self
    }

    pub fn read_timeout(mut self, dur: Duration) -> Self {
        self.security.timeouts.read_timeout = dur;
        self
    }

    pub fn write_timeout(mut self, dur: Duration) -> Self {
        self.security.timeouts.write_timeout = dur;
        self
    }

    pub fn keep_alive_timeout(mut self, dur: Duration) -> Self {
        self.security.timeouts.keep_alive_timeout = dur;
        self
    }

    pub fn rate_limit(mut self, requests: u32, window: Duration) -> Self {
        self.security.rate_limits.global = Some(RateLimit { requests, window });
        self
    }

    pub fn max_connections(mut self, n: usize) -> Self {
        self.security.rate_limits.max_connections = n;
        self
    }

    pub fn max_connections_per_ip(mut self, n: usize) -> Self {
        self.security.rate_limits.max_connections_per_ip = n;
        self
    }

    // ── Route builders ──────────────────────────────────────────────────

    pub fn page(mut self, pattern: &str, handler: fn(&Request) -> HtmlDocument) -> Self {
        self.router.page_route(pattern, handler);
        self
    }

    pub fn page_with_metadata(
        mut self,
        pattern: &str,
        handler: fn(&Request) -> HtmlDocument,
        metadata_fn: MetadataFn,
    ) -> Self {
        self.router.page_route_with_metadata(pattern, handler, metadata_fn);
        self
    }

    pub fn api(mut self, pattern: &str, handler: fn(&Request) -> Response) -> Self {
        self.router.api_route(pattern, handler);
        self
    }

    pub fn api_with_rate_limit(
        mut self,
        pattern: &str,
        handler: fn(&Request) -> Response,
        requests: u32,
        window: Duration,
    ) -> Self {
        self.router.api_route_with_rate_limit(pattern, handler, requests, window);
        self
    }

    /// Register a file-based route (route.rs pattern) with per-method handlers.
    pub fn file_route(mut self, pattern: &str, file_route: FileRoute) -> Self {
        self.router.file_route(pattern, file_route, true);
        self
    }

    /// Register a file-based page route (page.rs pattern) — only GET.
    pub fn file_page_route(mut self, pattern: &str, file_route: FileRoute) -> Self {
        self.router.file_route(pattern, file_route, false);
        self
    }

    pub fn not_found(mut self, handler: fn(&Request) -> Response) -> Self {
        self.router.not_found(handler);
        self
    }

    pub fn not_found_page(mut self, handler: fn(&Request) -> HtmlDocument) -> Self {
        self.router.not_found_page(handler);
        self
    }

    pub fn dynamic_page(mut self, pattern: &str, data: Arc<DynamicPageData>) -> Self {
        self.router.dynamic_page_route(pattern, data);
        self
    }

    pub fn not_found_dynamic_page(mut self, data: Arc<DynamicPageData>) -> Self {
        self.router.not_found_dynamic_page(data);
        self
    }

    pub fn listen(self) -> ! {
        let listener =
            TcpListener::bind((self.host.as_str(), self.port)).expect("failed to bind");
        listener.set_nonblocking(true).expect("failed to set non-blocking");

        let tls_ctx = if let Some(ref config) = self.tls_config {
            let ctx = SslContext::from_cert_and_key(
                config.cert_path.as_str(),
                config.key_path.as_str(),
            ).expect("failed to initialize TLS context");
            Some(ctx)
        } else {
            None
        };

        let mut event_loop = EventLoop::new(
            listener,
            self.router,
            self.num_workers,
            self.public_dir,
            tls_ctx,
            self.security,
        );

        event_loop.run()
    }
}
