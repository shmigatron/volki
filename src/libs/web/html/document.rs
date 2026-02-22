//! HtmlDocument — full page builder backed by `dom::Document`.

use super::element::{HtmlNode, meta, link};
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::libs::web::dom::{Document, NodeId};

pub struct HtmlDocument {
    doc: Document,
    _html_node: NodeId,
    head_node: NodeId,
    body_node_id: NodeId,
    lang: String,
    title: Option<String>,
}

impl HtmlDocument {
    pub fn new() -> Self {
        let mut doc = Document::new();
        let html = doc.create_element("html");
        let head = doc.create_element("head");
        let body = doc.create_element("body");
        doc.append_child(doc.root(), html);
        doc.append_child(html, head);
        doc.append_child(html, body);

        Self {
            doc,
            _html_node: html,
            head_node: head,
            body_node_id: body,
            lang: String::from("en"),
            title: None,
        }
    }

    pub fn lang(mut self, lang: &str) -> Self {
        self.lang = String::from(lang);
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(String::from(title));
        self
    }

    pub fn charset(mut self, charset: &str) -> Self {
        let node = meta().attr("charset", charset).into_node();
        let id = self.doc.import_node(&node);
        self.doc.append_child(self.head_node, id);
        self
    }

    pub fn viewport(mut self) -> Self {
        let node = meta()
            .attr("name", "viewport")
            .attr("content", "width=device-width, initial-scale=1")
            .into_node();
        let id = self.doc.import_node(&node);
        self.doc.append_child(self.head_node, id);
        self
    }

    pub fn stylesheet(mut self, href: &str) -> Self {
        let node = link()
            .attr("rel", "stylesheet")
            .attr("href", href)
            .into_node();
        let id = self.doc.import_node(&node);
        self.doc.append_child(self.head_node, id);
        self
    }

    pub fn script(mut self, src: &str) -> Self {
        let node = super::element::script().attr("src", src).into_node();
        let id = self.doc.import_node(&node);
        self.doc.append_child(self.head_node, id);
        self
    }

    /// Add a `<script type="module" src="...">` tag to the document body (end).
    pub fn script_module(mut self, src: &str) -> Self {
        let node = super::element::script()
            .attr("type", "module")
            .attr("src", src)
            .into_node();
        let id = self.doc.import_node(&node);
        self.doc.append_child(self.body_node_id, id);
        self
    }

    pub fn inline_style(mut self, css: &str) -> Self {
        let node = super::element::style().raw(css).into_node();
        let id = self.doc.import_node(&node);
        self.doc.append_child(self.head_node, id);
        self
    }

    pub fn head_node(mut self, node: HtmlNode) -> Self {
        let id = self.doc.import_node(&node);
        self.doc.append_child(self.head_node, id);
        self
    }

    pub fn body_node(mut self, node: HtmlNode) -> Self {
        let id = self.doc.import_node(&node);
        self.doc.append_child(self.body_node_id, id);
        self
    }

    pub fn body_nodes(mut self, nodes: Vec<HtmlNode>) -> Self {
        for node in nodes {
            let id = self.doc.import_node(&node);
            self.doc.append_child(self.body_node_id, id);
        }
        self
    }

    /// Apply a `Metadata` struct: sets title and adds all meta/link tags to head.
    pub fn metadata(mut self, m: &super::metadata::Metadata) -> Self {
        if let Some(ref t) = m.title {
            self.title = Some(t.clone());
        }
        for node in m.render_head_nodes() {
            let id = self.doc.import_node(&node);
            self.doc.append_child(self.head_node, id);
        }
        self
    }

    pub fn render(&self) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str("<!DOCTYPE html>\n<html lang=\"");
        out.push_str(self.lang.as_str());
        out.push_str("\">\n<head>\n");

        if let Some(ref title) = self.title {
            out.push_str("<title>");
            out.push_str(super::escape::escape_html(title.as_str()).as_str());
            out.push_str("</title>\n");
        }

        // Render head children
        for child_id in self.doc.children(self.head_node) {
            let rendered = self.doc.outer_html(child_id);
            out.push_str(rendered.as_str());
            out.push('\n');
        }

        out.push_str("</head>\n<body>\n");

        // Render body children
        for child_id in self.doc.children(self.body_node_id) {
            let rendered = self.doc.outer_html(child_id);
            out.push_str(rendered.as_str());
            out.push('\n');
        }

        out.push_str("</body>\n</html>");
        out
    }

    /// Returns a reference to the underlying `dom::Document`.
    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// Returns a mutable reference to the underlying `dom::Document`.
    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.doc
    }

    /// Returns the NodeId of the `<head>` element.
    pub fn head_id(&self) -> NodeId {
        self.head_node
    }

    /// Returns the NodeId of the `<body>` element.
    pub fn body_id(&self) -> NodeId {
        self.body_node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::web::html::element::*;

    #[test]
    fn test_basic_document() {
        let doc = HtmlDocument::new()
            .title("Test Page")
            .charset("utf-8")
            .viewport()
            .body_node(h1().text("Hello").into_node())
            .body_node(p().text("World").into_node());

        let html = doc.render();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Page</title>"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>World</p>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn test_document_with_style() {
        let doc = HtmlDocument::new()
            .title("Styled")
            .inline_style("body { margin: 0; }");

        let html = doc.render();
        assert!(html.contains("<style>body { margin: 0; }</style>"));
    }

    #[test]
    fn test_document_with_metadata() {
        use crate::libs::web::html::metadata::Metadata;

        let m = Metadata::new()
            .title("Meta Title")
            .description("Meta description")
            .og_title("OG Title")
            .og_type("website");

        let doc = HtmlDocument::new()
            .metadata(&m)
            .body_node(p().text("content").into_node());

        let html = doc.render();
        assert!(html.contains("<title>Meta Title</title>"));
        assert!(html.contains("name=\"description\""));
        assert!(html.contains("Meta description"));
        assert!(html.contains("property=\"og:title\""));
        assert!(html.contains("OG Title"));
        assert!(html.contains("property=\"og:type\""));
        assert!(html.contains("charset=\"utf-8\""));
    }
}
