//! Main event loop — accept, read, dispatch, write.

use super::connection::{ConnState, Connection, HandshakeResult};
use super::poll::{Event, Interest, Poller};
use super::pool::{Job, ThreadPool, log_request};
use crate::core::volkiwithstds::collections::{HashMap, Vec, VecDeque};
use crate::core::volkiwithstds::io::error::IoErrorKind;
use crate::core::volkiwithstds::net::{TcpListener, peer_ip_from_fd};
use crate::core::volkiwithstds::time::{Duration, Instant};
use crate::core::security::tls::context::SslContext;
use crate::core::security::tls::stream::ssl_set_fd;
use crate::libs::web::http::parser::{ParseResult, parse_request};
use crate::libs::web::http::response::Response;
use crate::libs::web::http::status::StatusCode;
use crate::libs::web::router::Router;
use crate::libs::web::security::{SecurityConfig, RateLimit};
use crate::libs::web::static_files::server::try_serve_static;
use crate::core::volkiwithstds::sys::syscalls;

pub struct EventLoop {
    listener: TcpListener,
    poller: Poller,
    pool: ThreadPool,
    connections: HashMap<i32, Connection>,
    router: Router,
    public_dir: Option<crate::core::volkiwithstds::collections::String>,
    tls_ctx: Option<SslContext>,
    security: SecurityConfig,
    ip_conn_counts: HashMap<u32, usize>,
    rate_tracker: HashMap<u64, VecDeque<Instant>>,
    last_sweep: Instant,
}

impl EventLoop {
    pub fn new(
        listener: TcpListener,
        router: Router,
        num_workers: usize,
        public_dir: Option<crate::core::volkiwithstds::collections::String>,
        tls_ctx: Option<SslContext>,
        security: SecurityConfig,
    ) -> Self {
        let poller = Poller::new().expect("failed to create poller");
        let pool = ThreadPool::new(num_workers);

        // Register listener for read events
        poller
            .register(listener.as_raw_fd(), Interest::Read)
            .expect("failed to register listener");

        Self {
            listener,
            poller,
            pool,
            connections: HashMap::new(),
            router,
            public_dir,
            tls_ctx,
            security,
            ip_conn_counts: HashMap::new(),
            rate_tracker: HashMap::new(),
            last_sweep: Instant::now(),
        }
    }

    pub fn run(&mut self) -> ! {
        let mut events = [Event {
            fd: 0,
            readable: false,
            writable: false,
            error: false,
            hangup: false,
        }; 256];

        loop {
            // Poll with 10ms timeout so we can drain worker results
            let count = match self.poller.poll(&mut events, 10) {
                Ok(n) => n,
                Err(e) => {
                    if e.kind() == IoErrorKind::Interrupted {
                        continue;
                    }
                    panic!("poll error: {:?}", e);
                }
            };

            let listener_fd = self.listener.as_raw_fd();

            for i in 0..count {
                let ev = &events[i];
                if ev.fd == listener_fd {
                    self.accept_connections();
                } else {
                    if ev.readable {
                        self.handle_readable(ev.fd);
                    }
                    if ev.writable {
                        self.handle_writable(ev.fd);
                    }
                    if ev.error || ev.hangup {
                        self.close_connection(ev.fd);
                    }
                }
            }

            // Drain worker results
            self.drain_results();

            // Sweep timeouts every 500ms
            if self.last_sweep.elapsed() >= Duration::from_millis(500) {
                self.sweep_timeouts();
                self.last_sweep = Instant::now();
            }
        }
    }

    fn accept_connections(&mut self) {
        loop {
            match self.listener.accept() {
                Ok(stream) => {
                    let fd = stream.as_raw_fd();
                    stream.set_nonblocking(true).ok();

                    // Don't let TcpStream's Drop close the fd — we manage it ourselves
                    core::mem::forget(stream);

                    // Check global connection limit
                    if self.connections.len() >= self.security.rate_limits.max_connections {
                        unsafe { syscalls::close(fd); }
                        continue;
                    }

                    // Get client IP
                    let client_ip = peer_ip_from_fd(fd).unwrap_or(0);

                    // Check per-IP connection limit
                    let ip_count = self.ip_conn_counts.get(&client_ip).copied().unwrap_or(0);
                    if ip_count >= self.security.rate_limits.max_connections_per_ip {
                        unsafe { syscalls::close(fd); }
                        continue;
                    }

                    let max_read_buf = self.security.size_limits.max_header_size
                        + self.security.size_limits.max_body_size;

                    if self.tls_ctx.is_some() {
                        // TLS mode: create SSL object and start handshaking
                        let tls_ctx = self.tls_ctx.as_ref().unwrap();
                        match tls_ctx.new_ssl() {
                            Ok(ssl) => {
                                if ssl_set_fd(ssl, fd).is_ok() {
                                    if self.poller.register(fd, Interest::Read).is_ok() {
                                        self.connections.insert(
                                            fd,
                                            Connection::new_tls(fd, ssl, client_ip, max_read_buf),
                                        );
                                        self.increment_ip_count(client_ip);
                                    } else {
                                        crate::core::security::tls::stream::ssl_free(ssl);
                                        unsafe { syscalls::close(fd); }
                                    }
                                } else {
                                    crate::core::security::tls::stream::ssl_free(ssl);
                                    unsafe { syscalls::close(fd); }
                                }
                            }
                            Err(_) => {
                                unsafe { syscalls::close(fd); }
                            }
                        }
                    } else {
                        // Plaintext mode
                        if self.poller.register(fd, Interest::Read).is_ok() {
                            self.connections.insert(
                                fd,
                                Connection::new(fd, client_ip, max_read_buf),
                            );
                            self.increment_ip_count(client_ip);
                        } else {
                            unsafe { syscalls::close(fd); }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == IoErrorKind::WouldBlock {
                        break;
                    }
                    break;
                }
            }
        }
    }

    fn handle_readable(&mut self, fd: i32) {
        // Check if this connection is handshaking
        let is_handshaking = self.connections.get(&fd)
            .map(|c| c.state == ConnState::Handshaking)
            .unwrap_or(false);

        if is_handshaking {
            self.handle_handshake(fd);
            return;
        }

        let should_dispatch = if let Some(conn) = self.connections.get_mut(&fd) {
            if conn.state != ConnState::ReadingRequest {
                false
            } else {
                match conn.try_read() {
                    Ok(true) => true,
                    Ok(false) => {
                        if conn.state == ConnState::Done {
                            true // will be cleaned up
                        } else {
                            false
                        }
                    }
                    Err(_) => {
                        conn.state = ConnState::Done;
                        true
                    }
                }
            }
        } else {
            false
        };

        if should_dispatch {
            self.try_dispatch(fd);
        }
    }

    fn handle_handshake(&mut self, fd: i32) {
        let result = if let Some(conn) = self.connections.get_mut(&fd) {
            conn.try_handshake()
        } else {
            return;
        };

        match result {
            Ok(HandshakeResult::Complete) => {
                // Handshake done — transition to reading request
                if let Some(conn) = self.connections.get_mut(&fd) {
                    conn.state = ConnState::ReadingRequest;
                }
                let _ = self.poller.modify(fd, Interest::Read);
            }
            Ok(HandshakeResult::WantRead) => {
                // Already registered for read — nothing to do
            }
            Ok(HandshakeResult::WantWrite) => {
                let _ = self.poller.modify(fd, Interest::Write);
            }
            Err(_) => {
                self.close_connection(fd);
            }
        }
    }

    fn try_dispatch(&mut self, fd: i32) {
        // Extract what we need without holding a mutable borrow on self.connections
        let (is_done, client_ip) = match self.connections.get(&fd) {
            Some(c) => (c.state == ConnState::Done, c.client_ip),
            None => return,
        };

        if is_done {
            self.close_connection(fd);
            return;
        }

        // Parse using an immutable slice — drops the borrow immediately
        let parse_result = {
            let conn = self.connections.get(&fd).unwrap();
            parse_request(conn.read_buf.as_slice(), &self.security.size_limits)
        };

        match parse_result {
            ParseResult::Complete(mut request, _consumed) => {
                // Check global rate limit — copy values to avoid borrow conflict
                let global_rl = self.security.rate_limits.global.as_ref()
                    .map(|gl| (gl.requests, gl.window));
                if let Some((requests, window)) = global_rl {
                    let limit = RateLimit { requests, window };
                    if self.is_rate_limited(client_ip as u64, &limit) {
                        let resp = Response::new(StatusCode::TOO_MANY_REQUESTS)
                            .text("Too Many Requests");
                        let conn = self.connections.get_mut(&fd).unwrap();
                        conn.set_response(resp.serialize());
                        conn.keep_alive = false;
                        let _ = self.poller.modify(fd, Interest::Write);
                        return;
                    }
                    self.record_request(client_ip as u64);
                }

                let start_time = Instant::now();

                // Check static files first
                if let Some(ref dir) = self.public_dir {
                    if request.method == crate::libs::web::http::method::Method::Get {
                        if let Some(resp) = try_serve_static(dir.as_str(), request.route_path.as_str()) {
                            let elapsed = start_time.elapsed();
                            log_request(
                                request.method.as_str(),
                                request.route_path.as_str(),
                                resp.status.code(),
                                elapsed,
                            );
                            let bytes = resp.serialize();
                            let keep_alive = request.headers.connection_keep_alive();
                            let conn = self.connections.get_mut(&fd).unwrap();
                            conn.set_response(bytes);
                            conn.keep_alive = keep_alive;
                            let _ = self.poller.modify(fd, Interest::Write);
                            return;
                        }
                    }
                }

                // Route the request
                let route_match = self.router.resolve(request.route_path.as_str(), &request.method);

                // Check per-route rate limit
                if let Some((requests, window)) = route_match.rate_limit {
                    let route_key = Self::per_route_key(client_ip, request.route_path.as_str());
                    let limit = RateLimit { requests, window };
                    if self.is_rate_limited(route_key, &limit) {
                        let resp = Response::new(StatusCode::TOO_MANY_REQUESTS)
                            .text("Too Many Requests");
                        let conn = self.connections.get_mut(&fd).unwrap();
                        conn.set_response(resp.serialize());
                        conn.keep_alive = false;
                        let _ = self.poller.modify(fd, Interest::Write);
                        return;
                    }
                    self.record_request(route_key);
                }

                request.params = route_match.params;

                let conn = self.connections.get_mut(&fd).unwrap();
                conn.state = ConnState::Processing;

                // Submit to worker pool
                self.pool.submit(Job {
                    conn_fd: fd,
                    request,
                    handler: route_match.handler,
                    metadata_fn: route_match.metadata_fn,
                    start_time,
                    is_not_found: route_match.is_not_found,
                });
            }
            ParseResult::Incomplete => {
                // Wait for more data
            }
            ParseResult::Error(msg) => {
                let status = match msg {
                    "headers too large" | "body too large" => StatusCode::PAYLOAD_TOO_LARGE,
                    "URI too long" => StatusCode::URI_TOO_LONG,
                    _ => StatusCode::BAD_REQUEST,
                };
                let resp = Response::new(status).text(status.reason_phrase());
                let conn = self.connections.get_mut(&fd).unwrap();
                conn.set_response(resp.serialize());
                conn.keep_alive = false;
                let _ = self.poller.modify(fd, Interest::Write);
            }
        }
    }

    fn handle_writable(&mut self, fd: i32) {
        // Check if this connection is handshaking (WantWrite during handshake)
        let is_handshaking = self.connections.get(&fd)
            .map(|c| c.state == ConnState::Handshaking)
            .unwrap_or(false);

        if is_handshaking {
            self.handle_handshake(fd);
            return;
        }

        let (done, keep_alive) = if let Some(conn) = self.connections.get_mut(&fd) {
            if conn.state != ConnState::WritingResponse {
                return;
            }
            match conn.try_write() {
                Ok(true) => (true, conn.keep_alive),
                Ok(false) => (false, false),
                Err(_) => {
                    conn.state = ConnState::Done;
                    (true, false)
                }
            }
        } else {
            return;
        };

        if done {
            if keep_alive {
                if let Some(conn) = self.connections.get_mut(&fd) {
                    conn.reset_for_keep_alive();
                    let _ = self.poller.modify(fd, Interest::Read);
                }
            } else {
                self.close_connection(fd);
            }
        }
    }

    fn drain_results(&mut self) {
        let results = self.pool.drain_results();
        for result in results {
            let fd = result.conn_fd;
            if let Some(conn) = self.connections.get_mut(&fd) {
                conn.set_response(result.response_bytes);
                conn.keep_alive = result.keep_alive;
                let _ = self.poller.modify(fd, Interest::Write);
            }
        }
    }

    fn close_connection(&mut self, fd: i32) {
        if let Some(mut conn) = self.connections.remove(&fd) {
            self.decrement_ip_count(conn.client_ip);
            conn.shutdown_tls();
        }
        let _ = self.poller.deregister(fd);
        unsafe {
            syscalls::close(fd);
        }
    }

    // ── IP connection tracking ──────────────────────────────────────────

    fn increment_ip_count(&mut self, ip: u32) {
        let count = self.ip_conn_counts.get(&ip).copied().unwrap_or(0);
        self.ip_conn_counts.insert(ip, count + 1);
    }

    fn decrement_ip_count(&mut self, ip: u32) {
        if let Some(count) = self.ip_conn_counts.get(&ip).copied() {
            if count <= 1 {
                self.ip_conn_counts.remove(&ip);
            } else {
                self.ip_conn_counts.insert(ip, count - 1);
            }
        }
    }

    // ── Rate limiting ───────────────────────────────────────────────────

    fn is_rate_limited(&mut self, key: u64, limit: &RateLimit) -> bool {
        if let Some(deque) = self.rate_tracker.get_mut(&key) {
            // Drain expired entries from front
            while let Some(front) = deque.front() {
                if front.elapsed() > limit.window {
                    deque.pop_front();
                } else {
                    break;
                }
            }
            deque.len() >= limit.requests as usize
        } else {
            false
        }
    }

    fn record_request(&mut self, key: u64) {
        if let Some(deque) = self.rate_tracker.get_mut(&key) {
            deque.push_back(Instant::now());
        } else {
            let mut deque = VecDeque::new();
            deque.push_back(Instant::now());
            self.rate_tracker.insert(key, deque);
        }
    }

    fn per_route_key(client_ip: u32, path: &str) -> u64 {
        // FNV-1a hash of path
        let mut hash: u32 = 2166136261;
        for &b in path.as_bytes() {
            hash ^= b as u32;
            hash = hash.wrapping_mul(16777619);
        }
        (client_ip as u64) | ((hash as u64) << 32)
    }

    // ── Timeout sweep ───────────────────────────────────────────────────

    fn sweep_timeouts(&mut self) {
        let mut timed_out = Vec::new();

        for (&fd, conn) in self.connections.iter() {
            let elapsed = conn.last_activity.elapsed();
            let should_timeout = match conn.state {
                ConnState::Handshaking => elapsed > self.security.timeouts.handshake_timeout,
                ConnState::ReadingRequest => {
                    if conn.read_buf.is_empty() {
                        elapsed > self.security.timeouts.keep_alive_timeout
                    } else {
                        elapsed > self.security.timeouts.read_timeout
                    }
                }
                ConnState::WritingResponse => elapsed > self.security.timeouts.write_timeout,
                ConnState::Processing => false, // worker owns it
                ConnState::Done => true,
            };
            if should_timeout {
                timed_out.push(fd);
            }
        }

        for fd in timed_out.iter() {
            self.close_connection(*fd);
        }

        // Clean stale rate tracker entries (empty or >5min old)
        let stale_timeout = Duration::from_secs(300);
        let mut stale_keys = Vec::new();
        for (&key, deque) in self.rate_tracker.iter() {
            if deque.is_empty() {
                stale_keys.push(key);
            } else if let Some(back) = deque.back() {
                if back.elapsed() > stale_timeout {
                    stale_keys.push(key);
                }
            }
        }
        for key in stale_keys.iter() {
            self.rate_tracker.remove(key);
        }
    }
}
