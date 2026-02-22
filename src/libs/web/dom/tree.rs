//! Tree manipulation — append, insert, remove, replace, clone.

use super::{Document, NodeId};
use super::node::{NodeData, NodeKind, ElementData};
use crate::core::volkiwithstds::collections::Vec;

impl Document {
    /// Unlinks a node from its current parent (if any) without freeing it.
    pub(crate) fn unlink(&mut self, child: NodeId) {
        let parent_id = self.nodes[child.0].parent;
        let prev = self.nodes[child.0].prev_sibling;
        let next = self.nodes[child.0].next_sibling;

        // Stitch siblings together
        if let Some(p) = prev {
            self.nodes[p.0].next_sibling = next;
        }
        if let Some(n) = next {
            self.nodes[n.0].prev_sibling = prev;
        }

        // Update parent's first_child / last_child
        if let Some(pid) = parent_id {
            if self.nodes[pid.0].first_child == Some(child) {
                self.nodes[pid.0].first_child = next;
            }
            if self.nodes[pid.0].last_child == Some(child) {
                self.nodes[pid.0].last_child = prev;
            }
        }

        self.nodes[child.0].parent = None;
        self.nodes[child.0].prev_sibling = None;
        self.nodes[child.0].next_sibling = None;
    }

    /// Appends `child` as the last child of `parent`.
    /// If `child` is already in the tree, it is first unlinked (DOM re-parenting).
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        // Unlink from old parent if needed
        if self.nodes[child.0].parent.is_some() {
            self.unlink(child);
        }

        self.nodes[child.0].parent = Some(parent);

        let old_last = self.nodes[parent.0].last_child;
        if let Some(last) = old_last {
            self.nodes[last.0].next_sibling = Some(child);
            self.nodes[child.0].prev_sibling = Some(last);
        } else {
            // First child
            self.nodes[parent.0].first_child = Some(child);
        }
        self.nodes[parent.0].last_child = Some(child);

        // Update id_index if child has an id
        if let NodeKind::Element(ref el) = self.nodes[child.0].kind {
            if let Some(ref id) = el.id {
                self.id_index.insert(id.clone(), child);
            }
        }

        self.record_child_list_mutation(parent);
    }

    /// Inserts `new_child` before `reference` under `parent`.
    /// If `reference` is `None`, acts like `append_child`.
    pub fn insert_before(&mut self, parent: NodeId, new_child: NodeId, reference: Option<NodeId>) {
        let reference = match reference {
            Some(r) => r,
            None => return self.append_child(parent, new_child),
        };

        // Unlink from old parent
        if self.nodes[new_child.0].parent.is_some() {
            self.unlink(new_child);
        }

        self.nodes[new_child.0].parent = Some(parent);

        let prev = self.nodes[reference.0].prev_sibling;
        self.nodes[new_child.0].next_sibling = Some(reference);
        self.nodes[new_child.0].prev_sibling = prev;
        self.nodes[reference.0].prev_sibling = Some(new_child);

        if let Some(p) = prev {
            self.nodes[p.0].next_sibling = Some(new_child);
        } else {
            // new_child becomes the first child
            self.nodes[parent.0].first_child = Some(new_child);
        }

        if let NodeKind::Element(ref el) = self.nodes[new_child.0].kind {
            if let Some(ref id) = el.id {
                self.id_index.insert(id.clone(), new_child);
            }
        }

        self.record_child_list_mutation(parent);
    }

    /// Removes `child` from `parent`. The node remains in the arena but is unlinked.
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        if self.nodes[child.0].parent != Some(parent) {
            return;
        }

        // Remove from id_index
        if let NodeKind::Element(ref el) = self.nodes[child.0].kind {
            if let Some(ref id) = el.id {
                self.id_index.remove(id.as_str());
            }
        }

        self.unlink(child);
        self.record_child_list_mutation(parent);
    }

    /// Replaces `old_child` with `new_child` under `parent`.
    pub fn replace_child(&mut self, parent: NodeId, new_child: NodeId, old_child: NodeId) {
        if self.nodes[old_child.0].parent != Some(parent) {
            return;
        }

        // Unlink new_child from its current position
        if self.nodes[new_child.0].parent.is_some() {
            self.unlink(new_child);
        }

        // Insert new_child at old_child's position
        let prev = self.nodes[old_child.0].prev_sibling;
        let next = self.nodes[old_child.0].next_sibling;

        self.nodes[new_child.0].parent = Some(parent);
        self.nodes[new_child.0].prev_sibling = prev;
        self.nodes[new_child.0].next_sibling = next;

        if let Some(p) = prev {
            self.nodes[p.0].next_sibling = Some(new_child);
        } else {
            self.nodes[parent.0].first_child = Some(new_child);
        }

        if let Some(n) = next {
            self.nodes[n.0].prev_sibling = Some(new_child);
        } else {
            self.nodes[parent.0].last_child = Some(new_child);
        }

        // Clean up old_child
        if let NodeKind::Element(ref el) = self.nodes[old_child.0].kind {
            if let Some(ref id) = el.id {
                self.id_index.remove(id.as_str());
            }
        }
        self.nodes[old_child.0].parent = None;
        self.nodes[old_child.0].prev_sibling = None;
        self.nodes[old_child.0].next_sibling = None;

        // Update id_index for new child
        if let NodeKind::Element(ref el) = self.nodes[new_child.0].kind {
            if let Some(ref id) = el.id {
                self.id_index.insert(id.clone(), new_child);
            }
        }

        self.record_child_list_mutation(parent);
    }

    /// Deep or shallow clone of a node. Returns the new node's id.
    pub fn clone_node(&mut self, id: NodeId, deep: bool) -> NodeId {
        let kind = match &self.nodes[id.0].kind {
            NodeKind::Document => NodeKind::Document,
            NodeKind::DocumentFragment => NodeKind::DocumentFragment,
            NodeKind::Text(s) => NodeKind::Text(s.clone()),
            NodeKind::Comment(s) => NodeKind::Comment(s.clone()),
            NodeKind::Element(el) => {
                let mut attrs = Vec::new();
                for (k, v) in el.attributes.iter() {
                    attrs.push((k.clone(), v.clone()));
                }
                let mut class_list = Vec::new();
                for c in el.class_list.iter() {
                    class_list.push(c.clone());
                }
                NodeKind::Element(ElementData {
                    tag: el.tag.clone(),
                    attributes: attrs,
                    id: el.id.clone(),
                    class_list,
                    self_closing: el.self_closing,
                })
            }
        };

        let new_id = self.alloc(NodeData::new(kind));

        if deep {
            let mut child_opt = self.nodes[id.0].first_child;
            while let Some(child) = child_opt {
                let cloned = self.clone_node(child, true);
                self.append_child(new_id, cloned);
                child_opt = self.nodes[child.0].next_sibling;
            }
        }

        new_id
    }

    /// Removes a node and all its descendants, adding their slots to the free list.
    pub fn remove_and_free(&mut self, id: NodeId) {
        // Unlink from parent
        if self.nodes[id.0].parent.is_some() {
            let parent = self.nodes[id.0].parent.unwrap();
            self.remove_child(parent, id);
        }

        // Free descendants depth-first
        self.free_subtree(id);
    }

    /// Recursively frees a node and all descendants.
    fn free_subtree(&mut self, id: NodeId) {
        // Collect children first
        let mut children = Vec::new();
        let mut child_opt = self.nodes[id.0].first_child;
        while let Some(child) = child_opt {
            children.push(child);
            child_opt = self.nodes[child.0].next_sibling;
        }

        for child in children {
            self.free_subtree(child);
        }

        // Remove from id_index
        if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
            if let Some(ref eid) = el.id {
                self.id_index.remove(eid.as_str());
            }
        }

        self.nodes[id.0].freed = true;
        self.nodes[id.0].parent = None;
        self.nodes[id.0].first_child = None;
        self.nodes[id.0].last_child = None;
        self.nodes[id.0].prev_sibling = None;
        self.nodes[id.0].next_sibling = None;
        self.free_list.push(id.0);
    }

    /// Records a ChildList mutation for observers (no-op if no observers).
    fn record_child_list_mutation(&mut self, _target: NodeId) {
        // Mutation recording is handled by the mutation module when observers exist.
        // This is a hook point — zero cost when no observers are registered.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::node::NodeKind as NK;

    #[test]
    fn test_append_child() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let child1 = doc.create_element("span");
        let child2 = doc.create_text("hello");

        doc.append_child(doc.root, parent);
        doc.append_child(parent, child1);
        doc.append_child(parent, child2);

        assert_eq!(doc.get(parent).first_child, Some(child1));
        assert_eq!(doc.get(parent).last_child, Some(child2));
        assert_eq!(doc.get(child1).next_sibling, Some(child2));
        assert_eq!(doc.get(child2).prev_sibling, Some(child1));
        assert_eq!(doc.get(child1).parent, Some(parent));
    }

    #[test]
    fn test_insert_before() {
        let mut doc = Document::new();
        let parent = doc.create_element("ul");
        let li1 = doc.create_element("li");
        let li2 = doc.create_element("li");
        let li3 = doc.create_element("li");

        doc.append_child(parent, li1);
        doc.append_child(parent, li3);
        doc.insert_before(parent, li2, Some(li3));

        assert_eq!(doc.get(parent).first_child, Some(li1));
        assert_eq!(doc.get(li1).next_sibling, Some(li2));
        assert_eq!(doc.get(li2).next_sibling, Some(li3));
        assert_eq!(doc.get(parent).last_child, Some(li3));
    }

    #[test]
    fn test_remove_child() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");
        doc.append_child(parent, child);
        doc.remove_child(parent, child);

        assert_eq!(doc.get(parent).first_child, None);
        assert_eq!(doc.get(parent).last_child, None);
        assert_eq!(doc.get(child).parent, None);
    }

    #[test]
    fn test_replace_child() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let old = doc.create_element("span");
        let new = doc.create_element("p");
        doc.append_child(parent, old);
        doc.replace_child(parent, new, old);

        assert_eq!(doc.get(parent).first_child, Some(new));
        assert_eq!(doc.get(parent).last_child, Some(new));
        assert_eq!(doc.get(old).parent, None);
        assert_eq!(doc.get(new).parent, Some(parent));
    }

    #[test]
    fn test_reparenting() {
        let mut doc = Document::new();
        let p1 = doc.create_element("div");
        let p2 = doc.create_element("div");
        let child = doc.create_element("span");

        doc.append_child(p1, child);
        assert_eq!(doc.get(child).parent, Some(p1));

        // Reparent to p2
        doc.append_child(p2, child);
        assert_eq!(doc.get(child).parent, Some(p2));
        assert_eq!(doc.get(p1).first_child, None);
        assert_eq!(doc.get(p2).first_child, Some(child));
    }

    #[test]
    fn test_clone_node_shallow() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let child = doc.create_text("hello");
        doc.append_child(div, child);

        let cloned = doc.clone_node(div, false);
        assert_eq!(doc.get(cloned).first_child, None); // shallow — no children
        if let NK::Element(ref el) = doc.get(cloned).kind {
            assert_eq!(el.tag.as_str(), "div");
        }
    }

    #[test]
    fn test_clone_node_deep() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let child = doc.create_text("hello");
        doc.append_child(div, child);

        let cloned = doc.clone_node(div, true);
        assert!(doc.get(cloned).first_child.is_some());
        let cc = doc.get(cloned).first_child.unwrap();
        if let NK::Text(ref t) = doc.get(cc).kind {
            assert_eq!(t.as_str(), "hello");
        }
    }

    #[test]
    fn test_remove_and_free() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");
        let grandchild = doc.create_text("hi");
        doc.append_child(doc.root, parent);
        doc.append_child(parent, child);
        doc.append_child(child, grandchild);

        doc.remove_and_free(parent);
        assert!(doc.nodes[parent.0].freed);
        assert!(doc.nodes[child.0].freed);
        assert!(doc.nodes[grandchild.0].freed);
        assert_eq!(doc.free_list.len(), 3);
    }
}
