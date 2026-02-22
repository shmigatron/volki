//! Runtime HTML types — like element.rs but with owned `String` tags.
//!
//! Used by the dev-mode interpreter to build HTML from dynamically-parsed
//! RSX AST nodes. The compile-time `HtmlElement` uses `&'static str` for
//! tags, which can't hold runtime-parsed strings. These types mirror the
//! same structure but are fully owned.

use crate::core::volkiwithstds::collections::{String, Vec};
use super::escape::{escape_html, escape_attr};

/// A runtime HTML node — parallel to `HtmlNode` but with owned tag strings.
pub enum RuntimeHtmlNode {
    Element(RuntimeHtmlElement),
    Text(String),
    Raw(String),
}

/// A runtime HTML element with an owned `String` tag name.
pub struct RuntimeHtmlElement {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<RuntimeHtmlNode>,
    pub self_closing: bool,
}

/// Render a runtime node to an HTML string.
pub fn render_runtime_node(node: &RuntimeHtmlNode) -> String {
    let mut out = String::new();
    render_into(node, &mut out);
    out
}

fn render_into(node: &RuntimeHtmlNode, out: &mut String) {
    match node {
        RuntimeHtmlNode::Text(t) => {
            let escaped = escape_html(t.as_str());
            out.push_str(escaped.as_str());
        }
        RuntimeHtmlNode::Raw(r) => {
            out.push_str(r.as_str());
        }
        RuntimeHtmlNode::Element(el) => {
            render_element_into(el, out);
        }
    }
}

fn render_element_into(el: &RuntimeHtmlElement, out: &mut String) {
    out.push('<');
    out.push_str(el.tag.as_str());

    for (name, value) in el.attrs.iter() {
        out.push(' ');
        out.push_str(name.as_str());
        out.push_str("=\"");
        let escaped = escape_attr(value.as_str());
        out.push_str(escaped.as_str());
        out.push('"');
    }

    if el.self_closing {
        out.push('>');
        return;
    }

    out.push('>');

    for child in el.children.iter() {
        render_into(child, out);
    }

    out.push_str("</");
    out.push_str(el.tag.as_str());
    out.push('>');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> String {
        String::from(v)
    }

    #[test]
    fn test_render_simple_element() {
        let node = RuntimeHtmlNode::Element(RuntimeHtmlElement {
            tag: s("div"),
            attrs: crate::vvec![(s("class"), s("container"))],
            children: crate::vvec![RuntimeHtmlNode::Text(s("Hello"))],
            self_closing: false,
        });
        let html = render_runtime_node(&node);
        assert_eq!(html.as_str(), "<div class=\"container\">Hello</div>");
    }

    #[test]
    fn test_render_self_closing() {
        let node = RuntimeHtmlNode::Element(RuntimeHtmlElement {
            tag: s("br"),
            attrs: Vec::new(),
            children: Vec::new(),
            self_closing: true,
        });
        let html = render_runtime_node(&node);
        assert_eq!(html.as_str(), "<br>");
    }

    #[test]
    fn test_render_nested() {
        let inner = RuntimeHtmlNode::Element(RuntimeHtmlElement {
            tag: s("span"),
            attrs: Vec::new(),
            children: crate::vvec![RuntimeHtmlNode::Text(s("inner"))],
            self_closing: false,
        });
        let outer = RuntimeHtmlNode::Element(RuntimeHtmlElement {
            tag: s("div"),
            attrs: Vec::new(),
            children: crate::vvec![inner],
            self_closing: false,
        });
        let html = render_runtime_node(&outer);
        assert_eq!(html.as_str(), "<div><span>inner</span></div>");
    }

    #[test]
    fn test_render_escaping() {
        let node = RuntimeHtmlNode::Text(s("<script>alert('xss')</script>"));
        let html = render_runtime_node(&node);
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_render_raw() {
        let node = RuntimeHtmlNode::Element(RuntimeHtmlElement {
            tag: s("div"),
            attrs: Vec::new(),
            children: crate::vvec![RuntimeHtmlNode::Raw(s("<b>bold</b>"))],
            self_closing: false,
        });
        let html = render_runtime_node(&node);
        assert_eq!(html.as_str(), "<div><b>bold</b></div>");
    }

    #[test]
    fn test_render_attr_escaping() {
        let node = RuntimeHtmlNode::Element(RuntimeHtmlElement {
            tag: s("div"),
            attrs: crate::vvec![(s("data-value"), s("a\"b"))],
            children: Vec::new(),
            self_closing: false,
        });
        let html = render_runtime_node(&node);
        assert_eq!(html.as_str(), "<div data-value=\"a&quot;b\"></div>");
    }

    #[test]
    fn test_render_multiple_attrs() {
        let node = RuntimeHtmlNode::Element(RuntimeHtmlElement {
            tag: s("a"),
            attrs: crate::vvec![
                (s("href"), s("/about")),
                (s("class"), s("link"))
            ],
            children: crate::vvec![RuntimeHtmlNode::Text(s("About"))],
            self_closing: false,
        });
        let html = render_runtime_node(&node);
        assert_eq!(html.as_str(), "<a href=\"/about\" class=\"link\">About</a>");
    }
}
