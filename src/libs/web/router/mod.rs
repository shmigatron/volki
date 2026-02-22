pub mod file_route;
pub mod loader;
pub mod matcher;
pub mod tree;

use file_route::FileRoute;
use tree::{RouteNode, RouteMatch, Handler, PageHandler, MatchedHandler};
use crate::core::volkiwithstds::sync::Arc;
use crate::core::volkiwithstds::time::Duration;
use crate::libs::web::html::metadata::MetadataFn;
use crate::libs::web::http::method::Method;
use crate::libs::web::http::request::Request;
use crate::libs::web::http::response::Response;
use crate::libs::web::http::status::StatusCode;
use crate::libs::web::interpreter::DynamicPageData;

pub struct Router {
    root: RouteNode,
    not_found_handler: Option<Handler>,
    not_found_page: Option<PageHandler>,
    not_found_dynamic: Option<Arc<DynamicPageData>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            root: RouteNode::new(),
            not_found_handler: None,
            not_found_page: None,
            not_found_dynamic: None,
        }
    }

    pub fn page_route(&mut self, pattern: &str, handler: PageHandler) {
        self.root.insert_page(pattern, handler);
    }

    pub fn page_route_with_metadata(
        &mut self,
        pattern: &str,
        handler: PageHandler,
        metadata_fn: MetadataFn,
    ) {
        self.root.insert_page_with_metadata(pattern, handler, metadata_fn);
    }

    pub fn api_route(&mut self, pattern: &str, handler: Handler) {
        self.root.insert(pattern, handler, true);
    }

    pub fn api_route_with_rate_limit(
        &mut self,
        pattern: &str,
        handler: Handler,
        requests: u32,
        window: Duration,
    ) {
        self.root.insert_with_rate_limit(pattern, handler, true, requests, window);
    }

    pub fn file_route(&mut self, pattern: &str, file_route: FileRoute, is_api: bool) {
        self.root.insert_file_route(pattern, file_route, is_api);
    }

    pub fn not_found(&mut self, handler: Handler) {
        self.not_found_handler = Some(handler);
    }

    pub fn not_found_page(&mut self, handler: PageHandler) {
        self.not_found_page = Some(handler);
    }

    pub fn dynamic_page_route(&mut self, pattern: &str, data: Arc<DynamicPageData>) {
        self.root.insert_dynamic_page(pattern, data);
    }

    pub fn not_found_dynamic_page(&mut self, data: Arc<DynamicPageData>) {
        self.not_found_dynamic = Some(data);
    }

    pub fn resolve(&self, path: &str, method: &Method) -> RouteMatch {
        if let Some(m) = self.root.match_path(path, method) {
            return m;
        }

        // Not found fallbacks — dynamic pages, then static pages, then handlers
        if let Some(ref data) = self.not_found_dynamic {
            return RouteMatch {
                handler: MatchedHandler::DynamicPage(data.clone()),
                params: crate::core::volkiwithstds::collections::HashMap::new(),
                is_api: false,
                metadata_fn: None,
                is_not_found: true,
                rate_limit: None,
            };
        }

        if let Some(page_handler) = self.not_found_page {
            return RouteMatch {
                handler: MatchedHandler::Page(page_handler),
                params: crate::core::volkiwithstds::collections::HashMap::new(),
                is_api: false,
                metadata_fn: None,
                is_not_found: true,
                rate_limit: None,
            };
        }

        if let Some(handler) = self.not_found_handler {
            return RouteMatch {
                handler: MatchedHandler::Handler(handler),
                params: crate::core::volkiwithstds::collections::HashMap::new(),
                is_api: false,
                metadata_fn: None,
                is_not_found: true,
                rate_limit: None,
            };
        }

        RouteMatch {
            handler: MatchedHandler::Handler(default_not_found),
            params: crate::core::volkiwithstds::collections::HashMap::new(),
            is_api: false,
            metadata_fn: None,
            is_not_found: true,
            rate_limit: None,
        }
    }
}

fn default_not_found(_req: &Request) -> Response {
    Response::new(StatusCode::NOT_FOUND).text("404 Not Found")
}
