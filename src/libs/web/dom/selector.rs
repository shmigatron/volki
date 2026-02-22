//! CSS selector parser and matcher.
//!
//! Supports: tag, .class, #id, [attr], [attr=val], [attr^=val], [attr$=val],
//! [attr*=val], combinators (descendant ` `, child `>`, adjacent `+`, general `~`),
//! pseudo-classes (:first-child, :last-child, :nth-child(an+b), :not()),
//! compound selectors, and comma-separated selector lists.

use super::{Document, NodeId};
use super::node::NodeKind;
use crate::core::volkiwithstds::collections::{String, Vec};

/// A parsed CSS selector (possibly a comma-separated list).
pub struct SelectorList {
    pub selectors: Vec<ComplexSelector>,
}

/// A complex selector: a chain of compound selectors linked by combinators.
/// Stored right-to-left: `[0]` is the key (rightmost) selector.
pub struct ComplexSelector {
    pub parts: Vec<(Combinator, CompoundSelector)>,
}

/// How two compound selectors are related.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Combinator {
    /// The key selector (no combinator, just the rightmost part).
    None,
    /// Descendant (whitespace).
    Descendant,
    /// Direct child (`>`).
    Child,
    /// Adjacent sibling (`+`).
    AdjacentSibling,
    /// General sibling (`~`).
    GeneralSibling,
}

/// A compound selector: a sequence of simple selectors applied to the same element.
pub struct CompoundSelector {
    pub parts: Vec<SimpleSelector>,
}

/// A single simple selector.
pub enum SimpleSelector {
    Universal,
    Tag(String),
    Class(String),
    Id(String),
    Attribute(AttrSelector),
    PseudoClass(PseudoClass),
    Not(CompoundSelector),
}

/// Attribute selector operators.
pub struct AttrSelector {
    pub name: String,
    pub op: Option<AttrOp>,
    pub value: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttrOp {
    Equals,
    StartsWith,
    EndsWith,
    Contains,
}

/// Pseudo-class selectors.
pub enum PseudoClass {
    FirstChild,
    LastChild,
    NthChild(i32, i32), // an+b
}

// ── Parser ──────────────────────────────────────────────────────────────────

/// Parses a CSS selector string into a `SelectorList`.
pub fn parse_selector(input: &str) -> Option<SelectorList> {
    let mut parser = SelectorParser::new(input);
    parser.parse_selector_list()
}

struct SelectorParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> SelectorParser<'a> {
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

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_ident(&mut self) -> Option<String> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            return None;
        }
        Some(String::from(&self.input[start..self.pos]))
    }

    fn parse_selector_list(&mut self) -> Option<SelectorList> {
        let mut selectors = Vec::new();
        selectors.push(self.parse_complex_selector()?);

        loop {
            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.advance();
                self.skip_whitespace();
                selectors.push(self.parse_complex_selector()?);
            } else {
                break;
            }
        }

        Some(SelectorList { selectors })
    }

    fn parse_complex_selector(&mut self) -> Option<ComplexSelector> {
        // Parse left-to-right: collect selectors and combinators separately
        let mut selectors = Vec::new();
        let mut combinators = Vec::new();

        selectors.push(self.parse_compound_selector()?);

        loop {
            let had_whitespace = self.skip_whitespace_check();
            match self.peek() {
                Some('>') => {
                    self.advance();
                    self.skip_whitespace();
                    combinators.push(Combinator::Child);
                    selectors.push(self.parse_compound_selector()?);
                }
                Some('+') => {
                    self.advance();
                    self.skip_whitespace();
                    combinators.push(Combinator::AdjacentSibling);
                    selectors.push(self.parse_compound_selector()?);
                }
                Some('~') => {
                    self.advance();
                    self.skip_whitespace();
                    combinators.push(Combinator::GeneralSibling);
                    selectors.push(self.parse_compound_selector()?);
                }
                Some(c) if had_whitespace && c != ',' && c != ')' => {
                    combinators.push(Combinator::Descendant);
                    selectors.push(self.parse_compound_selector()?);
                }
                _ => break,
            }
        }

        // Build right-to-left: key selector (last in L→R) first with Combinator::None,
        // then each preceding selector gets the combinator that linked it to the next.
        // L→R: selectors[0] --combinators[0]--> selectors[1] --combinators[1]--> selectors[2]
        // R→L: (None, sel[2]), (combinators[1], sel[1]), (combinators[0], sel[0])
        let len = selectors.len();
        let mut parts = Vec::with_capacity(len);

        // Drain selectors in reverse order
        let mut i = len;
        while i > 0 {
            i -= 1;
            // Move selector out by swapping with a dummy
            let sel = core::mem::replace(
                &mut selectors[i],
                CompoundSelector { parts: Vec::new() },
            );
            if i == len - 1 {
                parts.push((Combinator::None, sel));
            } else {
                parts.push((combinators[i], sel));
            }
        }

        Some(ComplexSelector { parts })
    }

    fn skip_whitespace_check(&mut self) -> bool {
        let start = self.pos;
        self.skip_whitespace();
        self.pos > start
    }

    fn parse_compound_selector(&mut self) -> Option<CompoundSelector> {
        let mut parts = Vec::new();

        loop {
            match self.peek() {
                Some('*') => {
                    self.advance();
                    parts.push(SimpleSelector::Universal);
                }
                Some('.') => {
                    self.advance();
                    let ident = self.parse_ident()?;
                    parts.push(SimpleSelector::Class(ident));
                }
                Some('#') => {
                    self.advance();
                    let ident = self.parse_ident()?;
                    parts.push(SimpleSelector::Id(ident));
                }
                Some('[') => {
                    parts.push(self.parse_attr_selector()?);
                }
                Some(':') => {
                    parts.push(self.parse_pseudo_class()?);
                }
                Some(c) if c.is_alphabetic() || c == '-' || c == '_' => {
                    let ident = self.parse_ident()?;
                    parts.push(SimpleSelector::Tag(ident));
                }
                _ => break,
            }
        }

        if parts.is_empty() {
            return None;
        }
        Some(CompoundSelector { parts })
    }

    fn parse_attr_selector(&mut self) -> Option<SimpleSelector> {
        // consume '['
        self.advance();
        self.skip_whitespace();
        let name = self.parse_ident()?;
        self.skip_whitespace();

        let (op, value) = match self.peek() {
            Some(']') => (None, None),
            Some('=') => {
                self.advance();
                let val = self.parse_attr_value()?;
                (Some(AttrOp::Equals), Some(val))
            }
            Some('^') => {
                self.advance();
                if self.peek() != Some('=') { return None; }
                self.advance();
                let val = self.parse_attr_value()?;
                (Some(AttrOp::StartsWith), Some(val))
            }
            Some('$') => {
                self.advance();
                if self.peek() != Some('=') { return None; }
                self.advance();
                let val = self.parse_attr_value()?;
                (Some(AttrOp::EndsWith), Some(val))
            }
            Some('*') => {
                self.advance();
                if self.peek() != Some('=') { return None; }
                self.advance();
                let val = self.parse_attr_value()?;
                (Some(AttrOp::Contains), Some(val))
            }
            _ => return None,
        };

        self.skip_whitespace();
        if self.peek() != Some(']') { return None; }
        self.advance();

        Some(SimpleSelector::Attribute(AttrSelector { name, op, value }))
    }

    fn parse_attr_value(&mut self) -> Option<String> {
        self.skip_whitespace();
        match self.peek() {
            Some('"') | Some('\'') => {
                let quote = self.advance().unwrap();
                let start = self.pos;
                while self.peek() != Some(quote) {
                    if self.peek().is_none() { return None; }
                    self.advance();
                }
                let val = String::from(&self.input[start..self.pos]);
                self.advance(); // closing quote
                Some(val)
            }
            _ => self.parse_ident(),
        }
    }

    fn parse_pseudo_class(&mut self) -> Option<SimpleSelector> {
        self.advance(); // consume ':'
        let name = self.parse_ident()?;

        match name.as_str() {
            "first-child" => Some(SimpleSelector::PseudoClass(PseudoClass::FirstChild)),
            "last-child" => Some(SimpleSelector::PseudoClass(PseudoClass::LastChild)),
            "nth-child" => {
                if self.peek() != Some('(') { return None; }
                self.advance();
                self.skip_whitespace();
                let (a, b) = self.parse_nth_expr()?;
                self.skip_whitespace();
                if self.peek() != Some(')') { return None; }
                self.advance();
                Some(SimpleSelector::PseudoClass(PseudoClass::NthChild(a, b)))
            }
            "not" => {
                if self.peek() != Some('(') { return None; }
                self.advance();
                self.skip_whitespace();
                let inner = self.parse_compound_selector()?;
                self.skip_whitespace();
                if self.peek() != Some(')') { return None; }
                self.advance();
                Some(SimpleSelector::Not(inner))
            }
            _ => None,
        }
    }

    fn parse_nth_expr(&mut self) -> Option<(i32, i32)> {
        self.skip_whitespace();

        // Handle "odd" and "even"
        if self.remaining().starts_with("odd") {
            self.pos += 3;
            return Some((2, 1));
        }
        if self.remaining().starts_with("even") {
            self.pos += 4;
            return Some((2, 0));
        }

        // Parse an+b
        let mut a = 0i32;
        let mut b = 0i32;

        let neg = self.peek() == Some('-');
        if neg || self.peek() == Some('+') {
            self.advance();
        }

        let num = self.parse_number();

        if self.peek() == Some('n') {
            self.advance();
            a = num.unwrap_or(1);
            if neg { a = -a; }

            self.skip_whitespace();
            if self.peek() == Some('+') || self.peek() == Some('-') {
                let b_neg = self.peek() == Some('-');
                self.advance();
                self.skip_whitespace();
                b = self.parse_number().unwrap_or(0);
                if b_neg { b = -b; }
            }
        } else {
            b = num.unwrap_or(0);
            if neg { b = -b; }
        }

        Some((a, b))
    }

    fn parse_number(&mut self) -> Option<i32> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            return None;
        }
        let s = &self.input[start..self.pos];
        let mut val: i32 = 0;
        for c in s.chars() {
            val = val * 10 + (c as i32 - '0' as i32);
        }
        Some(val)
    }
}

// ── Matcher ─────────────────────────────────────────────────────────────────

impl Document {
    /// Tests whether a node matches a selector list.
    pub fn matches_selector(&self, id: NodeId, selector: &SelectorList) -> bool {
        for complex in selector.selectors.iter() {
            if self.matches_complex(id, complex) {
                return true;
            }
        }
        false
    }

    fn matches_complex(&self, id: NodeId, selector: &ComplexSelector) -> bool {
        if selector.parts.is_empty() {
            return false;
        }

        // parts[0] is the key selector (rightmost)
        if !self.matches_compound(id, &selector.parts[0].1) {
            return false;
        }

        let mut current = id;
        let mut i = 1;
        while i < selector.parts.len() {
            let (ref comb, ref compound) = selector.parts[i];
            match comb {
                Combinator::Descendant => {
                    let mut ancestor = self.nodes[current.0].parent;
                    let mut found = false;
                    while let Some(anc) = ancestor {
                        if self.matches_compound(anc, compound) {
                            current = anc;
                            found = true;
                            break;
                        }
                        ancestor = self.nodes[anc.0].parent;
                    }
                    if !found { return false; }
                }
                Combinator::Child => {
                    match self.nodes[current.0].parent {
                        Some(p) if self.matches_compound(p, compound) => {
                            current = p;
                        }
                        _ => return false,
                    }
                }
                Combinator::AdjacentSibling => {
                    match self.nodes[current.0].prev_sibling {
                        Some(s) if self.matches_compound(s, compound) => {
                            current = s;
                        }
                        _ => return false,
                    }
                }
                Combinator::GeneralSibling => {
                    let mut sib = self.nodes[current.0].prev_sibling;
                    let mut found = false;
                    while let Some(s) = sib {
                        if self.matches_compound(s, compound) {
                            current = s;
                            found = true;
                            break;
                        }
                        sib = self.nodes[s.0].prev_sibling;
                    }
                    if !found { return false; }
                }
                Combinator::None => {}
            }
            i += 1;
        }

        true
    }

    fn matches_compound(&self, id: NodeId, selector: &CompoundSelector) -> bool {
        for part in selector.parts.iter() {
            if !self.matches_simple(id, part) {
                return false;
            }
        }
        true
    }

    fn matches_simple(&self, id: NodeId, selector: &SimpleSelector) -> bool {
        match selector {
            SimpleSelector::Universal => true,
            SimpleSelector::Tag(tag) => {
                if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
                    el.tag.as_str() == tag.as_str()
                } else {
                    false
                }
            }
            SimpleSelector::Class(cls) => {
                if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
                    el.class_list.contains(cls)
                } else {
                    false
                }
            }
            SimpleSelector::Id(sid) => {
                if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
                    el.id.as_ref().map(|i| i.as_str()) == Some(sid.as_str())
                } else {
                    false
                }
            }
            SimpleSelector::Attribute(attr) => self.matches_attr(id, attr),
            SimpleSelector::PseudoClass(pc) => self.matches_pseudo(id, pc),
            SimpleSelector::Not(inner) => !self.matches_compound(id, inner),
        }
    }

    fn matches_attr(&self, id: NodeId, attr: &AttrSelector) -> bool {
        if let NodeKind::Element(ref el) = self.nodes[id.0].kind {
            for (name, value) in el.attributes.iter() {
                if name.as_str() != attr.name.as_str() {
                    continue;
                }
                match (&attr.op, &attr.value) {
                    (None, _) => return true, // [attr] — just existence
                    (Some(AttrOp::Equals), Some(v)) => return value.as_str() == v.as_str(),
                    (Some(AttrOp::StartsWith), Some(v)) => return value.as_str().starts_with(v.as_str()),
                    (Some(AttrOp::EndsWith), Some(v)) => return value.as_str().ends_with(v.as_str()),
                    (Some(AttrOp::Contains), Some(v)) => return value.as_str().contains(v.as_str()),
                    _ => return false,
                }
            }
            false
        } else {
            false
        }
    }

    fn matches_pseudo(&self, id: NodeId, pseudo: &PseudoClass) -> bool {
        match pseudo {
            PseudoClass::FirstChild => {
                self.nodes[id.0].prev_sibling.is_none() && self.nodes[id.0].parent.is_some()
            }
            PseudoClass::LastChild => {
                self.nodes[id.0].next_sibling.is_none() && self.nodes[id.0].parent.is_some()
            }
            PseudoClass::NthChild(a, b) => {
                if let Some(parent) = self.nodes[id.0].parent {
                    let mut idx = 1i32; // 1-based
                    let mut sib = self.nodes[parent.0].first_child;
                    while let Some(s) = sib {
                        if s == id {
                            break;
                        }
                        idx += 1;
                        sib = self.nodes[s.0].next_sibling;
                    }
                    if *a == 0 {
                        idx == *b
                    } else {
                        let diff = idx - b;
                        diff % a == 0 && diff / a >= 0
                    }
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Document;

    #[test]
    fn test_parse_tag() {
        let sel = parse_selector("div").unwrap();
        assert_eq!(sel.selectors.len(), 1);
    }

    #[test]
    fn test_parse_class() {
        let sel = parse_selector(".foo").unwrap();
        assert_eq!(sel.selectors.len(), 1);
    }

    #[test]
    fn test_parse_id() {
        let sel = parse_selector("#main").unwrap();
        assert_eq!(sel.selectors.len(), 1);
    }

    #[test]
    fn test_parse_compound() {
        let sel = parse_selector("div.foo#bar").unwrap();
        assert_eq!(sel.selectors[0].parts[0].1.parts.len(), 3);
    }

    #[test]
    fn test_parse_descendant() {
        let sel = parse_selector("div span").unwrap();
        assert_eq!(sel.selectors[0].parts.len(), 2);
    }

    #[test]
    fn test_parse_child() {
        let sel = parse_selector("div > span").unwrap();
        assert_eq!(sel.selectors[0].parts.len(), 2);
        assert_eq!(sel.selectors[0].parts[1].0, Combinator::Child);
    }

    #[test]
    fn test_parse_comma() {
        let sel = parse_selector("div, span").unwrap();
        assert_eq!(sel.selectors.len(), 2);
    }

    #[test]
    fn test_parse_attr() {
        let sel = parse_selector("[data-x]").unwrap();
        assert_eq!(sel.selectors.len(), 1);
    }

    #[test]
    fn test_parse_attr_eq() {
        let sel = parse_selector("[data-x=\"hello\"]").unwrap();
        assert_eq!(sel.selectors.len(), 1);
    }

    #[test]
    fn test_parse_nth_child() {
        let sel = parse_selector(":nth-child(2n+1)").unwrap();
        assert_eq!(sel.selectors.len(), 1);
    }

    #[test]
    fn test_parse_not() {
        let sel = parse_selector(":not(.hidden)").unwrap();
        assert_eq!(sel.selectors.len(), 1);
    }

    #[test]
    fn test_match_tag() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let sel = parse_selector("div").unwrap();
        assert!(doc.matches_selector(div, &sel));

        let span = doc.create_element("span");
        assert!(!doc.matches_selector(span, &sel));
    }

    #[test]
    fn test_match_class() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.class_list_add(div, "foo");
        let sel = parse_selector(".foo").unwrap();
        assert!(doc.matches_selector(div, &sel));
    }

    #[test]
    fn test_match_id() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.set_attribute(div, "id", "main");
        let sel = parse_selector("#main").unwrap();
        assert!(doc.matches_selector(div, &sel));
    }

    #[test]
    fn test_match_descendant() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let span = doc.create_element("span");
        doc.append_child(doc.root, div);
        doc.append_child(div, span);

        let sel = parse_selector("div span").unwrap();
        assert!(doc.matches_selector(span, &sel));
        assert!(!doc.matches_selector(div, &sel));
    }

    #[test]
    fn test_match_child_combinator() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let p = doc.create_element("p");
        let span = doc.create_element("span");
        doc.append_child(doc.root, div);
        doc.append_child(div, p);
        doc.append_child(p, span);

        let sel = parse_selector("div > p").unwrap();
        assert!(doc.matches_selector(p, &sel));

        // span is not a direct child of div
        let sel2 = parse_selector("div > span").unwrap();
        assert!(!doc.matches_selector(span, &sel2));
    }

    #[test]
    fn test_match_first_child() {
        let mut doc = Document::new();
        let ul = doc.create_element("ul");
        let li1 = doc.create_element("li");
        let li2 = doc.create_element("li");
        doc.append_child(ul, li1);
        doc.append_child(ul, li2);

        let sel = parse_selector(":first-child").unwrap();
        assert!(doc.matches_selector(li1, &sel));
        assert!(!doc.matches_selector(li2, &sel));
    }

    #[test]
    fn test_match_not() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.class_list_add(div, "visible");

        let sel = parse_selector(":not(.hidden)").unwrap();
        assert!(doc.matches_selector(div, &sel));

        doc.class_list_add(div, "hidden");
        // Re-parse because class was added
        let sel2 = parse_selector(":not(.hidden)").unwrap();
        assert!(!doc.matches_selector(div, &sel2));
    }
}
