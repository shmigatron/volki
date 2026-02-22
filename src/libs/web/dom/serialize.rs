//! HTML serialization — inner_html, outer_html, render_document.

use super::{Document, NodeId};
use super::node::NodeKind;
use crate::core::volkiwithstds::collections::String;
use crate::libs::web::html::escape::{escape_html, escape_attr};

impl Document {
    /// Returns the inner HTML of a node (its children serialized).
    pub fn inner_html(&self, id: NodeId) -> String {
        let mut out = String::new();
        let mut child = self.nodes[id.0].first_child;
        while let Some(c) = child {
            self.serialize_node(c, &mut out);
            child = self.nodes[c.0].next_sibling;
        }
        out
    }

    /// Returns the outer HTML of a node (the node itself + its children).
    pub fn outer_html(&self, id: NodeId) -> String {
        let mut out = String::new();
        self.serialize_node(id, &mut out);
        out
    }

    /// Renders a full `<!DOCTYPE html>` document string.
    pub fn render_document(&self) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str("<!DOCTYPE html>\n");

        if let Some(html) = self.document_element() {
            self.serialize_node(html, &mut out);
        }

        out
    }

    fn serialize_node(&self, id: NodeId, out: &mut String) {
        match &self.nodes[id.0].kind {
            NodeKind::Text(t) => {
                let escaped = escape_html(t.as_str());
                out.push_str(escaped.as_str());
            }
            NodeKind::Comment(c) => {
                out.push_str("<!--");
                out.push_str(c.as_str());
                out.push_str("-->");
            }
            NodeKind::Element(el) => {
                out.push('<');
                out.push_str(el.tag.as_str());

                for (name, value) in el.attributes.iter() {
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

                let mut child = self.nodes[id.0].first_child;
                while let Some(c) = child {
                    self.serialize_node(c, out);
                    child = self.nodes[c.0].next_sibling;
                }

                out.push_str("</");
                out.push_str(el.tag.as_str());
                out.push('>');
            }
            NodeKind::Document | NodeKind::DocumentFragment => {
                let mut child = self.nodes[id.0].first_child;
                while let Some(c) = child {
                    self.serialize_node(c, out);
                    child = self.nodes[c.0].next_sibling;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Document;

    #[test]
    fn test_outer_html_simple() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "box");
        let txt = doc.create_text("hello");
        doc.append_child(div, txt);

        assert_eq!(doc.outer_html(div).as_str(), "<div class=\"box\">hello</div>");
    }

    #[test]
    fn test_inner_html() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span = doc.create_element("span");
        let txt = doc.create_text("hi");
        doc.append_child(div, span);
        doc.append_child(span, txt);

        assert_eq!(doc.inner_html(div).as_str(), "<span>hi</span>");
    }

    #[test]
    fn test_outer_html_void() {
        let mut doc = Document::new();
        let br = doc.create_element_void("br");
        assert_eq!(doc.outer_html(br).as_str(), "<br>");
    }

    #[test]
    fn test_outer_html_nested() {
        let mut doc = Document::new();
        let ul = doc.create_element("ul");
        let li1 = doc.create_element("li");
        let li2 = doc.create_element("li");
        let t1 = doc.create_text("one");
        let t2 = doc.create_text("two");
        doc.append_child(ul, li1);
        doc.append_child(ul, li2);
        doc.append_child(li1, t1);
        doc.append_child(li2, t2);

        assert_eq!(doc.outer_html(ul).as_str(), "<ul><li>one</li><li>two</li></ul>");
    }

    #[test]
    fn test_text_escaping() {
        let mut doc = Document::new();
        let p = doc.create_element("p");
        let txt = doc.create_text("<script>alert('xss')</script>");
        doc.append_child(p, txt);

        let html = doc.outer_html(p);
        assert!(html.as_str().contains("&lt;script&gt;"));
    }

    #[test]
    fn test_attr_escaping() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.set_attribute(div, "data-val", "a\"b");

        let html = doc.outer_html(div);
        assert_eq!(html.as_str(), "<div data-val=\"a&quot;b\"></div>");
    }

    #[test]
    fn test_comment_serialization() {
        let mut doc = Document::new();
        let c = doc.create_comment(" a comment ");
        assert_eq!(doc.outer_html(c).as_str(), "<!-- a comment -->");
    }

    #[test]
    fn test_render_document() {
        let doc = Document::new_html();
        let html = doc.render_document();
        assert!(html.as_str().contains("<!DOCTYPE html>"));
        assert!(html.as_str().contains("<html>"));
        assert!(html.as_str().contains("<head></head>"));
        assert!(html.as_str().contains("<body></body>"));
        assert!(html.as_str().contains("</html>"));
    }
}
