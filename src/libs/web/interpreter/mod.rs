//! Runtime interpreter — converts parsed RsxNode AST into HtmlDocument at runtime.
//!
//! Used by `web:dev` to serve `.volki` pages without a `cargo build` step.
//! The compiler's parser produces `RsxNode` trees; this module evaluates them
//! into `RuntimeHtmlNode` trees that render to HTML strings.

pub mod scanner;

use crate::core::volkiwithstds::collections::{HashMap, String, Vec};
use crate::libs::web::compiler::parser::{RsxNode, RsxAttr, RsxAttrValue};
use crate::libs::web::html::document::HtmlDocument;
use crate::libs::web::html::element::HtmlNode;
use crate::libs::web::html::runtime::{RuntimeHtmlNode, RuntimeHtmlElement, render_runtime_node};
use crate::libs::web::http::request::Request;

/// Data for a dynamically-interpreted page, built at startup by the dev scanner.
pub struct DynamicPageData {
    /// Parsed AST for the `-> Html` function body.
    pub nodes: Vec<RsxNode>,
    /// Generated CSS from volkistyle.
    pub css: String,
    /// Fragment functions: name → parsed AST nodes.
    pub fragments: HashMap<String, Vec<RsxNode>>,
    /// Extracted metadata from the `metadata()` function.
    pub metadata: Option<ParsedMetadata>,
    /// Optional generated client glue script URL.
    pub client_glue_url: Option<String>,
}

/// Metadata extracted from a `.volki` file's `metadata()` function body.
pub struct ParsedMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_type: Option<String>,
}

// DynamicPageData contains only Vec, String, HashMap — all Send+Sync in volkiwithstds.
unsafe impl Send for DynamicPageData {}
unsafe impl Sync for DynamicPageData {}

/// Interpret a dynamic page's AST into an HtmlDocument, ready to serve.
pub fn interpret_page(data: &DynamicPageData, _req: &Request) -> HtmlDocument {
    let mut doc = HtmlDocument::new();

    // Apply metadata if present
    if let Some(ref meta) = data.metadata {
        if let Some(ref title) = meta.title {
            doc = doc.title(title.as_str());
        }
        if let Some(ref desc) = meta.description {
            doc = doc.head_node(
                crate::libs::web::html::element::meta()
                    .attr("name", "description")
                    .attr("content", desc.as_str())
                    .into_node(),
            );
        }
        if let Some(ref og_title) = meta.og_title {
            doc = doc.head_node(
                crate::libs::web::html::element::meta()
                    .attr("property", "og:title")
                    .attr("content", og_title.as_str())
                    .into_node(),
            );
        }
        if let Some(ref og_desc) = meta.og_description {
            doc = doc.head_node(
                crate::libs::web::html::element::meta()
                    .attr("property", "og:description")
                    .attr("content", og_desc.as_str())
                    .into_node(),
            );
        }
        if let Some(ref og_type) = meta.og_type {
            doc = doc.head_node(
                crate::libs::web::html::element::meta()
                    .attr("property", "og:type")
                    .attr("content", og_type.as_str())
                    .into_node(),
            );
        }
    }

    // Inject CSS as inline style
    if !data.css.is_empty() {
        doc = doc.inline_style(data.css.as_str());
    }

    if let Some(ref glue_url) = data.client_glue_url {
        doc = doc.script_module(glue_url.as_str());
    }

    // Interpret each top-level AST node
    for node in data.nodes.iter() {
        match node {
            // Special <Style>{expr}</Style> — already handled via data.css, skip
            RsxNode::Element { tag, children, self_closing: false, .. }
                if tag.as_str() == "Style" =>
            {
                // If the expr isn't "CSS" (i.e. a custom const), we already collected
                // it during scanning. Just skip the Style element.
                let _ = children;
            }
            // Special <Stylesheet href="..." /> — link an external stylesheet
            RsxNode::Element { tag, attrs, self_closing: true, .. }
                if tag.as_str() == "Stylesheet" =>
            {
                for attr in attrs.iter() {
                    if attr.name.as_str() == "href" {
                        if let RsxAttrValue::Literal(v) = &attr.value {
                            doc = doc.stylesheet(v.as_str());
                        }
                    }
                }
            }
            _ => {
                let html_nodes = interpret_node(node, &data.fragments);
                for html_node in html_nodes {
                    doc = doc.body_node(html_node);
                }
            }
        }
    }

    doc
}

/// Interpret a single RsxNode into HtmlNode(s).
fn interpret_node(node: &RsxNode, fragments: &HashMap<String, Vec<RsxNode>>) -> Vec<HtmlNode> {
    match node {
        RsxNode::Text(s) => {
            let mut v = Vec::new();
            v.push(HtmlNode::Text(s.clone()));
            v
        }
        RsxNode::Expr(expr) => {
            interpret_expr(expr.as_str(), fragments)
        }
        RsxNode::Element { tag, attrs, children, self_closing } => {
            // Resolve component tags (PascalCase) to fragment function calls
            if is_component_like(tag.as_str()) {
                if let Some(frag_nodes) = fragments.get(tag.as_str()) {
                    let mut result = Vec::new();
                    for node in frag_nodes.iter() {
                        let nodes = interpret_node(node, fragments);
                        for n in nodes {
                            result.push(n);
                        }
                    }
                    return result;
                }
            }
            let runtime_el = interpret_element(tag, attrs, children, *self_closing, fragments);
            let rendered = render_runtime_node(&RuntimeHtmlNode::Element(runtime_el));
            let mut v = Vec::new();
            v.push(HtmlNode::Raw(rendered));
            v
        }
        RsxNode::CondAnd { body, .. } => {
            // In dev-mode interpretation, always render the body
            let mut v = Vec::new();
            for node in body.iter() {
                let nodes = interpret_node(node, fragments);
                for n in nodes { v.push(n); }
            }
            v
        }
        RsxNode::Ternary { if_true, .. } => {
            // In dev-mode interpretation, render the true branch
            let mut v = Vec::new();
            for node in if_true.iter() {
                let nodes = interpret_node(node, fragments);
                for n in nodes { v.push(n); }
            }
            v
        }
    }
}

/// Interpret an expression node.
///
/// - If it looks like a fragment call `name()`, look it up in fragments.
/// - Otherwise, render a dev-mode placeholder.
fn interpret_expr(expr: &str, fragments: &HashMap<String, Vec<RsxNode>>) -> Vec<HtmlNode> {
    let trimmed = expr.trim();

    // Check for fragment call: "name()" pattern
    if let Some(name) = extract_fn_call_name(trimmed) {
        if let Some(frag_nodes) = fragments.get(name) {
            let mut result = Vec::new();
            for node in frag_nodes.iter() {
                let nodes = interpret_node(node, fragments);
                for n in nodes {
                    result.push(n);
                }
            }
            return result;
        }
    }

    // Unknown expression — render placeholder
    let mut v = Vec::new();
    let placeholder = crate::vformat!(
        "<div class=\"__volki-dev-placeholder\" style=\"padding:4px 8px;background:#fef3c7;border:1px dashed #d97706;border-radius:4px;font-family:monospace;font-size:12px;color:#92400e;\">[{}]</div>",
        trimmed
    );
    v.push(HtmlNode::Raw(placeholder));
    v
}

/// Check if a tag is a custom component (PascalCase, not a builtin like Style/Head/Stylesheet).
fn is_component_like(tag: &str) -> bool {
    let first = tag.as_bytes().first().copied().unwrap_or(b'\0');
    first.is_ascii_uppercase() && tag != "Style" && tag != "Head" && tag != "Stylesheet"
}

/// Extract function name from a simple call expression like `sidebar_content()`.
/// Returns None for complex expressions.
fn extract_fn_call_name(expr: &str) -> Option<&str> {
    let trimmed = expr.trim();
    let open = trimmed.find('(')?;
    if !trimmed.ends_with(')') || open == 0 {
        return None;
    }
    let name = trimmed[..open].trim();
    // Must be a valid identifier
    if !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Some(name);
    }
    None
}

/// Interpret an element node into a RuntimeHtmlElement.
fn interpret_element(
    tag: &str,
    attrs: &[RsxAttr],
    children: &[RsxNode],
    self_closing: bool,
    fragments: &HashMap<String, Vec<RsxNode>>,
) -> RuntimeHtmlElement {
    let mut runtime_attrs = Vec::new();
    for attr in attrs {
        match &attr.value {
            RsxAttrValue::Literal(v) => runtime_attrs.push((attr.name.clone(), v.clone())),
            RsxAttrValue::Expr(v) => {
                if attr.name.as_str().starts_with("on") {
                    runtime_attrs.push((crate::vformat!("data-volki-{}", attr.name), v.clone()));
                }
            }
        }
    }

    let mut runtime_children = Vec::new();
    if !self_closing {
        for child in children {
            match child {
                RsxNode::Text(s) => {
                    runtime_children.push(RuntimeHtmlNode::Text(s.clone()));
                }
                RsxNode::Expr(expr) => {
                    let html_nodes = interpret_expr(expr.as_str(), fragments);
                    for html_node in html_nodes {
                        match html_node {
                            HtmlNode::Text(t) => runtime_children.push(RuntimeHtmlNode::Text(t)),
                            HtmlNode::Raw(r) => runtime_children.push(RuntimeHtmlNode::Raw(r)),
                            HtmlNode::Element(el) => {
                                // Convert HtmlElement to rendered string
                                let rendered = crate::libs::web::html::render::render_element(&el);
                                runtime_children.push(RuntimeHtmlNode::Raw(rendered));
                            }
                        }
                    }
                }
                RsxNode::Element { tag: child_tag, attrs: child_attrs, children: child_children, self_closing: child_sc } => {
                    // Resolve component tags (PascalCase) to fragment function calls
                    if is_component_like(child_tag.as_str()) {
                        if let Some(frag_nodes) = fragments.get(child_tag.as_str()) {
                            for node in frag_nodes.iter() {
                                let html_nodes = interpret_node(node, fragments);
                                for html_node in html_nodes {
                                    match html_node {
                                        HtmlNode::Text(t) => runtime_children.push(RuntimeHtmlNode::Text(t)),
                                        HtmlNode::Raw(r) => runtime_children.push(RuntimeHtmlNode::Raw(r)),
                                        HtmlNode::Element(el) => {
                                            let rendered = crate::libs::web::html::render::render_element(&el);
                                            runtime_children.push(RuntimeHtmlNode::Raw(rendered));
                                        }
                                    }
                                }
                            }
                            continue;
                        }
                    }
                    let child_el = interpret_element(
                        child_tag.as_str(),
                        child_attrs,
                        child_children,
                        *child_sc,
                        fragments,
                    );
                    runtime_children.push(RuntimeHtmlNode::Element(child_el));
                }
                RsxNode::CondAnd { body, .. } => {
                    for node in body.iter() {
                        let html_nodes = interpret_node(node, fragments);
                        for html_node in html_nodes {
                            match html_node {
                                HtmlNode::Text(t) => runtime_children.push(RuntimeHtmlNode::Text(t)),
                                HtmlNode::Raw(r) => runtime_children.push(RuntimeHtmlNode::Raw(r)),
                                HtmlNode::Element(el) => {
                                    let rendered = crate::libs::web::html::render::render_element(&el);
                                    runtime_children.push(RuntimeHtmlNode::Raw(rendered));
                                }
                            }
                        }
                    }
                }
                RsxNode::Ternary { if_true, .. } => {
                    for node in if_true.iter() {
                        let html_nodes = interpret_node(node, fragments);
                        for html_node in html_nodes {
                            match html_node {
                                HtmlNode::Text(t) => runtime_children.push(RuntimeHtmlNode::Text(t)),
                                HtmlNode::Raw(r) => runtime_children.push(RuntimeHtmlNode::Raw(r)),
                                HtmlNode::Element(el) => {
                                    let rendered = crate::libs::web::html::render::render_element(&el);
                                    runtime_children.push(RuntimeHtmlNode::Raw(rendered));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    RuntimeHtmlElement {
        tag: String::from(tag),
        attrs: runtime_attrs,
        children: runtime_children,
        self_closing,
    }
}

/// Extract metadata values from a `metadata()` function body string.
///
/// Looks for chained method calls like `.title("...")`, `.description("...")`, etc.
pub fn extract_metadata(body: &str) -> Option<ParsedMetadata> {
    let title = extract_string_arg(body, ".title(");
    let description = extract_string_arg(body, ".description(");
    let og_title = extract_string_arg(body, ".og_title(");
    let og_description = extract_string_arg(body, ".og_description(");
    let og_type = extract_string_arg(body, ".og_type(");

    if title.is_none() && description.is_none() && og_title.is_none()
        && og_description.is_none() && og_type.is_none()
    {
        return None;
    }

    Some(ParsedMetadata {
        title,
        description,
        og_title,
        og_description,
        og_type,
    })
}

/// Extract a string argument from a pattern like `.method("value")`.
fn extract_string_arg(source: &str, pattern: &str) -> Option<String> {
    let start = source.find(pattern)?;
    let after = &source[start + pattern.len()..];
    // Expect opening quote
    if !after.starts_with('"') {
        return None;
    }
    let after = &after[1..];
    // Find closing quote (handle escaped quotes)
    let mut i = 0;
    let bytes = after.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(String::from(&after[..i]));
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::web::compiler::parser::{RsxAttr, RsxAttrValue};

    fn s(v: &str) -> String {
        String::from(v)
    }

    #[test]
    fn test_interpret_simple_page() {
        let data = DynamicPageData {
            nodes: crate::vvec![
                RsxNode::Element {
                    tag: s("div"),
                    attrs: crate::vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("main")) }],
                    children: crate::vvec![RsxNode::Text(s("hello"))],
                    self_closing: false,
                }
            ],
            css: s(".main{font-size:16px;}"),
            fragments: HashMap::new(),
            metadata: None,
            client_glue_url: None,
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        assert!(html.contains("<div class=\"main\">hello</div>"));
        assert!(html.contains("<style>.main{font-size:16px;}</style>"));
    }

    #[test]
    fn test_interpret_with_fragment() {
        let mut fragments = HashMap::new();
        fragments.insert(
            s("sidebar"),
            crate::vvec![RsxNode::Element {
                tag: s("nav"),
                attrs: Vec::new(),
                children: crate::vvec![RsxNode::Text(s("nav content"))],
                self_closing: false,
            }],
        );

        let data = DynamicPageData {
            nodes: crate::vvec![
                RsxNode::Element {
                    tag: s("div"),
                    attrs: Vec::new(),
                    children: crate::vvec![RsxNode::Expr(s("sidebar()"))],
                    self_closing: false,
                }
            ],
            css: String::new(),
            fragments,
            metadata: None,
            client_glue_url: None,
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        assert!(html.contains("<nav>nav content</nav>"));
    }

    #[test]
    fn test_interpret_component_tag_resolved() {
        let mut fragments = HashMap::new();
        fragments.insert(
            s("SidebarContent"),
            crate::vvec![RsxNode::Element {
                tag: s("nav"),
                attrs: Vec::new(),
                children: crate::vvec![RsxNode::Text(s("sidebar"))],
                self_closing: false,
            }],
        );

        let data = DynamicPageData {
            nodes: crate::vvec![
                RsxNode::Element {
                    tag: s("div"),
                    attrs: Vec::new(),
                    children: crate::vvec![
                        RsxNode::Element {
                            tag: s("SidebarContent"),
                            attrs: Vec::new(),
                            children: Vec::new(),
                            self_closing: true,
                        }
                    ],
                    self_closing: false,
                }
            ],
            css: String::new(),
            fragments,
            metadata: None,
            client_glue_url: None,
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        // Component should be resolved to its fragment content, not a literal <SidebarContent> tag
        assert!(html.contains("<nav>sidebar</nav>"));
        assert!(!html.contains("<SidebarContent"));
        assert!(!html.contains("sidebarcontent"));
    }

    #[test]
    fn test_interpret_top_level_component_tag() {
        let mut fragments = HashMap::new();
        fragments.insert(
            s("MainContent"),
            crate::vvec![
                RsxNode::Element {
                    tag: s("h1"),
                    attrs: Vec::new(),
                    children: crate::vvec![RsxNode::Text(s("hello"))],
                    self_closing: false,
                },
                RsxNode::Element {
                    tag: s("p"),
                    attrs: Vec::new(),
                    children: crate::vvec![RsxNode::Text(s("world"))],
                    self_closing: false,
                }
            ],
        );

        let data = DynamicPageData {
            nodes: crate::vvec![
                RsxNode::Element {
                    tag: s("MainContent"),
                    attrs: Vec::new(),
                    children: Vec::new(),
                    self_closing: true,
                }
            ],
            css: String::new(),
            fragments,
            metadata: None,
            client_glue_url: None,
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        assert!(html.contains("<h1>hello</h1>"));
        assert!(html.contains("<p>world</p>"));
        assert!(!html.contains("<MainContent"));
        assert!(!html.contains("maincontent"));
    }

    #[test]
    fn test_interpret_unknown_expr_placeholder() {
        let data = DynamicPageData {
            nodes: crate::vvec![RsxNode::Expr(s("complex_fn(a, b)"))],
            css: String::new(),
            fragments: HashMap::new(),
            metadata: None,
            client_glue_url: None,
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        assert!(html.contains("__volki-dev-placeholder"));
        assert!(html.contains("complex_fn(a, b)"));
    }

    #[test]
    fn test_interpret_with_metadata() {
        let data = DynamicPageData {
            nodes: crate::vvec![RsxNode::Text(s("content"))],
            css: String::new(),
            fragments: HashMap::new(),
            metadata: Some(ParsedMetadata {
                title: Some(s("My Page")),
                description: Some(s("A test page")),
                og_title: None,
                og_description: None,
                og_type: None,
            }),
            client_glue_url: None,
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        assert!(html.contains("<title>My Page</title>"));
        assert!(html.contains("name=\"description\""));
        assert!(html.contains("A test page"));
    }

    #[test]
    fn test_interpret_style_element_skipped() {
        let data = DynamicPageData {
            nodes: crate::vvec![
                RsxNode::Element {
                    tag: s("Style"),
                    attrs: Vec::new(),
                    children: crate::vvec![RsxNode::Expr(s("CSS"))],
                    self_closing: false,
                },
                RsxNode::Element {
                    tag: s("div"),
                    attrs: Vec::new(),
                    children: crate::vvec![RsxNode::Text(s("content"))],
                    self_closing: false,
                }
            ],
            css: s("body{margin:0;}"),
            fragments: HashMap::new(),
            metadata: None,
            client_glue_url: None,
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        // CSS should appear once as inline style, not duplicated
        assert!(html.contains("<style>body{margin:0;}</style>"));
        assert!(html.contains("<div>content</div>"));
        // Should NOT contain a literal <Style> tag in body
        assert!(!html.contains("<Style>"));
    }

    #[test]
    fn test_extract_metadata_full() {
        let body = r#"
            Metadata::new()
                .title("volki db editor")
                .description("A web-based database editor")
                .og_title("DB Editor")
                .og_type("website")
        "#;
        let meta = extract_metadata(body).unwrap();
        assert_eq!(meta.title.as_ref().unwrap().as_str(), "volki db editor");
        assert_eq!(meta.description.as_ref().unwrap().as_str(), "A web-based database editor");
        assert_eq!(meta.og_title.as_ref().unwrap().as_str(), "DB Editor");
        assert_eq!(meta.og_type.as_ref().unwrap().as_str(), "website");
        assert!(meta.og_description.is_none());
    }

    #[test]
    fn test_extract_metadata_none() {
        let body = "Response::ok()";
        assert!(extract_metadata(body).is_none());
    }

    #[test]
    fn test_extract_fn_call_name() {
        assert_eq!(extract_fn_call_name("sidebar()"), Some("sidebar"));
        assert_eq!(extract_fn_call_name("sidebar_content()"), Some("sidebar_content"));
        assert_eq!(extract_fn_call_name("complex(a, b)"), Some("complex"));
        assert_eq!(extract_fn_call_name("CSS"), None);
        assert_eq!(extract_fn_call_name(""), None);
    }

    #[test]
    fn test_interpret_self_closing_element() {
        let data = DynamicPageData {
            nodes: crate::vvec![
                RsxNode::Element {
                    tag: s("input"),
                    attrs: crate::vvec![
                        RsxAttr { name: s("type"), value: RsxAttrValue::Literal(s("text")) },
                        RsxAttr { name: s("placeholder"), value: RsxAttrValue::Literal(s("Search...")) }
                    ],
                    children: Vec::new(),
                    self_closing: true,
                }
            ],
            css: String::new(),
            fragments: HashMap::new(),
            metadata: None,
            client_glue_url: None,
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        assert!(html.contains("<input type=\"text\" placeholder=\"Search...\">"));
    }

    #[test]
    fn test_interpret_stylesheet_tag() {
        let data = DynamicPageData {
            nodes: crate::vvec![
                RsxNode::Element {
                    tag: s("Stylesheet"),
                    attrs: crate::vvec![RsxAttr { name: s("href"), value: RsxAttrValue::Literal(s("/styles/app.css")) }],
                    children: Vec::new(),
                    self_closing: true,
                },
                RsxNode::Element {
                    tag: s("div"),
                    attrs: Vec::new(),
                    children: crate::vvec![RsxNode::Text(s("content"))],
                    self_closing: false,
                }
            ],
            css: String::new(),
            fragments: HashMap::new(),
            metadata: None,
            client_glue_url: None,
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        assert!(html.contains("rel=\"stylesheet\""));
        assert!(html.contains("href=\"/styles/app.css\""));
        assert!(html.contains("<div>content</div>"));
    }

    #[test]
    fn test_interpret_injects_client_glue_script() {
        let data = DynamicPageData {
            nodes: crate::vvec![RsxNode::Text(s("content"))],
            css: String::new(),
            fragments: HashMap::new(),
            metadata: None,
            client_glue_url: Some(s("/wasm/page_glue.js")),
        };

        let req = Request::new(
            crate::libs::web::http::method::Method::Get,
            String::from("/"),
            crate::libs::web::http::headers::Headers::new(),
            Vec::new(),
        );
        let doc = interpret_page(&data, &req);
        let html = doc.render();
        assert!(html.contains("<script type=\"module\" src=\"/wasm/page_glue.js\"></script>"));
    }
}
