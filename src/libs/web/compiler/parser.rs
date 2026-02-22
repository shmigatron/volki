//! RSX Parser — recursive descent parser producing an AST from tokens.

use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::path::PathBuf;

use super::tokenizer::Token;
use super::CompileError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RsxAttrValue {
    Literal(String),
    Expr(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RsxAttr {
    pub name: String,
    pub value: RsxAttrValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RsxNode {
    Element {
        tag: String,
        attrs: Vec<RsxAttr>,
        children: Vec<RsxNode>,
        self_closing: bool,
    },
    Text(String),
    Expr(String),
    CondAnd {
        condition: String,
        body: Vec<RsxNode>,
    },
    Ternary {
        condition: String,
        if_true: Vec<RsxNode>,
        if_false: Vec<RsxNode>,
    },
}

/// Skip whitespace bytes starting at `pos`, return first non-whitespace position.
fn skip_ws(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() {
        match bytes[pos] {
            b' ' | b'\t' | b'\n' | b'\r' => pos += 1,
            _ => break,
        }
    }
    pos
}

/// Check if position starts JSX content: `<` + alpha, or `"`.
fn is_jsx_start(bytes: &[u8], pos: usize) -> bool {
    if pos >= bytes.len() {
        return false;
    }
    match bytes[pos] {
        b'"' => true,
        b'<' => pos + 1 < bytes.len() && bytes[pos + 1].is_ascii_alphabetic(),
        _ => false,
    }
}

/// Find first `&&` at depth 0 where what follows (after whitespace) is JSX start.
/// Returns the byte position of the first `&`.
fn find_cond_and_split(bytes: &[u8], len: usize) -> Option<usize> {
    let mut i = 0;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;

    while i < len {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' { i += 2; continue; }
                    if bytes[i] == b'"' { i += 1; break; }
                    i += 1;
                }
                continue;
            }
            b'{' => { brace_depth += 1; }
            b'}' => { brace_depth -= 1; }
            b'(' => { paren_depth += 1; }
            b')' => { paren_depth -= 1; }
            b'&' if brace_depth == 0 && paren_depth == 0
                && i + 1 < len && bytes[i + 1] == b'&' =>
            {
                let after = skip_ws(bytes, i + 2);
                if is_jsx_start(bytes, after) {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find first top-level `&&` regardless of what follows.
fn find_top_level_and(bytes: &[u8], len: usize) -> Option<usize> {
    let mut i = 0;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;

    while i < len {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' { i += 2; continue; }
                    if bytes[i] == b'"' { i += 1; break; }
                    i += 1;
                }
                continue;
            }
            b'{' => { brace_depth += 1; }
            b'}' => { brace_depth -= 1; }
            b'(' => { paren_depth += 1; }
            b')' => { paren_depth -= 1; }
            b'&' if brace_depth == 0 && paren_depth == 0
                && i + 1 < len && bytes[i + 1] == b'&' =>
            {
                return Some(i);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find `?` at depth 0 followed (after whitespace) by JSX start.
/// Returns the byte position of the `?`.
fn find_ternary_question(bytes: &[u8], len: usize) -> Option<usize> {
    let mut i = 0;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;

    while i < len {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' { i += 2; continue; }
                    if bytes[i] == b'"' { i += 1; break; }
                    i += 1;
                }
                continue;
            }
            b'{' => { brace_depth += 1; }
            b'}' => { brace_depth -= 1; }
            b'(' => { paren_depth += 1; }
            b')' => { paren_depth -= 1; }
            b'?' if brace_depth == 0 && paren_depth == 0 => {
                let after = skip_ws(bytes, i + 1);
                if is_jsx_start(bytes, after) {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find first top-level `?` regardless of what follows.
fn find_top_level_question(bytes: &[u8], len: usize) -> Option<usize> {
    let mut i = 0;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;

    while i < len {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' { i += 2; continue; }
                    if bytes[i] == b'"' { i += 1; break; }
                    i += 1;
                }
                continue;
            }
            b'{' => { brace_depth += 1; }
            b'}' => { brace_depth -= 1; }
            b'(' => { paren_depth += 1; }
            b')' => { paren_depth -= 1; }
            b'?' if brace_depth == 0 && paren_depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find `:` separator between ternary true/false branches.
/// Tracks JSX tag nesting, brace depth, and string literals.
/// Returns byte position of `:` when found at all-depths-zero after content.
fn find_ternary_colon(bytes: &[u8], len: usize) -> Option<usize> {
    let mut i = 0;
    let mut tag_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut seen_content = false;

    while i < len {
        match bytes[i] {
            b'"' => {
                seen_content = true;
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' { i += 2; continue; }
                    if bytes[i] == b'"' { i += 1; break; }
                    i += 1;
                }
                continue;
            }
            b'{' => { brace_depth += 1; }
            b'}' => { brace_depth -= 1; }
            b'<' if brace_depth == 0 => {
                seen_content = true;
                if i + 1 < len && bytes[i + 1] == b'/' {
                    // Close tag </name>
                    i += 2;
                    while i < len && bytes[i] != b'>' { i += 1; }
                    if i < len { i += 1; }
                    tag_depth -= 1;
                    continue;
                } else if i + 1 < len && bytes[i + 1].is_ascii_alphabetic() {
                    // Open tag <name...> or <name.../>
                    i += 1;
                    while i < len && (bytes[i].is_ascii_alphanumeric()
                        || bytes[i] == b'_' || bytes[i] == b'-')
                    {
                        i += 1;
                    }
                    // Scan attributes until > or />
                    let mut self_closing = false;
                    while i < len {
                        if bytes[i] == b'"' {
                            i += 1;
                            while i < len {
                                if bytes[i] == b'\\' { i += 2; continue; }
                                if bytes[i] == b'"' { i += 1; break; }
                                i += 1;
                            }
                            continue;
                        }
                        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'>' {
                            self_closing = true;
                            i += 2;
                            break;
                        }
                        if bytes[i] == b'>' {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                    if !self_closing {
                        tag_depth += 1;
                    }
                    continue;
                }
            }
            b':' if brace_depth == 0 && tag_depth == 0 && seen_content => {
                return Some(i);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    file: PathBuf,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token], file: PathBuf) -> Self {
        Self {
            tokens,
            pos: 0,
            file,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    #[allow(dead_code)]
    fn expect_advance(&mut self, msg: &str) -> Result<&Token, CompileError> {
        if self.pos < self.tokens.len() {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            Ok(tok)
        } else {
            Err(self.error(msg))
        }
    }

    fn error(&self, msg: &str) -> CompileError {
        CompileError {
            file: self.file.clone(),
            line: 0,
            col: 0,
            message: String::from(msg),
        }
    }

    /// Parse all top-level nodes.
    fn parse_nodes(&mut self) -> Result<Vec<RsxNode>, CompileError> {
        let mut nodes = Vec::new();
        while self.pos < self.tokens.len() {
            // Stop if we see a CloseTag (means we're inside a parent element)
            if let Some(Token::CloseTag(_)) = self.peek() {
                break;
            }
            nodes.push(self.parse_node()?);
        }
        Ok(nodes)
    }

    /// Parse a single node.
    fn parse_node(&mut self) -> Result<RsxNode, CompileError> {
        match self.peek() {
            Some(Token::OpenTag(_)) => self.parse_element(),
            Some(Token::TextLiteral(_)) => {
                if let Some(Token::TextLiteral(s)) = self.advance() {
                    Ok(RsxNode::Text(s.clone()))
                } else {
                    Err(self.error("expected text literal"))
                }
            }
            Some(Token::Expression(_)) => {
                let expr = if let Some(Token::Expression(s)) = self.advance() {
                    s.clone()
                } else {
                    return Err(self.error("expected expression"));
                };
                self.parse_expression(expr)
            }
            Some(_) => Err(self.error("unexpected token")),
            None => Err(self.error("unexpected end of tokens")),
        }
    }

    /// Analyze an expression string for `?:` ternary or `&&` conditional patterns.
    /// Falls back to plain `Expr` if no JSX-style conditional is detected.
    fn parse_expression(&self, expr: String) -> Result<RsxNode, CompileError> {
        let bytes = expr.as_str().as_bytes();
        let len = bytes.len();

        // Try ternary first (lower precedence — binds outermost)
        if let Some(q_pos) = find_ternary_question(bytes, len) {
            let rest = &expr.as_str()[q_pos + 1..];
            let rest_bytes = rest.as_bytes();
            let Some(c_pos) = find_ternary_colon(rest_bytes, rest_bytes.len()) else {
                return Err(self.error("invalid ternary expression: expected `:`"));
            };
            let condition = expr.as_str()[..q_pos].trim();
            let true_str = rest[..c_pos].trim();
            let false_str = rest[c_pos + 1..].trim();
            if condition.is_empty() || true_str.is_empty() || false_str.is_empty() {
                return Err(self.error("invalid ternary expression: expected `cond ? a : b`"));
            }
            let if_true = self.parse_inline_rsx(true_str)?;
            let if_false = self.parse_inline_rsx(false_str)?;
            return Ok(RsxNode::Ternary {
                condition: String::from(condition),
                if_true,
                if_false,
            });
        }

        // Try && conditional
        if let Some(split_pos) = find_cond_and_split(bytes, len) {
            let condition = expr.as_str()[..split_pos].trim();
            let jsx_str = expr.as_str()[split_pos + 2..].trim();
            if condition.is_empty() || jsx_str.is_empty() {
                return Err(self.error("invalid conditional expression: expected `cond && <rsx>`"));
            }
            let body = self.parse_inline_rsx(jsx_str)?;
            return Ok(RsxNode::CondAnd {
                condition: String::from(condition),
                body,
            });
        }

        // Catch obvious malformed conditionals early.
        if let Some(q_pos) = find_top_level_question(bytes, len) {
            let condition = expr.as_str()[..q_pos].trim();
            if condition.is_empty() {
                return Err(self.error("invalid ternary expression: missing condition before `?`"));
            }
        }
        if let Some(and_pos) = find_top_level_and(bytes, len) {
            let condition = expr.as_str()[..and_pos].trim();
            let rhs = expr.as_str()[and_pos + 2..].trim();
            if condition.is_empty() || rhs.is_empty() {
                return Err(self.error("invalid conditional expression: expected `cond && <expr>`"));
            }
        }

        // Plain expression
        Ok(RsxNode::Expr(expr))
    }

    /// Tokenize and parse a JSX string slice, enabling recursive handling
    /// of nested conditionals inside branches.
    fn parse_inline_rsx(&self, src: &str) -> Result<Vec<RsxNode>, CompileError> {
        let tokens = super::tokenizer::tokenize(src, self.file.clone())?;
        let mut sub_parser = Parser::new(&tokens, self.file.clone());
        sub_parser.parse_nodes()
    }

    /// Parse an element: `<tag attrs...>children...</tag>` or `<tag attrs... />`
    fn parse_element(&mut self) -> Result<RsxNode, CompileError> {
        // Consume OpenTag
        let tag = match self.advance() {
            Some(Token::OpenTag(name)) => name.clone(),
            _ => return Err(self.error("expected opening tag")),
        };

        // Parse attributes
        let mut attrs = Vec::new();
        loop {
            match self.peek() {
                Some(Token::AttrName(_)) => {
                    let name = match self.advance() {
                        Some(Token::AttrName(n)) => n.clone(),
                        _ => return Err(self.error("expected attribute name")),
                    };
                    // Expect =
                    match self.advance() {
                        Some(Token::AttrEquals) => {}
                        _ => return Err(self.error("expected '=' after attribute name")),
                    }
                    // Expect value
                    let value = match self.advance() {
                        Some(Token::AttrValue(v)) => RsxAttrValue::Literal(v.clone()),
                        Some(Token::AttrExpr(v)) => RsxAttrValue::Expr(v.clone()),
                        _ => return Err(self.error("expected attribute value")),
                    };
                    attrs.push(RsxAttr { name, value });
                }
                Some(Token::SelfCloseEnd) | Some(Token::TagEnd) => break,
                _ => return Err(self.error("unexpected token in tag attributes")),
            }
        }

        // Check for self-closing or end of opening tag
        match self.advance() {
            Some(Token::SelfCloseEnd) => {
                return Ok(RsxNode::Element {
                    tag,
                    attrs,
                    children: Vec::new(),
                    self_closing: true,
                });
            }
            Some(Token::TagEnd) => {
                // Parse children until matching close tag
            }
            _ => return Err(self.error("expected '>' or '/>' after tag attributes")),
        }

        // Parse children
        let children = self.parse_nodes()?;

        // Expect closing tag
        match self.advance() {
            Some(Token::CloseTag(close_name)) => {
                if close_name.as_str() != tag.as_str() {
                    return Err(self.error("mismatched closing tag"));
                }
            }
            _ => return Err(self.error("expected closing tag")),
        }

        Ok(RsxNode::Element {
            tag,
            attrs,
            children,
            self_closing: false,
        })
    }
}

/// Parse a token stream into a list of RSX AST nodes.
pub fn parse(tokens: &[Token], file: PathBuf) -> Result<Vec<RsxNode>, CompileError> {
    let mut parser = Parser::new(tokens, file);
    parser.parse_nodes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::web::compiler::tokenizer;

    fn parse_rsx(src: &str) -> Vec<RsxNode> {
        let file = PathBuf::from("<test>");
        let tokens = tokenizer::tokenize(src, file.clone()).unwrap();
        parse(&tokens, file).unwrap()
    }

    #[test]
    fn test_parse_element_with_children() {
        let nodes = parse_rsx(r#"<div class="outer"><span>"inner"</span></div>"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { tag, attrs, children, self_closing } => {
                assert_eq!(tag.as_str(), "div");
                assert_eq!(attrs.len(), 1);
                assert_eq!(attrs[0].name.as_str(), "class");
                assert_eq!(attrs[0].value, RsxAttrValue::Literal(String::from("outer")));
                assert!(!self_closing);
                assert_eq!(children.len(), 1);
                match &children[0] {
                    RsxNode::Element { tag, children, .. } => {
                        assert_eq!(tag.as_str(), "span");
                        assert_eq!(children.len(), 1);
                        assert_eq!(children[0], RsxNode::Text(String::from("inner")));
                    }
                    _ => panic!("expected element"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_self_closing() {
        let nodes = parse_rsx(r#"<input type="text" />"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { tag, attrs, self_closing, .. } => {
                assert_eq!(tag.as_str(), "input");
                assert_eq!(attrs.len(), 1);
                assert_eq!(attrs[0].name.as_str(), "type");
                assert_eq!(attrs[0].value, RsxAttrValue::Literal(String::from("text")));
                assert!(self_closing);
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_attribute_expr_value() {
        let nodes = parse_rsx(r#"<button onclick={on_click}>"go"</button>"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { attrs, .. } => {
                assert_eq!(attrs.len(), 1);
                assert_eq!(attrs[0].name.as_str(), "onclick");
                assert_eq!(attrs[0].value, RsxAttrValue::Expr(String::from("on_click")));
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_mixed_children() {
        let nodes = parse_rsx(r#"<div>"hello"{content()}<span>"world"</span></div>"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { children, .. } => {
                assert_eq!(children.len(), 3);
                assert_eq!(children[0], RsxNode::Text(String::from("hello")));
                assert_eq!(children[1], RsxNode::Expr(String::from("content()")));
                match &children[2] {
                    RsxNode::Element { tag, .. } => assert_eq!(tag.as_str(), "span"),
                    _ => panic!("expected element"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_multiple_top_level() {
        let nodes = parse_rsx(r#"<div>"one"</div><span>"two"</span>"#);
        assert_eq!(nodes.len(), 2);
        match &nodes[0] {
            RsxNode::Element { tag, .. } => assert_eq!(tag.as_str(), "div"),
            _ => panic!("expected div"),
        }
        match &nodes[1] {
            RsxNode::Element { tag, .. } => assert_eq!(tag.as_str(), "span"),
            _ => panic!("expected span"),
        }
    }

    #[test]
    fn test_parse_expression_child() {
        let nodes = parse_rsx(r#"<div>{sidebar_content()}</div>"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(children[0], RsxNode::Expr(String::from("sidebar_content()")));
            }
            _ => panic!("expected element"),
        }
    }

    // ── Conditional rendering tests ──

    #[test]
    fn test_parse_cond_and_element() {
        let nodes = parse_rsx(r#"<div>{is_admin && <span>"Admin"</span>}</div>"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    RsxNode::CondAnd { condition, body } => {
                        assert_eq!(condition.as_str(), "is_admin");
                        assert_eq!(body.len(), 1);
                        match &body[0] {
                            RsxNode::Element { tag, children, .. } => {
                                assert_eq!(tag.as_str(), "span");
                                assert_eq!(children.len(), 1);
                                assert_eq!(children[0], RsxNode::Text(String::from("Admin")));
                            }
                            _ => panic!("expected element in body"),
                        }
                    }
                    _ => panic!("expected CondAnd"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_cond_and_text() {
        let nodes = parse_rsx(r#"<div>{show && "visible text"}</div>"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    RsxNode::CondAnd { condition, body } => {
                        assert_eq!(condition.as_str(), "show");
                        assert_eq!(body.len(), 1);
                        assert_eq!(body[0], RsxNode::Text(String::from("visible text")));
                    }
                    _ => panic!("expected CondAnd"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_ternary() {
        let nodes = parse_rsx(
            r#"<div>{flag ? <span>"yes"</span> : <span>"no"</span>}</div>"#,
        );
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    RsxNode::Ternary { condition, if_true, if_false } => {
                        assert_eq!(condition.as_str(), "flag");
                        assert_eq!(if_true.len(), 1);
                        assert_eq!(if_false.len(), 1);
                        match &if_true[0] {
                            RsxNode::Element { tag, children, .. } => {
                                assert_eq!(tag.as_str(), "span");
                                assert_eq!(children[0], RsxNode::Text(String::from("yes")));
                            }
                            _ => panic!("expected element in if_true"),
                        }
                        match &if_false[0] {
                            RsxNode::Element { tag, children, .. } => {
                                assert_eq!(tag.as_str(), "span");
                                assert_eq!(children[0], RsxNode::Text(String::from("no")));
                            }
                            _ => panic!("expected element in if_false"),
                        }
                    }
                    _ => panic!("expected Ternary"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_complex_condition_and() {
        // Multiple && where only the last one precedes JSX
        let nodes = parse_rsx(r#"<div>{x > 0 && items.len() > 0 && <br />}</div>"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    RsxNode::CondAnd { condition, body } => {
                        assert_eq!(condition.as_str(), "x > 0 && items.len() > 0");
                        assert_eq!(body.len(), 1);
                        match &body[0] {
                            RsxNode::Element { tag, self_closing, .. } => {
                                assert_eq!(tag.as_str(), "br");
                                assert!(self_closing);
                            }
                            _ => panic!("expected self-closing element"),
                        }
                    }
                    _ => panic!("expected CondAnd"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_non_jsx_and_stays_expr() {
        // {a && b} with no JSX → stays as Expr
        let nodes = parse_rsx(r#"<div>{a && b}</div>"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(children[0], RsxNode::Expr(String::from("a && b")));
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_plain_expr_unchanged() {
        let nodes = parse_rsx(r#"<div>{sidebar_content()}</div>"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(children[0], RsxNode::Expr(String::from("sidebar_content()")));
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_nested_conditional() {
        // {a && <div>{b ? <x /> : <y />}</div>}
        let nodes = parse_rsx(
            r#"<div>{a && <div>{b ? <x /> : <y />}</div>}</div>"#,
        );
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Element { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    RsxNode::CondAnd { condition, body } => {
                        assert_eq!(condition.as_str(), "a");
                        assert_eq!(body.len(), 1);
                        match &body[0] {
                            RsxNode::Element { tag, children, .. } => {
                                assert_eq!(tag.as_str(), "div");
                                assert_eq!(children.len(), 1);
                                match &children[0] {
                                    RsxNode::Ternary { condition, if_true, if_false } => {
                                        assert_eq!(condition.as_str(), "b");
                                        assert_eq!(if_true.len(), 1);
                                        assert_eq!(if_false.len(), 1);
                                    }
                                    _ => panic!("expected Ternary inside nested div"),
                                }
                            }
                            _ => panic!("expected div element in CondAnd body"),
                        }
                    }
                    _ => panic!("expected CondAnd"),
                }
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn test_parse_ternary_self_closing() {
        let nodes = parse_rsx(r#"{active ? <div /> : <span />}"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Ternary { condition, if_true, if_false } => {
                assert_eq!(condition.as_str(), "active");
                assert_eq!(if_true.len(), 1);
                assert_eq!(if_false.len(), 1);
                match &if_true[0] {
                    RsxNode::Element { tag, self_closing, .. } => {
                        assert_eq!(tag.as_str(), "div");
                        assert!(self_closing);
                    }
                    _ => panic!("expected div"),
                }
                match &if_false[0] {
                    RsxNode::Element { tag, self_closing, .. } => {
                        assert_eq!(tag.as_str(), "span");
                        assert!(self_closing);
                    }
                    _ => panic!("expected span"),
                }
            }
            _ => panic!("expected Ternary"),
        }
    }

    #[test]
    fn test_parse_ternary_text_branches() {
        let nodes = parse_rsx(r#"{ok ? "yes" : "no"}"#);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            RsxNode::Ternary { condition, if_true, if_false } => {
                assert_eq!(condition.as_str(), "ok");
                assert_eq!(if_true.len(), 1);
                assert_eq!(if_false.len(), 1);
                assert_eq!(if_true[0], RsxNode::Text(String::from("yes")));
                assert_eq!(if_false[0], RsxNode::Text(String::from("no")));
            }
            _ => panic!("expected Ternary"),
        }
    }

    #[test]
    fn test_parse_invalid_ternary_missing_colon_errors() {
        let file = PathBuf::from("<test>");
        let tokens = tokenizer::tokenize(r#"{ok ? <div />}"#, file.clone()).unwrap();
        let result = parse(&tokens, file);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.as_str().contains("ternary"));
    }

    #[test]
    fn test_parse_invalid_conditional_and_errors() {
        let file = PathBuf::from("<test>");
        let tokens = tokenizer::tokenize(r#"{flag &&}"#, file.clone()).unwrap();
        let result = parse(&tokens, file);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.as_str().contains("conditional"));
    }
}
