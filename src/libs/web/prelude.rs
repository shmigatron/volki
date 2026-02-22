//! Convenience re-exports for SDK users.

pub use super::server::Server;
pub use super::http::method::Method;
pub use super::http::status::StatusCode;
pub use super::http::headers::Headers;
pub use super::http::request::Request;
pub use super::http::response::Response;
pub use super::html::document::HtmlDocument;
pub use super::router::file_route::FileRoute;
pub use super::html::element::{
    div, span, p, h1, h2, h3, h4, h5, h6,
    a, nav, header, footer, main_el, section, article,
    ul, ol, li, table, thead, tbody, tr, th, td,
    form, button, label, textarea, select, option,
    pre, code, strong, em, blockquote, script, style,
    br, hr, img, input, meta, link,
    text, raw_html,
    HtmlNode, HtmlElement, IntoChildren,
};
pub use crate::core::volkiwithstds::collections::Vec;
pub use super::html::metadata::{Metadata, MetadataFn, Robots};
pub use super::router::tree::PageHandler;
pub use super::html::render::{render_node, render_element};
pub use super::dom::{Document, NodeId, NodeType, Event, EventPhase};

/// Type aliases for return types.
pub type Html = HtmlDocument;
pub type Fragment = Vec<HtmlNode>;

/// Marker type for client-side WASM functions.
/// Functions returning `Client` are compiled to WASM with auto-generated JS glue.
/// The compiler strips these from server output and generates separate artifacts.
pub type Client = ();

/// Marker type for stateful component functions.
/// Functions returning `Component` are compiled to WASM with state management.
/// They persist state across renders and automatically re-run when state changes.
pub type Component = ();
