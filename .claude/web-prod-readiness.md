# Web Production Readiness

Everything that needs to happen before `src/libs/web/` can serve a real website.

---

## P0 — Build Blockers ✅

- [x] Fix `wasm_codegen.rs` compile errors (string literal parse errors, missing `pop()`)
- [x] Verify `compiler/mod.rs` arg counts for `generate_wasm_module` / `generate_js_glue`
- [x] Fix interpreter unresolved import
- [x] Fix router/reactor non-exhaustive match on `DynamicPage`
- [x] Run `cargo check` clean with zero errors

## P0 — Security (Critical)

- [x] HTTPS/TLS support (OpenSSL FFI — `Server::tls()`, `--tls-cert`/`--tls-key`, TLSv1.2+ only)
- [x] Request size limits (max body size, max header size, max URI length)
- [x] Connection timeouts (read, write, keep-alive, idle)
- [x] Rate limiting (per-IP, per-route)
- [ ] Static file path traversal hardening (URL-encoded `%2e%2e`, null bytes, symlinks)
- [ ] CORS handling (`Access-Control-*` headers, preflight `OPTIONS`)
- [ ] CSRF protection (token generation + validation for state-changing methods)
- [ ] Security headers (`Content-Security-Policy`, `X-Frame-Options`, `X-Content-Type-Options`, `Strict-Transport-Security`, `X-XSS-Protection`)
- [ ] Request ID generation + propagation (for tracing / abuse tracking)

## P1 — Core HTTP

- [ ] Response compression (gzip, brotli, deflate with `Accept-Encoding` negotiation)
- [ ] Chunked transfer encoding (streaming responses without full materialization)
- [ ] Cookie parsing (`Cookie` header → map) and `Set-Cookie` builder (`HttpOnly`, `Secure`, `SameSite`, `Max-Age`, `Path`, `Domain`)
- [ ] Form body parsing (`application/x-www-form-urlencoded`)
- [ ] Multipart body parsing (`multipart/form-data`, file uploads)
- [ ] JSON body parsing (`request.json::<T>()` or equivalent)
- [ ] Query parameter deserialization (typed extraction from query string)
- [ ] Content negotiation (`Accept` header parsing, response format switching)
- [ ] `HEAD` request auto-handling (return headers without body for any `GET` route)
- [ ] `304 Not Modified` support (ETags, `Last-Modified`, `If-None-Match`, `If-Modified-Since`)

## P1 — Server Infrastructure

- [ ] Middleware / handler chain (composable layers for logging, auth, CORS, compression, etc.)
- [ ] Graceful shutdown (SIGTERM/SIGINT handling, drain in-flight requests, close listeners)
- [ ] Panic recovery in handlers (catch panics, return 500, respawn worker thread)
- [ ] Thread pool condvar notification (replace 1ms busy-wait spin loop)
- [ ] Connection limits (max concurrent connections, backpressure)
- [ ] Structured logging (log levels, JSON output, request IDs, timestamps, log file rotation)
- [ ] Health check endpoint (`/health`, `/ready` for load balancers and orchestrators)
- [ ] Error pages (customizable 404, 500 pages with dev-mode stack traces)

## P1 — Routing

- [ ] Route groups / prefixes (`/api/v1/*` grouping with shared middleware)
- [ ] Wildcard routes (`/files/*path` catch-all)
- [ ] Route-level middleware (apply auth only to certain routes)
- [ ] Redirect helpers (`Response::redirect`, `Response::permanent_redirect`)
- [ ] Trailing slash normalization (redirect `/about/` → `/about` or vice versa)

## P2 — NestJS-Inspired Features

### Guards

- [ ] Guard trait (`fn can_activate(&self, request: &Request) -> bool`)
- [ ] Auth guard (JWT validation, API key validation, session check)
- [ ] Role-based guard (`@Roles("admin", "editor")` equivalent)
- [ ] Guard composition (combine multiple guards with AND/OR logic)
- [ ] Route-level and group-level guard attachment

### Interceptors

- [ ] Interceptor trait (pre-handler + post-handler hooks, can transform request/response)
- [ ] Logging interceptor (request/response timing, payload logging)
- [ ] Transform interceptor (wrap all responses in `{ data: ..., meta: ... }` envelope)
- [ ] Cache interceptor (response caching with TTL per route)
- [ ] Timeout interceptor (cancel handler if it exceeds a deadline)

### Pipes (Validation + Transformation)

- [ ] Pipe trait (validate and/or transform input before handler runs)
- [ ] Validation pipe (struct-level validation rules, reject 400 on bad input)
- [ ] Parse pipe (parse path params: `ParseIntPipe`, `ParseUuidPipe`)
- [ ] Default value pipe (fill missing optional params)
- [ ] Trim / sanitize pipe (strip whitespace, escape HTML in user input)

### Exception Filters

- [ ] Exception filter trait (catch specific error types, return custom responses)
- [ ] Global exception filter (catch-all for unhandled errors → 500 JSON)
- [ ] Per-route exception filters (different error formatting for API vs HTML routes)
- [ ] Error serialization (structured error responses with error code, message, details)

### Cron / Scheduled Tasks

- [ ] Cron scheduler (register functions with cron expressions)
- [ ] `@Cron("0 */5 * * *")` equivalent decorator/attribute
- [ ] Named cron jobs (start, stop, get status by name)
- [ ] Cron job overlap protection (skip if previous run still active)
- [ ] Timezone support for cron expressions
- [ ] One-shot delayed tasks (`run_after(duration, || { ... })`)
- [ ] Interval tasks (`run_every(duration, || { ... })`)
- [ ] Startup / shutdown hooks (run code on server start / graceful stop)

### Queues / Background Jobs

- [ ] Job queue abstraction (in-memory to start, pluggable backends later)
- [ ] Job producer (`queue.add("email", job_data)`)
- [ ] Job consumer / processor (dedicated worker threads)
- [ ] Job retry with backoff (configurable max retries, exponential backoff)
- [ ] Job priority levels (urgent, normal, low)
- [ ] Dead letter queue (failed jobs after max retries)
- [ ] Job progress tracking (percentage, status updates)
- [ ] Concurrency control (max concurrent jobs per queue)
- [ ] Delayed jobs (process after a specified delay)
- [ ] Job events (completed, failed, stalled, progress)

### Events / Event Emitter

- [ ] Event emitter (pub/sub within the server process)
- [ ] Typed events (event name → payload type mapping)
- [ ] Async event listeners (handlers run in thread pool)
- [ ] Event listener ordering (priority)
- [ ] One-time listeners (auto-remove after first fire)

### Dependency Injection / Services

- [ ] Service registry (singleton services accessible from handlers)
- [ ] Service lifecycle (init on startup, cleanup on shutdown)
- [ ] Scoped services (per-request instances)
- [ ] Service dependencies (service A depends on service B, init order)

## P2 — Sessions + Auth

- [ ] Session middleware (cookie-based session ID → server-side store)
- [ ] In-memory session store (dev default)
- [ ] Session expiry and cleanup
- [ ] `request.session()` accessor
- [ ] Flash messages (one-time session messages across redirects)
- [ ] Auth module (login, logout, session creation/destruction)
- [ ] Password hashing utilities
- [ ] JWT generation + validation
- [ ] OAuth2 client (authorization code flow)
- [ ] API key authentication

## P2 — Dev Experience

- [ ] Hot reload / file watching (`web:dev` reloads on `.volki` file changes)
- [ ] In-browser error overlay (parse errors, runtime errors rendered in page)
- [ ] Dev-mode source maps (map compiled output back to `.volki` source)
- [ ] Dev-mode request inspector (show request/response details in terminal)
- [ ] `--open` flag (auto-open browser on `web:dev` start)
- [ ] Port conflict detection (try next port if default is in use)

## P2 — Client-Side / WASM

- [ ] Finish `use_effect` hook (Phase2 — effect re-runs on dependency changes)
- [ ] Finish `use_memo` hooks (Phase3 — `use_memo_i32`, `use_memo_f32`)
- [ ] Auto-install `wasm32-unknown-unknown` target (or clear error + install command)
- [ ] WASM size optimization (`wasm-opt`, `wasm-strip`, LTO)
- [ ] Client-side routing (SPA navigation without full page reloads)
- [ ] Hydration (server-rendered HTML + client-side WASM takeover)

## P2 — Static Files + Assets

- [ ] ETag generation (content hash for conditional requests)
- [ ] `Last-Modified` header from file mtime
- [ ] `304 Not Modified` responses for conditional requests
- [ ] Cache-busting via content hash in filenames (`styles.abc123.css`)
- [ ] Asset pipeline (CSS/JS bundling, minification for production)
- [ ] Configurable `Cache-Control` per file type (immutable for hashed assets, short TTL for HTML)
- [ ] Directory listing (optional, disabled by default)
- [ ] Range requests (`Accept-Ranges`, `206 Partial Content` for large files / video)

## P3 — WebSockets

- [ ] HTTP `Upgrade` handling (101 Switching Protocols)
- [ ] WebSocket frame parser (text, binary, ping/pong, close)
- [ ] WebSocket connection management (open, message, close, error callbacks)
- [ ] Broadcast / rooms (send to all connected clients, or to a named room)
- [ ] WebSocket authentication (validate on upgrade)
- [ ] Heartbeat / ping interval (detect dead connections)

## P3 — HTTP/2

- [ ] HTTP/2 frame parser
- [ ] Stream multiplexing
- [ ] Header compression (HPACK)
- [ ] Server push
- [ ] Flow control

## P3 — Database Integration

- [ ] Database connection pooling (reuse connections across requests)
- [ ] Query builder or lightweight ORM
- [ ] Migration system (up/down SQL migrations, version tracking)
- [ ] Transaction support in request handlers

## P3 — Caching

- [ ] In-memory cache (LRU with TTL)
- [ ] Cache key generation helpers
- [ ] Route-level cache decorator (cache full responses by URL)
- [ ] Cache invalidation (manual, TTL-based, event-based)
- [ ] Distributed cache interface (for multi-instance deployments)

## P3 — Platform + Deployment

- [ ] Windows support (`Poller` impl for IOCP or fallback `select`)
- [ ] Daemon mode (`-d` flag, PID file, background process)
- [ ] Systemd integration (socket activation, notify ready)
- [ ] Environment variable overrides (host, port, workers, log level)
- [ ] Production config file (separate from `volki.toml` dev config)
- [ ] Docker-friendly defaults (bind `0.0.0.0`, respect `PORT` env var, log to stdout)
- [ ] Cluster mode (multi-process with shared socket for multi-core utilization)

## P3 — Observability

- [ ] Request metrics (requests/sec, latency percentiles, error rate)
- [ ] Prometheus-compatible metrics endpoint (`/metrics`)
- [ ] Distributed tracing (trace ID propagation, span creation)
- [ ] Runtime introspection endpoint (thread pool status, connection count, uptime)

## P3 — Testing

- [ ] Integration tests for HTTP server (end-to-end request → response)
- [ ] Test client (`TestServer` that boots the app and sends requests in-process)
- [ ] Load / stress test suite (benchmark throughput, latency, connection limits)
- [ ] Test utilities (request builders, response assertions, mock services)
