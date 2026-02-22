//! Attribute and text content operations on DOM nodes.

use super::{Document, NodeId};
use super::node::NodeKind;
use crate::core::volkiwithstds::collections::{String, Vec};

impl Document {
    /// Returns the tag name of an element node.
    pub fn tag_name(&self, id: NodeId) -> Option<&str> {
        if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
            Some(el.tag.as_str())
        } else {
            None
        }
    }

    /// Gets the value of an attribute on an element.
    pub fn get_attribute(&self, id: NodeId, name: &str) -> Option<&str> {
        if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
            for (k, v) in el.attributes.iter() {
                if k.as_str() == name {
                    return Some(v.as_str());
                }
            }
        }
        None
    }

    /// Sets an attribute on an element. Handles special cases for "id" and "class".
    pub fn set_attribute(&mut self, id: NodeId, name: &str, value: &str) {
        if let NodeKind::Element(ref mut el) = self.nodes[id.0].kind {
            // Special handling for "id"
            if name == "id" {
                // Remove old id from index
                if let Some(ref old_id) = el.id {
                    self.id_index.remove(old_id.as_str());
                }
                el.id = Some(String::from(value));
                self.id_index.insert(String::from(value), id);
            }

            // Special handling for "class"
            if name == "class" {
                el.class_list = Vec::new();
                for cls in value.split(' ') {
                    if !cls.is_empty() {
                        el.class_list.push(String::from(cls));
                    }
                }
            }

            // Update or insert in attribute list
            for (k, v) in el.attributes.iter_mut() {
                if k.as_str() == name {
                    *v = String::from(value);
                    return;
                }
            }
            el.attributes.push((String::from(name), String::from(value)));
        }
    }

    /// Removes an attribute from an element.
    pub fn remove_attribute(&mut self, id: NodeId, name: &str) {
        if let NodeKind::Element(ref mut el) = self.nodes[id.0].kind {
            if name == "id" {
                if let Some(ref old_id) = el.id {
                    self.id_index.remove(old_id.as_str());
                }
                el.id = None;
            }
            if name == "class" {
                el.class_list = Vec::new();
            }

            el.attributes.retain(|(k, _)| k.as_str() != name);
        }
    }

    /// Checks whether an element has a given attribute.
    pub fn has_attribute(&self, id: NodeId, name: &str) -> bool {
        self.get_attribute(id, name).is_some()
    }

    /// Adds a class to the element's class list.
    pub fn class_list_add(&mut self, id: NodeId, class: &str) {
        if let NodeKind::Element(ref mut el) = self.nodes[id.0].kind {
            let cls = String::from(class);
            if !el.class_list.contains(&cls) {
                el.class_list.push(cls);
                self.sync_class_attribute(id);
            }
        }
    }

    /// Removes a class from the element's class list.
    pub fn class_list_remove(&mut self, id: NodeId, class: &str) {
        if let NodeKind::Element(ref mut el) = self.nodes[id.0].kind {
            let cls = String::from(class);
            el.class_list.retain(|c| *c != cls);
            self.sync_class_attribute(id);
        }
    }

    /// Toggles a class on the element. Returns `true` if the class is now present.
    pub fn class_list_toggle(&mut self, id: NodeId, class: &str) -> bool {
        if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
            let cls = String::from(class);
            if el.class_list.contains(&cls) {
                self.class_list_remove(id, class);
                return false;
            }
        }
        self.class_list_add(id, class);
        true
    }

    /// Checks whether the element's class list contains a given class.
    pub fn class_list_contains(&self, id: NodeId, class: &str) -> bool {
        if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
            let cls = String::from(class);
            return el.class_list.contains(&cls);
        }
        false
    }

    /// Syncs the class_list back to the "class" attribute.
    fn sync_class_attribute(&mut self, id: NodeId) {
        if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
            let class_str = if el.class_list.is_empty() {
                None
            } else {
                let mut s = String::new();
                for (i, c) in el.class_list.iter().enumerate() {
                    if i > 0 {
                        s.push(' ');
                    }
                    s.push_str(c.as_str());
                }
                Some(s)
            };

            // Now update the attribute
            if let NodeKind::Element(ref mut el) = self.nodes[id.0].kind {
                match class_str {
                    Some(ref val) => {
                        for (k, v) in el.attributes.iter_mut() {
                            if k.as_str() == "class" {
                                *v = val.clone();
                                return;
                            }
                        }
                        el.attributes.push((String::from("class"), val.clone()));
                    }
                    None => {
                        el.attributes.retain(|(k, _)| k.as_str() != "class");
                    }
                }
            }
        }
    }

    /// Returns the concatenated text content of a node and all its descendants.
    pub fn text_content(&self, id: NodeId) -> String {
        let mut out = String::new();
        self.collect_text(id, &mut out);
        out
    }

    fn collect_text(&self, id: NodeId, out: &mut String) {
        match &self.nodes[id.0].kind {
            NodeKind::Text(s) => out.push_str(s.as_str()),
            _ => {
                let mut child = self.nodes[id.0].first_child;
                while let Some(c) = child {
                    self.collect_text(c, out);
                    child = self.nodes[c.0].next_sibling;
                }
            }
        }
    }

    /// Removes all children and sets the text content to a single text node.
    pub fn set_text_content(&mut self, id: NodeId, text: &str) {
        // Remove all children
        let mut children = Vec::new();
        let mut child = self.nodes[id.0].first_child;
        while let Some(c) = child {
            let next = self.nodes[c.0].next_sibling;
            children.push(c);
            child = next;
        }
        for c in children {
            self.remove_and_free(c);
        }

        // Add a text node
        if !text.is_empty() {
            let txt = self.create_text(text);
            self.append_child(id, txt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Document;

    #[test]
    fn test_set_get_attribute() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.set_attribute(div, "data-x", "42");
        assert_eq!(doc.get_attribute(div, "data-x"), Some("42"));
        assert_eq!(doc.get_attribute(div, "data-y"), None);
    }

    #[test]
    fn test_remove_attribute() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.set_attribute(div, "data-x", "42");
        doc.remove_attribute(div, "data-x");
        assert!(!doc.has_attribute(div, "data-x"));
    }

    #[test]
    fn test_id_attribute() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.set_attribute(div, "id", "main");
        assert_eq!(doc.get_attribute(div, "id"), Some("main"));
        assert!(doc.id_index.contains_key("main"));

        doc.set_attribute(div, "id", "other");
        assert!(!doc.id_index.contains_key("main"));
        assert!(doc.id_index.contains_key("other"));
    }

    #[test]
    fn test_class_list() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.class_list_add(div, "foo");
        doc.class_list_add(div, "bar");
        assert!(doc.class_list_contains(div, "foo"));
        assert!(doc.class_list_contains(div, "bar"));

        doc.class_list_remove(div, "foo");
        assert!(!doc.class_list_contains(div, "foo"));
        assert!(doc.class_list_contains(div, "bar"));
    }

    #[test]
    fn test_class_list_toggle() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        assert!(doc.class_list_toggle(div, "active"));
        assert!(doc.class_list_contains(div, "active"));
        assert!(!doc.class_list_toggle(div, "active"));
        assert!(!doc.class_list_contains(div, "active"));
    }

    #[test]
    fn test_set_class_attribute_syncs() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.set_attribute(div, "class", "foo bar baz");
        assert!(doc.class_list_contains(div, "foo"));
        assert!(doc.class_list_contains(div, "bar"));
        assert!(doc.class_list_contains(div, "baz"));
    }

    #[test]
    fn test_text_content() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let t1 = doc.create_text("hello ");
        let span = doc.create_element("span");
        let t2 = doc.create_text("world");
        doc.append_child(div, t1);
        doc.append_child(div, span);
        doc.append_child(span, t2);

        assert_eq!(doc.text_content(div).as_str(), "hello world");
    }

    #[test]
    fn test_set_text_content() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let child = doc.create_element("span");
        doc.append_child(div, child);

        doc.set_text_content(div, "new text");
        assert_eq!(doc.text_content(div).as_str(), "new text");
        // The old child's slot may be reused by the new text node,
        // so we verify the div has exactly one text child.
        assert_eq!(doc.children_count(div), 1);
    }

    #[test]
    fn test_tag_name() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        assert_eq!(doc.tag_name(div), Some("div"));

        let txt = doc.create_text("hi");
        assert_eq!(doc.tag_name(txt), None);
    }
}
