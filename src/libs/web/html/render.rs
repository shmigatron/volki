//! HTML node-to-string rendering.

use super::element::{HtmlElement, HtmlNode};
use super::escape::escape_html;
use crate::core::volkiwithstds::collections::String;

pub fn render_node(node: &HtmlNode) -> String {
    let mut out = String::new();
    render_into(node, &mut out);
    out
}

pub fn render_element(el: &HtmlElement) -> String {
    let mut out = String::new();
    render_element_into(el, &mut out);
    out
}

fn render_into(node: &HtmlNode, out: &mut String) {
    match node {
        HtmlNode::Text(t) => {
            let escaped = escape_html(t.as_str());
            out.push_str(escaped.as_str());
        }
        HtmlNode::Raw(r) => {
            out.push_str(r.as_str());
        }
        HtmlNode::Element(el) => {
            render_element_into(el, out);
        }
    }
}

fn render_element_into(el: &HtmlElement, out: &mut String) {
    out.push('<');
    out.push_str(el.tag);

    for (name, value) in el.attrs.iter() {
        out.push(' ');
        out.push_str(name.as_str());
        out.push_str("=\"");
        let escaped = super::escape::escape_attr(value.as_str());
        out.push_str(escaped.as_str());
        out.push('"');
    }

    if el.self_closing {
        out.push_str(">");
        return;
    }

    out.push('>');

    for child in el.children.iter() {
        render_into(child, out);
    }

    out.push_str("</");
    out.push_str(el.tag);
    out.push('>');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::web::html::element::*;

    #[test]
    fn test_render_simple() {
        let el = div().class("container").text("Hello");
        let html = render_element(&el);
        assert_eq!(html.as_str(), "<div class=\"container\">Hello</div>");
    }

    #[test]
    fn test_render_nested() {
        let el = ul().child(li().text("one").into_node()).child(li().text("two").into_node());
        let html = render_element(&el);
        assert_eq!(html.as_str(), "<ul><li>one</li><li>two</li></ul>");
    }

    #[test]
    fn test_render_void() {
        let el = br();
        let html = render_element(&el);
        assert_eq!(html.as_str(), "<br>");
    }

    #[test]
    fn test_render_escaping() {
        let el = p().text("<script>alert('xss')</script>");
        let html = render_element(&el);
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_render_raw() {
        let el = div().raw("<b>bold</b>");
        let html = render_element(&el);
        assert_eq!(html.as_str(), "<div><b>bold</b></div>");
    }

    #[test]
    fn test_render_attr_escaping() {
        let el = div().attr("data-value", "a\"b");
        let html = render_element(&el);
        assert_eq!(html.as_str(), "<div data-value=\"a&quot;b\"></div>");
    }
}
