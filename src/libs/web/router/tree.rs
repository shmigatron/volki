//! Trie-based route tree.

use super::file_route::FileRoute;
use super::matcher::{RouteSegment, parse_route_path};
use crate::core::volkiwithstds::collections::{Box, HashMap, String, Vec};
use crate::core::volkiwithstds::sync::Arc;
use crate::core::volkiwithstds::time::Duration;
use crate::libs::web::html::document::HtmlDocument;
use crate::libs::web::html::metadata::MetadataFn;
use crate::libs::web::http::method::Method;
use crate::libs::web::http::request::Request;
use crate::libs::web::http::response::Response;
use crate::libs::web::interpreter::DynamicPageData;

pub type Handler = fn(&Request) -> Response;
pub type PageHandler = fn(&Request) -> HtmlDocument;

/// The resolved handler for a matched route.
pub enum MatchedHandler {
    Handler(Handler),
    Page(PageHandler),
    DynamicPage(Arc<DynamicPageData>),
}

/// A route endpoint can be a single handler, a page, or a per-method file route.
pub enum RouteHandler {
    Single(Handler),
    Page(PageHandler),
    FileRoute(FileRoute),
    DynamicPage(Arc<DynamicPageData>),
}

impl RouteHandler {
    fn resolve(&self, method: &Method) -> MatchedHandler {
        match self {
            RouteHandler::Single(h) => MatchedHandler::Handler(*h),
            RouteHandler::Page(h) => MatchedHandler::Page(*h),
            RouteHandler::FileRoute(fr) => MatchedHandler::Handler(fr.resolve(method)),
            RouteHandler::DynamicPage(d) => MatchedHandler::DynamicPage(d.clone()),
        }
    }
}

pub struct RouteMatch {
    pub handler: MatchedHandler,
    pub params: HashMap<String, String>,
    pub is_api: bool,
    pub metadata_fn: Option<MetadataFn>,
    pub is_not_found: bool,
    pub rate_limit: Option<(u32, Duration)>,
}

pub struct RouteNode {
    handler: Option<RouteHandler>,
    metadata_fn: Option<MetadataFn>,
    is_api: bool,
    rate_limit: Option<(u32, Duration)>,
    static_children: HashMap<String, RouteNode>,
    dynamic_child: Option<(String, Box<RouteNode>)>,
    catch_all: Option<(String, RouteHandler, bool, Option<MetadataFn>)>,
}

impl RouteNode {
    pub fn new() -> Self {
        Self {
            handler: None,
            metadata_fn: None,
            is_api: false,
            rate_limit: None,
            static_children: HashMap::new(),
            dynamic_child: None,
            catch_all: None,
        }
    }

    pub fn insert(&mut self, pattern: &str, handler: Handler, is_api: bool) {
        let segments = parse_route_path(pattern);
        self.insert_segments(&segments, 0, RouteHandler::Single(handler), is_api, None, None);
    }

    pub fn insert_with_rate_limit(
        &mut self,
        pattern: &str,
        handler: Handler,
        is_api: bool,
        requests: u32,
        window: Duration,
    ) {
        let segments = parse_route_path(pattern);
        self.insert_segments(
            &segments,
            0,
            RouteHandler::Single(handler),
            is_api,
            None,
            Some((requests, window)),
        );
    }

    pub fn insert_page(&mut self, pattern: &str, handler: PageHandler) {
        let segments = parse_route_path(pattern);
        self.insert_segments(&segments, 0, RouteHandler::Page(handler), false, None, None);
    }

    pub fn insert_page_with_metadata(
        &mut self,
        pattern: &str,
        handler: PageHandler,
        metadata_fn: MetadataFn,
    ) {
        let segments = parse_route_path(pattern);
        self.insert_segments(
            &segments,
            0,
            RouteHandler::Page(handler),
            false,
            Some(metadata_fn),
            None,
        );
    }

    pub fn insert_with_metadata(
        &mut self,
        pattern: &str,
        handler: Handler,
        is_api: bool,
        metadata_fn: MetadataFn,
    ) {
        let segments = parse_route_path(pattern);
        self.insert_segments(
            &segments,
            0,
            RouteHandler::Single(handler),
            is_api,
            Some(metadata_fn),
            None,
        );
    }

    pub fn insert_dynamic_page(&mut self, pattern: &str, data: Arc<DynamicPageData>) {
        let segments = parse_route_path(pattern);
        self.insert_segments(&segments, 0, RouteHandler::DynamicPage(data), false, None, None);
    }

    pub fn insert_file_route(&mut self, pattern: &str, file_route: FileRoute, is_api: bool) {
        let meta_fn = file_route.metadata_fn;
        let segments = parse_route_path(pattern);
        self.insert_segments(&segments, 0, RouteHandler::FileRoute(file_route), is_api, meta_fn, None);
    }

    fn insert_segments(
        &mut self,
        segments: &[RouteSegment],
        idx: usize,
        route_handler: RouteHandler,
        is_api: bool,
        meta_fn: Option<MetadataFn>,
        rl: Option<(u32, Duration)>,
    ) {
        if idx >= segments.len() {
            self.handler = Some(route_handler);
            self.metadata_fn = meta_fn;
            self.is_api = is_api;
            self.rate_limit = rl;
            return;
        }

        match &segments[idx] {
            RouteSegment::Static(name) => {
                if !self.static_children.contains_key(name.as_str()) {
                    self.static_children
                        .insert(name.clone(), RouteNode::new());
                }
                let child = self.static_children.get_mut(name.as_str()).unwrap();
                child.insert_segments(segments, idx + 1, route_handler, is_api, meta_fn, rl);
            }
            RouteSegment::Dynamic(param_name) => {
                if self.dynamic_child.is_none() {
                    self.dynamic_child =
                        Some((param_name.clone(), Box::new(RouteNode::new())));
                }
                let (_, child): &mut (String, Box<RouteNode>) = self.dynamic_child.as_mut().unwrap();
                child.insert_segments(segments, idx + 1, route_handler, is_api, meta_fn, rl);
            }
            RouteSegment::CatchAll(param_name) => {
                self.catch_all = Some((param_name.clone(), route_handler, is_api, meta_fn));
            }
        }
    }

    pub fn match_path(&self, path: &str, method: &Method) -> Option<RouteMatch> {
        let trimmed = path.trim_matches('/');
        let segments: Vec<&str> = if trimmed.is_empty() {
            Vec::new()
        } else {
            trimmed.split('/').collect()
        };
        let mut params = HashMap::new();
        self.match_segments(&segments, 0, &mut params, method)
    }

    fn match_segments(
        &self,
        segments: &[&str],
        idx: usize,
        params: &mut HashMap<String, String>,
        method: &Method,
    ) -> Option<RouteMatch> {
        if idx >= segments.len() {
            if let Some(ref rh) = self.handler {
                return Some(RouteMatch {
                    handler: rh.resolve(method),
                    params: params.clone(),
                    is_api: self.is_api,
                    metadata_fn: self.metadata_fn,
                    is_not_found: false,
                    rate_limit: self.rate_limit,
                });
            }
            return None;
        }

        let segment = segments[idx];

        // Try static match first
        if let Some(child) = self.static_children.get(segment) {
            if let Some(m) = child.match_segments(segments, idx + 1, params, method) {
                return Some(m);
            }
        }

        // Try dynamic match
        if let Some((ref param_name, ref child)) = self.dynamic_child {
            params.insert(param_name.clone(), String::from(segment));
            if let Some(m) = child.match_segments(segments, idx + 1, params, method) {
                return Some(m);
            }
            params.remove(param_name.as_str());
        }

        // Try catch-all
        if let Some((ref param_name, ref rh, ref is_api, ref meta_fn)) = self.catch_all {
            let remaining: Vec<&str> = segments[idx..].iter().copied().collect();
            let joined = remaining.join("/");
            params.insert(param_name.clone(), joined);
            return Some(RouteMatch {
                handler: rh.resolve(method),
                params: params.clone(),
                is_api: *is_api,
                metadata_fn: *meta_fn,
                is_not_found: false,
                rate_limit: None,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_handler(_req: &Request) -> Response {
        Response::ok().text("ok")
    }

    fn post_handler(_req: &Request) -> Response {
        Response::ok().text("posted")
    }

    #[test]
    fn test_static_route() {
        let mut root = RouteNode::new();
        root.insert("/hello", dummy_handler, false);
        let m = root.match_path("/hello", &Method::Get).unwrap();
        assert!(!m.is_api);
    }

    #[test]
    fn test_dynamic_route() {
        let mut root = RouteNode::new();
        root.insert("/users/[id]", dummy_handler, false);
        let m = root.match_path("/users/42", &Method::Get).unwrap();
        assert_eq!(m.params.get("id").unwrap().as_str(), "42");
    }

    #[test]
    fn test_catch_all() {
        let mut root = RouteNode::new();
        root.insert("/docs/[...slug]", dummy_handler, false);
        let m = root.match_path("/docs/a/b/c", &Method::Get).unwrap();
        assert_eq!(m.params.get("slug").unwrap().as_str(), "a/b/c");
    }

    #[test]
    fn test_no_match() {
        let root = RouteNode::new();
        assert!(root.match_path("/anything", &Method::Get).is_none());
    }

    #[test]
    fn test_root_route() {
        let mut root = RouteNode::new();
        root.insert("/", dummy_handler, false);
        assert!(root.match_path("/", &Method::Get).is_some());
    }

    #[test]
    fn test_api_flag() {
        let mut root = RouteNode::new();
        root.insert("/api/users", dummy_handler, true);
        let m = root.match_path("/api/users", &Method::Get).unwrap();
        assert!(m.is_api);
    }

    fn as_handler(m: &MatchedHandler) -> Handler {
        match m {
            MatchedHandler::Handler(h) => *h,
            MatchedHandler::Page(_) => panic!("expected Handler, got Page"),
            MatchedHandler::DynamicPage(_) => panic!("expected Handler, got DynamicPage"),
        }
    }

    #[test]
    fn test_file_route_dispatches_by_method() {
        let mut root = RouteNode::new();
        let fr = FileRoute::new().get(dummy_handler).post(post_handler);
        root.insert_file_route("/api/items", fr, true);

        let m = root.match_path("/api/items", &Method::Get).unwrap();
        assert_eq!(as_handler(&m.handler) as usize, dummy_handler as Handler as usize);

        let m = root.match_path("/api/items", &Method::Post).unwrap();
        assert_eq!(as_handler(&m.handler) as usize, post_handler as Handler as usize);
    }

    #[test]
    fn test_file_route_405_for_undefined_method() {
        let mut root = RouteNode::new();
        let fr = FileRoute::new().get(dummy_handler);
        root.insert_file_route("/api/items", fr, true);

        let m = root.match_path("/api/items", &Method::Delete).unwrap();
        // Should resolve to the 405 handler, not dummy_handler
        assert_ne!(as_handler(&m.handler) as usize, dummy_handler as Handler as usize);
    }

    fn dummy_page(_req: &Request) -> HtmlDocument {
        HtmlDocument::new().title("test")
    }

    #[test]
    fn test_page_route() {
        let mut root = RouteNode::new();
        root.insert_page("/about", dummy_page);
        let m = root.match_path("/about", &Method::Get).unwrap();
        assert!(!m.is_api);
        assert!(matches!(m.handler, MatchedHandler::Page(_)));
    }
}
