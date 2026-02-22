//! DOM node types — arena-stored data for every node in the tree.

use crate::core::volkiwithstds::collections::{String, Vec};

/// A copyable handle into the `Document` arena.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub(crate) usize);

/// DOM node type constants (matching the W3C spec numbers).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeType {
    Element = 1,
    Text = 3,
    Comment = 8,
    Document = 9,
    DocumentFragment = 11,
}

/// The data payload of a node — determines what kind of node it is.
pub enum NodeKind {
    Document,
    Element(ElementData),
    Text(String),
    Comment(String),
    DocumentFragment,
}

/// Element-specific data stored inside `NodeKind::Element`.
pub struct ElementData {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub id: Option<String>,
    pub class_list: Vec<String>,
    pub self_closing: bool,
}

impl ElementData {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: String::from(tag),
            attributes: Vec::new(),
            id: None,
            class_list: Vec::new(),
            self_closing: false,
        }
    }

    pub fn new_void(tag: &str) -> Self {
        Self {
            tag: String::from(tag),
            attributes: Vec::new(),
            id: None,
            class_list: Vec::new(),
            self_closing: true,
        }
    }
}

/// Per-node storage for event listeners.
pub struct EventListenerEntry {
    pub event_type: String,
    pub callback_id: usize,
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
}

/// Arena-stored node data. Every node in the tree is one of these.
pub struct NodeData {
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub event_listeners: Vec<EventListenerEntry>,
    /// Marks this slot as freed (available for reuse).
    pub freed: bool,
}

impl NodeData {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            parent: None,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            event_listeners: Vec::new(),
            freed: false,
        }
    }

    pub fn node_type(&self) -> NodeType {
        match &self.kind {
            NodeKind::Document => NodeType::Document,
            NodeKind::Element(_) => NodeType::Element,
            NodeKind::Text(_) => NodeType::Text,
            NodeKind::Comment(_) => NodeType::Comment,
            NodeKind::DocumentFragment => NodeType::DocumentFragment,
        }
    }
}
