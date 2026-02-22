//! DOM query methods — getElementById, getElementsByTagName, querySelector, etc.

use super::{Document, NodeId};
use super::node::NodeKind;
use super::selector::parse_selector;
use crate::core::volkiwithstds::collections::{String, Vec};

impl Document {
    /// O(1) lookup by element id via the id_index HashMap.
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        self.id_index.get(id).copied()
    }

    /// Returns all element descendants with a given tag name.
    pub fn get_elements_by_tag_name(&self, root: NodeId, tag: &str) -> Vec<NodeId> {
        let mut results = Vec::new();
        for node_id in self.descendants(root) {
            if let NodeKind::Element(ref el) = self.nodes[node_id.0].kind {
                if el.tag.as_str() == tag {
                    results.push(node_id);
                }
            }
        }
        results
    }

    /// Returns all element descendants with a given class name.
    pub fn get_elements_by_class_name(&self, root: NodeId, class: &str) -> Vec<NodeId> {
        let mut results = Vec::new();
        let cls = String::from(class);
        for node_id in self.descendants(root) {
            if let NodeKind::Element(ref el) = self.nodes[node_id.0].kind {
                if el.class_list.contains(&cls) {
                    results.push(node_id);
                }
            }
        }
        results
    }

    /// Returns the first descendant matching a CSS selector, or None.
    pub fn query_selector(&self, root: NodeId, selector_str: &str) -> Option<NodeId> {
        let sel = parse_selector(selector_str)?;
        for node_id in self.descendants(root) {
            if self.matches_selector(node_id, &sel) {
                return Some(node_id);
            }
        }
        None
    }

    /// Returns all descendants matching a CSS selector.
    pub fn query_selector_all(&self, root: NodeId, selector_str: &str) -> Vec<NodeId> {
        let mut results = Vec::new();
        if let Some(sel) = parse_selector(selector_str) {
            for node_id in self.descendants(root) {
                if self.matches_selector(node_id, &sel) {
                    results.push(node_id);
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::super::Document;

    #[test]
    fn test_get_element_by_id() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.set_attribute(div, "id", "main");
        doc.append_child(doc.root, div);

        assert_eq!(doc.get_element_by_id("main"), Some(div));
        assert_eq!(doc.get_element_by_id("other"), None);
    }

    #[test]
    fn test_get_elements_by_tag_name() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let s1 = doc.create_element("span");
        let s2 = doc.create_element("span");
        let p = doc.create_element("p");
        doc.append_child(doc.root, div);
        doc.append_child(div, s1);
        doc.append_child(div, s2);
        doc.append_child(div, p);

        let spans = doc.get_elements_by_tag_name(doc.root, "span");
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_get_elements_by_class_name() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let a = doc.create_element("a");
        let b = doc.create_element("b");
        doc.class_list_add(a, "link");
        doc.class_list_add(b, "link");
        doc.append_child(doc.root, div);
        doc.append_child(div, a);
        doc.append_child(div, b);

        let links = doc.get_elements_by_class_name(doc.root, "link");
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_query_selector() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let p = doc.create_element("p");
        doc.class_list_add(p, "intro");
        doc.append_child(doc.root, div);
        doc.append_child(div, p);

        let found = doc.query_selector(doc.root, "p.intro");
        assert_eq!(found, Some(p));
    }

    #[test]
    fn test_query_selector_all() {
        let mut doc = Document::new();
        let ul = doc.create_element("ul");
        let li1 = doc.create_element("li");
        let li2 = doc.create_element("li");
        let li3 = doc.create_element("li");
        doc.append_child(doc.root, ul);
        doc.append_child(ul, li1);
        doc.append_child(ul, li2);
        doc.append_child(ul, li3);

        let items = doc.query_selector_all(doc.root, "li");
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_query_selector_complex() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.set_attribute(div, "id", "app");
        let ul = doc.create_element("ul");
        let li = doc.create_element("li");
        doc.class_list_add(li, "active");
        doc.append_child(doc.root, div);
        doc.append_child(div, ul);
        doc.append_child(ul, li);

        let found = doc.query_selector(doc.root, "#app ul li.active");
        assert_eq!(found, Some(li));
    }
}
