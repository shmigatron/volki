//! Tree traversal iterators — children, descendants, ancestors.

use super::{Document, NodeId};
use crate::core::volkiwithstds::collections::Vec;

/// Iterates over the direct children of a node.
pub struct ChildIter<'a> {
    doc: &'a Document,
    current: Option<NodeId>,
}

impl<'a> ChildIter<'a> {
    pub fn new(doc: &'a Document, parent: NodeId) -> Self {
        Self {
            doc,
            current: doc.nodes[parent.0].first_child,
        }
    }
}

impl<'a> Iterator for ChildIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let cur = self.current?;
        self.current = self.doc.nodes[cur.0].next_sibling;
        Some(cur)
    }
}

/// Pre-order depth-first traversal of all descendants.
pub struct DescendantIter<'a> {
    doc: &'a Document,
    stack: Vec<NodeId>,
}

impl<'a> DescendantIter<'a> {
    pub fn new(doc: &'a Document, root: NodeId) -> Self {
        let mut stack = Vec::new();
        // Push children in reverse order so first child is visited first
        let mut children = Vec::new();
        let mut child = doc.nodes[root.0].first_child;
        while let Some(c) = child {
            children.push(c);
            child = doc.nodes[c.0].next_sibling;
        }
        let mut i = children.len();
        while i > 0 {
            i -= 1;
            stack.push(children[i]);
        }
        Self { doc, stack }
    }
}

impl<'a> Iterator for DescendantIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let node = self.stack.pop()?;
        // Push children in reverse order
        let mut children = Vec::new();
        let mut child = self.doc.nodes[node.0].first_child;
        while let Some(c) = child {
            children.push(c);
            child = self.doc.nodes[c.0].next_sibling;
        }
        let mut i = children.len();
        while i > 0 {
            i -= 1;
            self.stack.push(children[i]);
        }
        Some(node)
    }
}

/// Walks up the ancestor chain from a node.
pub struct AncestorIter<'a> {
    doc: &'a Document,
    current: Option<NodeId>,
}

impl<'a> AncestorIter<'a> {
    pub fn new(doc: &'a Document, node: NodeId) -> Self {
        Self {
            doc,
            current: doc.nodes[node.0].parent,
        }
    }
}

impl<'a> Iterator for AncestorIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let cur = self.current?;
        self.current = self.doc.nodes[cur.0].parent;
        Some(cur)
    }
}

impl Document {
    /// Returns an iterator over direct children of `id`.
    pub fn children(&self, id: NodeId) -> ChildIter<'_> {
        ChildIter::new(self, id)
    }

    /// Returns an iterator over all descendants (pre-order DFS).
    pub fn descendants(&self, id: NodeId) -> DescendantIter<'_> {
        DescendantIter::new(self, id)
    }

    /// Returns an iterator walking up through ancestors.
    pub fn ancestors(&self, id: NodeId) -> AncestorIter<'_> {
        AncestorIter::new(self, id)
    }

    /// Returns the number of direct children of a node.
    pub fn children_count(&self, id: NodeId) -> usize {
        let mut count = 0;
        let mut child = self.nodes[id.0].first_child;
        while let Some(c) = child {
            count += 1;
            child = self.nodes[c.0].next_sibling;
        }
        count
    }

    /// Returns the nth direct child (0-indexed).
    pub fn nth_child(&self, id: NodeId, n: usize) -> Option<NodeId> {
        let mut child = self.nodes[id.0].first_child;
        let mut i = 0;
        while let Some(c) = child {
            if i == n {
                return Some(c);
            }
            i += 1;
            child = self.nodes[c.0].next_sibling;
        }
        None
    }

    /// Returns the parent of a node.
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.0].parent
    }

    /// Returns the first child of a node.
    pub fn first_child(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.0].first_child
    }

    /// Returns the last child of a node.
    pub fn last_child(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.0].last_child
    }

    /// Returns the next sibling of a node.
    pub fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.0].next_sibling
    }

    /// Returns the previous sibling of a node.
    pub fn prev_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.0].prev_sibling
    }
}

#[cfg(test)]
mod tests {
    use super::super::Document;

    #[test]
    fn test_children_iter() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let c1 = doc.create_element("a");
        let c2 = doc.create_element("b");
        let c3 = doc.create_element("c");
        doc.append_child(parent, c1);
        doc.append_child(parent, c2);
        doc.append_child(parent, c3);

        let ids: Vec<_> = doc.children(parent).collect();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], c1);
        assert_eq!(ids[1], c2);
        assert_eq!(ids[2], c3);
    }

    #[test]
    fn test_descendants_iter() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span = doc.create_element("span");
        let txt = doc.create_text("hi");
        let p = doc.create_element("p");
        doc.append_child(div, span);
        doc.append_child(span, txt);
        doc.append_child(div, p);

        let ids: Vec<_> = doc.descendants(div).collect();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], span);
        assert_eq!(ids[1], txt);
        assert_eq!(ids[2], p);
    }

    #[test]
    fn test_ancestor_iter() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span = doc.create_element("span");
        let txt = doc.create_text("hi");
        doc.append_child(doc.root, div);
        doc.append_child(div, span);
        doc.append_child(span, txt);

        let ancestors: Vec<_> = doc.ancestors(txt).collect();
        assert_eq!(ancestors.len(), 3);
        assert_eq!(ancestors[0], span);
        assert_eq!(ancestors[1], div);
        assert_eq!(ancestors[2], doc.root);
    }

    #[test]
    fn test_children_count() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let a = doc.create_element("a");
        let b = doc.create_element("b");
        doc.append_child(parent, a);
        doc.append_child(parent, b);
        assert_eq!(doc.children_count(parent), 2);
    }

    #[test]
    fn test_nth_child() {
        let mut doc = Document::new();
        let parent = doc.create_element("ul");
        let li0 = doc.create_element("li");
        let li1 = doc.create_element("li");
        doc.append_child(parent, li0);
        doc.append_child(parent, li1);

        assert_eq!(doc.nth_child(parent, 0), Some(li0));
        assert_eq!(doc.nth_child(parent, 1), Some(li1));
        assert_eq!(doc.nth_child(parent, 2), None);
    }

    use crate::core::volkiwithstds::collections::Vec;

    #[test]
    fn test_navigation() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let c1 = doc.create_element("a");
        let c2 = doc.create_element("b");
        doc.append_child(parent, c1);
        doc.append_child(parent, c2);

        assert_eq!(doc.first_child(parent), Some(c1));
        assert_eq!(doc.last_child(parent), Some(c2));
        assert_eq!(doc.next_sibling(c1), Some(c2));
        assert_eq!(doc.prev_sibling(c2), Some(c1));
        assert_eq!(doc.parent(c1), Some(parent));
    }
}
