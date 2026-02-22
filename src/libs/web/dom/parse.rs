//! HTML fragment parser — parses trusted HTML strings into DOM nodes.

use super::{Document, NodeId};
use super::node::{ElementData, NodeKind};
use crate::core::volkiwithstds::collections::{String, Vec};

/// Set of void (self-closing) HTML elements.
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
            | "link" | "meta" | "param" | "source" | "track" | "wbr"
    )
}

impl Document {
    /// Parses an HTML fragment and appends the resulting nodes as children of `parent`.
    pub fn parse_html_fragment(&mut self, parent: NodeId, html: &str) {
        let mut parser = FragmentParser::new(html);
        while let Some(token) = parser.next_token() {
            match token {
                Token::OpenTag { tag, attrs, self_closing } => {
                    let is_void = self_closing || is_void_element(tag.as_str());
                    let mut el_data = if is_void {
                        ElementData::new_void(tag.as_str())
                    } else {
                        ElementData::new(tag.as_str())
                    };

                    // Process attributes
                    for (name, value) in attrs.iter() {
                        if name.as_str() == "id" {
                            el_data.id = Some(value.clone());
                        }
                        if name.as_str() == "class" {
                            for cls in value.as_str().split(' ') {
                                if !cls.is_empty() {
                                    el_data.class_list.push(String::from(cls));
                                }
                            }
                        }
                        el_data.attributes.push((name.clone(), value.clone()));
                    }

                    let node_id = self.alloc(super::node::NodeData::new(NodeKind::Element(el_data)));

                    // Update id_index
                    if let NodeKind::Element(ref el) = self.nodes[node_id.0].kind {
                        if let Some(ref id) = el.id {
                            self.id_index.insert(id.clone(), node_id);
                        }
                    }

                    self.append_child(parent, node_id);

                    if !is_void {
                        // Recursively parse children until we hit the closing tag
                        self.parse_children(node_id, &mut parser, tag.as_str());
                    }
                }
                Token::Text(text) => {
                    if !text.is_empty() {
                        let decoded = decode_entities(text.as_str());
                        let txt = self.create_text(decoded.as_str());
                        self.append_child(parent, txt);
                    }
                }
                Token::Comment(text) => {
                    let c = self.create_comment(text.as_str());
                    self.append_child(parent, c);
                }
                Token::CloseTag { .. } => {
                    // Unexpected close tag at top level — ignore
                }
            }
        }
    }

    fn parse_children(&mut self, parent: NodeId, parser: &mut FragmentParser<'_>, parent_tag: &str) {
        while let Some(token) = parser.next_token() {
            match token {
                Token::CloseTag { ref tag } if tag.as_str() == parent_tag => {
                    return;
                }
                Token::OpenTag { tag, attrs, self_closing } => {
                    let is_void = self_closing || is_void_element(tag.as_str());
                    let mut el_data = if is_void {
                        ElementData::new_void(tag.as_str())
                    } else {
                        ElementData::new(tag.as_str())
                    };

                    for (name, value) in attrs.iter() {
                        if name.as_str() == "id" {
                            el_data.id = Some(value.clone());
                        }
                        if name.as_str() == "class" {
                            for cls in value.as_str().split(' ') {
                                if !cls.is_empty() {
                                    el_data.class_list.push(String::from(cls));
                                }
                            }
                        }
                        el_data.attributes.push((name.clone(), value.clone()));
                    }

                    let node_id = self.alloc(super::node::NodeData::new(NodeKind::Element(el_data)));

                    if let NodeKind::Element(ref el) = self.nodes[node_id.0].kind {
                        if let Some(ref id) = el.id {
                            self.id_index.insert(id.clone(), node_id);
                        }
                    }

                    self.append_child(parent, node_id);

                    if !is_void {
                        self.parse_children(node_id, parser, tag.as_str());
                    }
                }
                Token::Text(text) => {
                    if !text.is_empty() {
                        let decoded = decode_entities(text.as_str());
                        let txt = self.create_text(decoded.as_str());
                        self.append_child(parent, txt);
                    }
                }
                Token::Comment(text) => {
                    let c = self.create_comment(text.as_str());
                    self.append_child(parent, c);
                }
                Token::CloseTag { .. } => {
                    // Mismatched close tag — ignore
                }
            }
        }
    }

    /// Sets the innerHTML of a node by parsing an HTML string.
    pub fn set_inner_html(&mut self, id: NodeId, html: &str) {
        // Remove all existing children
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

        // Parse new content
        self.parse_html_fragment(id, html);
    }
}

// ── Tokenizer ───────────────────────────────────────────────────────────────

enum Token {
    OpenTag {
        tag: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
    },
    CloseTag {
        tag: String,
    },
    Text(String),
    Comment(String),
}

struct FragmentParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> FragmentParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.pos >= self.input.len() {
            return None;
        }

        if self.remaining().starts_with("<!--") {
            return Some(self.parse_comment());
        }

        if self.peek() == Some('<') {
            if self.remaining().starts_with("</") {
                return Some(self.parse_close_tag());
            }
            return Some(self.parse_open_tag());
        }

        Some(self.parse_text())
    }

    fn parse_text(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && self.peek() != Some('<') {
            self.advance();
        }
        Token::Text(String::from(&self.input[start..self.pos]))
    }

    fn parse_comment(&mut self) -> Token {
        // Skip "<!--"
        self.pos += 4;
        let start = self.pos;
        while self.pos < self.input.len() {
            if self.remaining().starts_with("-->") {
                let text = &self.input[start..self.pos];
                self.pos += 3;
                return Token::Comment(String::from(text));
            }
            self.advance();
        }
        Token::Comment(String::from(&self.input[start..self.pos]))
    }

    fn parse_close_tag(&mut self) -> Token {
        // Skip "</"
        self.pos += 2;
        let tag = self.parse_tag_name();
        self.skip_until('>');
        self.advance(); // consume '>'
        Token::CloseTag { tag }
    }

    fn parse_open_tag(&mut self) -> Token {
        // Skip "<"
        self.advance();
        let tag = self.parse_tag_name();
        let mut attrs = Vec::new();
        let mut self_closing = false;

        loop {
            self.skip_ws();
            match self.peek() {
                None => break,
                Some('>') => {
                    self.advance();
                    break;
                }
                Some('/') => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                    }
                    self_closing = true;
                    break;
                }
                _ => {
                    if let Some((name, value)) = self.parse_attribute() {
                        attrs.push((name, value));
                    } else {
                        self.advance(); // skip unknown char
                    }
                }
            }
        }

        Token::OpenTag { tag, attrs, self_closing }
    }

    fn parse_tag_name(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let name = &self.input[start..self.pos];
        // Lowercase the tag name
        let mut s = String::with_capacity(name.len());
        for c in name.chars() {
            if c >= 'A' && c <= 'Z' {
                s.push((c as u8 + 32) as char);
            } else {
                s.push(c);
            }
        }
        s
    }

    fn parse_attribute(&mut self) -> Option<(String, String)> {
        let name = self.parse_attr_name()?;
        self.skip_ws();

        if self.peek() != Some('=') {
            return Some((name, String::from("")));
        }
        self.advance(); // consume '='
        self.skip_ws();

        let value = self.parse_attr_value();
        Some((name, value))
    }

    fn parse_attr_name(&mut self) -> Option<String> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '@' {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            return None;
        }
        // Lowercase attribute names (same as tag names) so id/class detection works
        let raw = &self.input[start..self.pos];
        let mut name = String::new();
        for c in raw.chars() {
            if c >= 'A' && c <= 'Z' {
                name.push((c as u8 + 32) as char);
            } else {
                name.push(c);
            }
        }
        Some(name)
    }

    fn parse_attr_value(&mut self) -> String {
        match self.peek() {
            Some('"') | Some('\'') => {
                let quote = self.advance().unwrap();
                let start = self.pos;
                while self.peek() != Some(quote) && self.peek().is_some() {
                    self.advance();
                }
                let val = String::from(&self.input[start..self.pos]);
                self.advance(); // closing quote
                val
            }
            _ => {
                let start = self.pos;
                while let Some(c) = self.peek() {
                    if c == ' ' || c == '>' || c == '/' {
                        break;
                    }
                    self.advance();
                }
                String::from(&self.input[start..self.pos])
            }
        }
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_until(&mut self, target: char) {
        while let Some(c) = self.peek() {
            if c == target {
                return;
            }
            self.advance();
        }
    }
}

/// Decodes basic HTML entities.
fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'&' {
            let rest = &input[i..];
            if rest.starts_with("&amp;") {
                out.push('&');
                i += 5;
            } else if rest.starts_with("&lt;") {
                out.push('<');
                i += 4;
            } else if rest.starts_with("&gt;") {
                out.push('>');
                i += 4;
            } else if rest.starts_with("&quot;") {
                out.push('"');
                i += 6;
            } else if rest.starts_with("&#x27;") || rest.starts_with("&apos;") {
                out.push('\'');
                i += 6;
            } else if rest.starts_with("&#") {
                // Numeric entity
                let end = rest.find(';').unwrap_or(rest.len());
                let num_str = &rest[2..end];
                let code = if num_str.starts_with('x') || num_str.starts_with('X') {
                    u32::from_str_radix(&num_str[1..], 16).ok()
                } else {
                    let mut val: u32 = 0;
                    let mut valid = true;
                    for c in num_str.chars() {
                        if c.is_ascii_digit() {
                            val = val * 10 + (c as u32 - '0' as u32);
                        } else {
                            valid = false;
                            break;
                        }
                    }
                    if valid { Some(val) } else { None }
                };
                if let Some(code) = code {
                    if let Some(ch) = char::from_u32(code) {
                        out.push(ch);
                        i += end + 1;
                        continue;
                    }
                }
                out.push('&');
                i += 1;
            } else {
                out.push('&');
                i += 1;
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Document;
    use super::super::node::NodeKind;

    #[test]
    fn test_parse_simple() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        doc.parse_html_fragment(parent, "<p>Hello</p>");

        let p = doc.first_child(parent).unwrap();
        assert_eq!(doc.tag_name(p), Some("p"));
        assert_eq!(doc.text_content(p).as_str(), "Hello");
    }

    #[test]
    fn test_parse_nested() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        doc.parse_html_fragment(parent, "<ul><li>one</li><li>two</li></ul>");

        let ul = doc.first_child(parent).unwrap();
        assert_eq!(doc.tag_name(ul), Some("ul"));
        assert_eq!(doc.children_count(ul), 2);
    }

    #[test]
    fn test_parse_void() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        doc.parse_html_fragment(parent, "<br><hr>");

        assert_eq!(doc.children_count(parent), 2);
        let br = doc.first_child(parent).unwrap();
        if let NodeKind::Element(ref el) = doc.get(br).kind {
            assert!(el.self_closing);
        }
    }

    #[test]
    fn test_parse_attributes() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        doc.parse_html_fragment(parent, "<a href=\"/about\" class=\"link\">About</a>");

        let a = doc.first_child(parent).unwrap();
        assert_eq!(doc.get_attribute(a, "href"), Some("/about"));
        assert_eq!(doc.get_attribute(a, "class"), Some("link"));
        assert!(doc.class_list_contains(a, "link"));
    }

    #[test]
    fn test_parse_comment() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        doc.parse_html_fragment(parent, "<!-- a comment --><p>hi</p>");

        let comment = doc.first_child(parent).unwrap();
        if let NodeKind::Comment(ref c) = doc.get(comment).kind {
            assert_eq!(c.as_str(), " a comment ");
        } else {
            panic!("Expected Comment");
        }
    }

    #[test]
    fn test_set_inner_html() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let old = doc.create_text("old");
        doc.append_child(div, old);

        doc.set_inner_html(div, "<p>new</p>");
        assert_eq!(doc.inner_html(div).as_str(), "<p>new</p>");
    }

    #[test]
    fn test_decode_entities() {
        assert_eq!(decode_entities("&amp;").as_str(), "&");
        assert_eq!(decode_entities("&lt;b&gt;").as_str(), "<b>");
        assert_eq!(decode_entities("&quot;hi&quot;").as_str(), "\"hi\"");
        assert_eq!(decode_entities("no entities").as_str(), "no entities");
    }

    #[test]
    fn test_parse_self_closing_slash() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        doc.parse_html_fragment(parent, "<img src=\"a.png\" />");

        let img = doc.first_child(parent).unwrap();
        assert_eq!(doc.tag_name(img), Some("img"));
        assert_eq!(doc.get_attribute(img, "src"), Some("a.png"));
    }

    #[test]
    fn test_parse_id_indexed() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        doc.parse_html_fragment(parent, "<div id=\"app\">content</div>");

        assert!(doc.get_element_by_id("app").is_some());
    }
}
