//! Worker thread pool with job/result queues.

use crate::core::volkiwithstds::collections::{Vec, VecDeque};
use crate::core::volkiwithstds::sync::{Arc, Mutex};
use crate::core::volkiwithstds::thread;
use crate::core::volkiwithstds::time::{Duration, Instant};
use crate::libs::web::html::metadata::{MetadataFn, inject_metadata, is_html_content_type};
use crate::libs::web::http::request::Request;
use crate::libs::web::http::response::Response;
use crate::libs::web::router::tree::MatchedHandler;

pub struct Job {
    pub conn_fd: i32,
    pub request: Request,
    pub handler: MatchedHandler,
    pub metadata_fn: Option<MetadataFn>,
    pub start_time: Instant,
    pub is_not_found: bool,
}

pub struct JobResult {
    pub conn_fd: i32,
    pub response_bytes: Vec<u8>,
    pub keep_alive: bool,
}

// Safety: Job contains fn pointers (Send) and Request (Send-safe interior)
unsafe impl Send for Job {}
unsafe impl Send for JobResult {}

pub struct ThreadPool {
    job_queue: Arc<Mutex<VecDeque<Job>>>,
    result_queue: Arc<Mutex<VecDeque<JobResult>>>,
    _handles: Vec<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(num_workers: usize) -> Self {
        let job_queue = Arc::new(Mutex::new(VecDeque::new()));
        let result_queue = Arc::new(Mutex::new(VecDeque::new()));
        let mut handles = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let jobs = job_queue.clone();
            let results = result_queue.clone();
            let handle = thread::spawn(move || {
                worker_loop(jobs, results);
            });
            handles.push(handle);
        }

        Self {
            job_queue,
            result_queue,
            _handles: handles,
        }
    }

    pub fn submit(&self, job: Job) {
        self.job_queue.lock().push_back(job);
    }

    pub fn drain_results(&self) -> Vec<JobResult> {
        let mut queue = self.result_queue.lock();
        let mut results = Vec::new();
        while let Some(r) = queue.pop_front() {
            results.push(r);
        }
        results
    }
}

fn worker_loop(
    jobs: Arc<Mutex<VecDeque<Job>>>,
    results: Arc<Mutex<VecDeque<JobResult>>>,
) {
    loop {
        let job = {
            let mut queue = jobs.lock();
            queue.pop_front()
        };

        match job {
            Some(j) => {
                let method = j.request.method;
                let path = j.request.route_path.clone();

                let is_not_found = j.is_not_found;
                let mut response = match j.handler {
                    MatchedHandler::Handler(h) => h(&j.request),
                    MatchedHandler::Page(h) => Response::ok().document(&h(&j.request)),
                    MatchedHandler::DynamicPage(ref data) => {
                        let doc = crate::libs::web::interpreter::interpret_page(data, &j.request);
                        Response::ok().document(&doc)
                    }
                };
                if is_not_found {
                    response.status = crate::libs::web::http::status::StatusCode::NOT_FOUND;
                }
                let keep_alive = j.request.headers.connection_keep_alive();

                // Auto-inject metadata if a metadata_fn is registered
                if let Some(meta_fn) = j.metadata_fn {
                    let meta = meta_fn(&j.request);
                    // Validate — warnings are non-fatal, just discard for now
                    let _warnings = meta.validate();
                    // Only inject into HTML responses
                    let is_html = response
                        .headers
                        .get("content-type")
                        .map(|ct| is_html_content_type(ct))
                        .unwrap_or(false);
                    if is_html {
                        inject_metadata(&mut response.body, &meta);
                    }
                }

                let elapsed = j.start_time.elapsed();
                log_request(method.as_str(), &path, response.status.code(), elapsed);

                let response_bytes = response.serialize();

                results.lock().push_back(JobResult {
                    conn_fd: j.conn_fd,
                    response_bytes,
                    keep_alive,
                });
            }
            None => {
                // Idle — sleep briefly to avoid busy-spinning
                thread::sleep(Duration::from_millis(1));
            }
        }
    }
}

/// Log a request/response line to stderr.
pub fn log_request(method: &str, path: &str, status: u16, elapsed: Duration) {
    use crate::core::cli::style;

    let ms = elapsed.as_millis();
    let time_str = style::format_duration(ms);

    let status_str = crate::vformat!("{status}");
    let colored_status = if status < 300 {
        style::green(&status_str)
    } else if status < 400 {
        style::cyan(&status_str)
    } else if status < 500 {
        style::yellow(&status_str)
    } else {
        style::red(&status_str)
    };

    let dim_time = style::dim(&crate::vformat!("({time_str})"));

    crate::veprintln!("  {method:<7} {path:<30} {colored_status} {dim_time}");
}
