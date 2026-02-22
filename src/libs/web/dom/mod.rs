//! Arena-based DOM — a full document tree with JS-like manipulation APIs.
//!
//! The `Document` struct owns all nodes in a flat `Vec` arena, referenced by
//! copyable `NodeId` handles. Parent/child/sibling links use `Option<NodeId>`,
//! forming a doubly-linked list per parent. No reference cycles, no lifetime
//! annotations.

pub mod node;
pub mod tree;
pub mod attributes;
pub mod serialize;
pub mod traversal;
pub mod query;
pub mod selector;
pub mod events;
pub mod parse;
pub mod mutation;

pub use node::{NodeId, NodeType, NodeKind, NodeData, ElementData, EventListenerEntry};
pub use events::{Event, EventPhase, CallbackRegistry};

use crate::core::volkiwithstds::collections::{String, Vec, HashMap};
use node::NodeKind as NK;

/// An arena-based DOM document. All nodes live in `self.nodes`.
pub struct Document {
    pub(crate) nodes: Vec<NodeData>,
    pub(crate) root: NodeId,
    pub(crate) free_list: Vec<usize>,
    pub(crate) id_index: HashMap<String, NodeId>,
    pub(crate) mutation_observers: Vec<mutation::MutationObserver>,
}

impl Document {
    /// Creates a new empty document with a root Document node at index 0.
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        nodes.push(NodeData::new(NK::Document));
        Self {
            nodes,
            root: NodeId(0),
            free_list: Vec::new(),
            id_index: HashMap::new(),
            mutation_observers: Vec::new(),
        }
    }

    /// Creates a new document with `<html>`, `<head>`, and `<body>` scaffolding.
    pub fn new_html() -> Self {
        let mut doc = Self::new();
        let html = doc.create_element("html");
        let head = doc.create_element("head");
        let body = doc.create_element("body");
        doc.append_child(doc.root, html);
        doc.append_child(html, head);
        doc.append_child(html, body);
        doc
    }

    /// Allocates a new node in the arena, reusing freed slots.
    pub(crate) fn alloc(&mut self, data: NodeData) -> NodeId {
        if let Some(idx) = self.free_list.pop() {
            self.nodes[idx] = data;
            NodeId(idx)
        } else {
            let idx = self.nodes.len();
            self.nodes.push(data);
            NodeId(idx)
        }
    }

    /// Creates a new Element node in the arena.
    pub fn create_element(&mut self, tag: &str) -> NodeId {
        let el = ElementData::new(tag);
        self.alloc(NodeData::new(NK::Element(el)))
    }

    /// Creates a new void (self-closing) Element node.
    pub fn create_element_void(&mut self, tag: &str) -> NodeId {
        let el = ElementData::new_void(tag);
        self.alloc(NodeData::new(NK::Element(el)))
    }

    /// Creates a new Text node in the arena.
    pub fn create_text(&mut self, text: &str) -> NodeId {
        self.alloc(NodeData::new(NK::Text(String::from(text))))
    }

    /// Creates a new Comment node in the arena.
    pub fn create_comment(&mut self, text: &str) -> NodeId {
        self.alloc(NodeData::new(NK::Comment(String::from(text))))
    }

    /// Creates a new DocumentFragment node in the arena.
    pub fn create_document_fragment(&mut self) -> NodeId {
        self.alloc(NodeData::new(NK::DocumentFragment))
    }

    /// Returns the root document node.
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Returns the `<html>` element (first element child of root), if present.
    pub fn document_element(&self) -> Option<NodeId> {
        let mut child = self.nodes[self.root.0].first_child;
        while let Some(id) = child {
            if let NK::Element(ref el) = self.nodes[id.0].kind {
                if el.tag.as_str() == "html" {
                    return Some(id);
                }
            }
            child = self.nodes[id.0].next_sibling;
        }
        None
    }

    /// Returns the `<head>` element, if present.
    pub fn head(&self) -> Option<NodeId> {
        let html = self.document_element()?;
        let mut child = self.nodes[html.0].first_child;
        while let Some(id) = child {
            if let NK::Element(ref el) = self.nodes[id.0].kind {
                if el.tag.as_str() == "head" {
                    return Some(id);
                }
            }
            child = self.nodes[id.0].next_sibling;
        }
        None
    }

    /// Returns the `<body>` element, if present.
    pub fn body(&self) -> Option<NodeId> {
        let html = self.document_element()?;
        let mut child = self.nodes[html.0].first_child;
        while let Some(id) = child {
            if let NK::Element(ref el) = self.nodes[id.0].kind {
                if el.tag.as_str() == "body" {
                    return Some(id);
                }
            }
            child = self.nodes[id.0].next_sibling;
        }
        None
    }

    /// Returns a reference to the node data at `id`.
    pub fn get(&self, id: NodeId) -> &NodeData {
        debug_assert!(!self.nodes[id.0].freed, "access to freed node");
        &self.nodes[id.0]
    }

    /// Returns a mutable reference to the node data at `id`.
    pub fn get_mut(&mut self, id: NodeId) -> &mut NodeData {
        debug_assert!(!self.nodes[id.0].freed, "access to freed node");
        &mut self.nodes[id.0]
    }

    /// Recursively converts an `HtmlNode` tree into arena nodes.
    /// This is the bridge that lets existing `HtmlElement` builders work with the DOM.
    pub fn import_node(&mut self, node: &crate::libs::web::html::element::HtmlNode) -> NodeId {
        use crate::libs::web::html::element::HtmlNode;

        match node {
            HtmlNode::Text(s) => self.create_text(s.as_str()),
            HtmlNode::Raw(s) => {
                // Raw HTML is preserved as a text node that won't be escaped.
                // We store it as a special element to preserve the raw semantics.
                // For serialization purposes, we wrap it in a DocumentFragment
                // and parse it. But for simplicity and to avoid double-escaping,
                // we store raw content directly.
                let frag = self.create_document_fragment();
                self.parse_html_fragment(frag, s.as_str());
                // If parse produced exactly one child, return it directly
                if self.children_count(frag) == 1 {
                    let child = self.first_child(frag).unwrap();
                    self.remove_child(frag, child);
                    self.free_list.push(frag.0);
                    self.nodes[frag.0].freed = true;
                    child
                } else if self.children_count(frag) == 0 {
                    // Empty raw content — just create an empty text node
                    self.free_list.push(frag.0);
                    self.nodes[frag.0].freed = true;
                    self.create_text("")
                } else {
                    // Multiple children — return the fragment itself
                    frag
                }
            }
            HtmlNode::Element(el) => {
                let node_id = if el.self_closing {
                    self.create_element_void(el.tag)
                } else {
                    self.create_element(el.tag)
                };

                // Import attributes
                for (name, value) in el.attrs.iter() {
                    self.set_attribute(node_id, name.as_str(), value.as_str());
                }

                // Import children recursively
                for child in el.children.iter() {
                    let child_id = self.import_node(child);
                    self.append_child(node_id, child_id);
                }

                node_id
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_document() {
        let doc = Document::new();
        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(doc.get(doc.root).node_type(), NodeType::Document);
    }

    #[test]
    fn test_new_html() {
        let doc = Document::new_html();
        assert!(doc.document_element().is_some());
        assert!(doc.head().is_some());
        assert!(doc.body().is_some());
    }

    #[test]
    fn test_create_element() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        assert_eq!(doc.get(div).node_type(), NodeType::Element);
        if let NK::Element(ref el) = doc.get(div).kind {
            assert_eq!(el.tag.as_str(), "div");
            assert!(!el.self_closing);
        } else {
            panic!("Expected Element");
        }
    }

    #[test]
    fn test_create_element_void() {
        let mut doc = Document::new();
        let br = doc.create_element_void("br");
        if let NK::Element(ref el) = doc.get(br).kind {
            assert!(el.self_closing);
        } else {
            panic!("Expected Element");
        }
    }

    #[test]
    fn test_create_text() {
        let mut doc = Document::new();
        let txt = doc.create_text("hello");
        assert_eq!(doc.get(txt).node_type(), NodeType::Text);
        if let NK::Text(ref s) = doc.get(txt).kind {
            assert_eq!(s.as_str(), "hello");
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_create_comment() {
        let mut doc = Document::new();
        let c = doc.create_comment("a comment");
        assert_eq!(doc.get(c).node_type(), NodeType::Comment);
    }

    #[test]
    fn test_create_document_fragment() {
        let mut doc = Document::new();
        let f = doc.create_document_fragment();
        assert_eq!(doc.get(f).node_type(), NodeType::DocumentFragment);
    }

    #[test]
    fn test_free_list_reuse() {
        let mut doc = Document::new();
        let a = doc.create_element("a");
        let b = doc.create_element("b");
        let len_before = doc.nodes.len();
        // Free slot for 'b'
        doc.nodes[b.0].freed = true;
        doc.free_list.push(b.0);
        // Allocate again — should reuse b's slot
        let c = doc.create_element("c");
        assert_eq!(c.0, b.0);
        assert_eq!(doc.nodes.len(), len_before);
        let _ = a; // suppress unused
    }
}
