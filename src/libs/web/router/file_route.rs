//! File-based route types: `route.rs` (per-method) and `page.rs` (GET page).

use crate::libs::web::html::metadata::MetadataFn;
use crate::libs::web::http::method::Method;
use crate::libs::web::http::request::Request;
use crate::libs::web::http::response::Response;
use crate::libs::web::http::status::StatusCode;

pub type Handler = fn(&Request) -> Response;

/// A route file that exports typed HTTP method handlers.
///
/// Mirrors the Next.js `route.ts` convention — export `get`, `post`, `put`,
/// `patch`, `delete`, or `head` functions and they'll be dispatched by method.
pub struct FileRoute {
    pub get: Option<Handler>,
    pub post: Option<Handler>,
    pub put: Option<Handler>,
    pub patch: Option<Handler>,
    pub delete: Option<Handler>,
    pub head: Option<Handler>,
    pub metadata_fn: Option<MetadataFn>,
}

impl FileRoute {
    pub fn new() -> Self {
        Self {
            get: None,
            post: None,
            put: None,
            patch: None,
            delete: None,
            head: None,
            metadata_fn: None,
        }
    }

    pub fn get(mut self, handler: Handler) -> Self {
        self.get = Some(handler);
        self
    }

    pub fn post(mut self, handler: Handler) -> Self {
        self.post = Some(handler);
        self
    }

    pub fn put(mut self, handler: Handler) -> Self {
        self.put = Some(handler);
        self
    }

    pub fn patch(mut self, handler: Handler) -> Self {
        self.patch = Some(handler);
        self
    }

    pub fn delete(mut self, handler: Handler) -> Self {
        self.delete = Some(handler);
        self
    }

    pub fn head(mut self, handler: Handler) -> Self {
        self.head = Some(handler);
        self
    }

    pub fn metadata(mut self, meta_fn: MetadataFn) -> Self {
        self.metadata_fn = Some(meta_fn);
        self
    }

    /// Resolve a handler for the given HTTP method.
    /// Returns the handler if defined, or a 405 Method Not Allowed handler.
    pub fn resolve(&self, method: &Method) -> Handler {
        let handler = match method {
            Method::Get => self.get,
            Method::Post => self.post,
            Method::Put => self.put,
            Method::Patch => self.patch,
            Method::Delete => self.delete,
            Method::Head => self.head.or(self.get), // HEAD falls back to GET
            Method::Options => Some(method_not_allowed as Handler), // handled below
        };

        match handler {
            Some(h) => h,
            None => method_not_allowed,
        }
    }

    /// Returns true if at least one method is defined.
    pub fn has_any(&self) -> bool {
        self.get.is_some()
            || self.post.is_some()
            || self.put.is_some()
            || self.patch.is_some()
            || self.delete.is_some()
            || self.head.is_some()
    }
}

fn method_not_allowed(_req: &Request) -> Response {
    Response::new(StatusCode::METHOD_NOT_ALLOWED).text("405 Method Not Allowed")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_handler(_req: &Request) -> Response {
        Response::ok().text("ok")
    }

    fn created_handler(_req: &Request) -> Response {
        Response::new(StatusCode::CREATED).text("created")
    }

    #[test]
    fn test_get_only() {
        let route = FileRoute::new().get(ok_handler);
        // GET resolves
        let h = route.resolve(&Method::Get);
        assert_eq!(h as usize, ok_handler as Handler as usize);
        // POST returns 405
        let h = route.resolve(&Method::Post);
        assert_eq!(h as usize, method_not_allowed as Handler as usize);
    }

    #[test]
    fn test_multiple_methods() {
        let route = FileRoute::new().get(ok_handler).post(created_handler);
        assert_eq!(route.resolve(&Method::Get) as usize, ok_handler as Handler as usize);
        assert_eq!(route.resolve(&Method::Post) as usize, created_handler as Handler as usize);
    }

    #[test]
    fn test_head_falls_back_to_get() {
        let route = FileRoute::new().get(ok_handler);
        assert_eq!(route.resolve(&Method::Head) as usize, ok_handler as Handler as usize);
    }

    #[test]
    fn test_has_any() {
        assert!(!FileRoute::new().has_any());
        assert!(FileRoute::new().get(ok_handler).has_any());
    }
}
