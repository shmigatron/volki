//! RSX Tokenizer — converts RSX source text into a stream of tokens.

use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::path::PathBuf;

use super::CompileError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// `<div` — opening tag name
    OpenTag(String),
    /// `</div>` — closing tag
    CloseTag(String),
    /// `/>`
    SelfCloseEnd,
    /// `>`
    TagEnd,
    /// Attribute name (e.g. `class`, `href`)
    AttrName(String),
    /// `=`
    AttrEquals,
    /// `"value"` — quoted attribute value (without quotes)
    AttrValue(String),
    /// `{expr}` — brace attribute expression (without outer braces)
    AttrExpr(String),
    /// `"hello"` — string literal as a child node
    TextLiteral(String),
    /// `{expr()}` — Rust expression (brace-matched, without outer braces)
    Expression(String),
}

struct Tokenizer<'a> {
    bytes: &'a [u8],
    pos: usize,
    file: PathBuf,
    tokens: Vec<Token>,
    /// Are we inside an opening tag (between `<name` and `>` or `/>`)?
    in_tag: bool,
}

impl<'a> Tokenizer<'a> {
    fn new(source: &'a str, file: PathBuf) -> Self {
        Self {
            bytes: source.as_bytes(),
            pos: 0,
            file,
            tokens: Vec::new(),
            in_tag: false,
        }
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.bytes.len() {
            Some(self.bytes[self.pos])
        } else {
            None
        }
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        let idx = self.pos + offset;
        if idx < self.bytes.len() {
            Some(self.bytes[idx])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<u8> {
        if self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn current_line_col(&self) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for i in 0..self.pos.min(self.bytes.len()) {
            if self.bytes[i] == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    fn error(&self, msg: &str) -> CompileError {
        let (line, col) = self.current_line_col();
        CompileError {
            file: self.file.clone(),
            line,
            col,
            message: String::from(msg),
        }
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let s = unsafe { core::str::from_utf8_unchecked(&self.bytes[start..self.pos]) };
        String::from(s)
    }

    fn read_quoted_string(&mut self) -> Result<String, CompileError> {
        // Skip the opening quote
        self.advance();
        let start = self.pos;
        while self.pos < self.bytes.len() {
            if self.bytes[self.pos] == b'\\' {
                self.pos += 2;
                continue;
            }
            if self.bytes[self.pos] == b'"' {
                let s = unsafe { core::str::from_utf8_unchecked(&self.bytes[start..self.pos]) };
                let result = String::from(s);
                self.pos += 1; // skip closing quote
                return Ok(result);
            }
            self.pos += 1;
        }
        Err(self.error("unterminated string literal"))
    }

    fn read_brace_expression(&mut self) -> Result<String, CompileError> {
        // Skip the opening brace
        self.advance();
        let start = self.pos;
        let mut depth = 1;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'"' => {
                    // Skip string literal
                    self.pos += 1;
                    while self.pos < self.bytes.len() {
                        if self.bytes[self.pos] == b'\\' {
                            self.pos += 2;
                            continue;
                        }
                        if self.bytes[self.pos] == b'"' {
                            self.pos += 1;
                            break;
                        }
                        self.pos += 1;
                    }
                    continue;
                }
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let s = unsafe {
                            core::str::from_utf8_unchecked(&self.bytes[start..self.pos])
                        };
                        let result = String::from(s.trim());
                        self.pos += 1; // skip closing brace
                        return Ok(result);
                    }
                }
                _ => {}
            }
            self.pos += 1;
        }
        Err(self.error("unterminated brace expression"))
    }

    fn tokenize(mut self) -> Result<Vec<Token>, CompileError> {
        while self.pos < self.bytes.len() {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                break;
            }

            let b = self.bytes[self.pos];

            if self.in_tag {
                // Inside an opening tag — parse attributes, >, or />
                match b {
                    b'/' if self.peek_at(1) == Some(b'>') => {
                        self.pos += 2;
                        self.in_tag = false;
                        self.tokens.push(Token::SelfCloseEnd);
                    }
                    b'>' => {
                        self.pos += 1;
                        self.in_tag = false;
                        self.tokens.push(Token::TagEnd);
                    }
                    b'=' => {
                        self.pos += 1;
                        self.tokens.push(Token::AttrEquals);
                    }
                    b'"' => {
                        let val = self.read_quoted_string()?;
                        self.tokens.push(Token::AttrValue(val));
                    }
                    b'{' => {
                        let expr = self.read_brace_expression()?;
                        self.tokens.push(Token::AttrExpr(expr));
                    }
                    _ if b.is_ascii_alphabetic() || b == b'_' => {
                        let name = self.read_ident();
                        self.tokens.push(Token::AttrName(name));
                    }
                    _ => {
                        return Err(self.error("unexpected character in tag"));
                    }
                }
            } else {
                // Outside a tag — parse children, new tags, close tags, text, expressions
                match b {
                    b'<' if self.peek_at(1) == Some(b'/') => {
                        // Close tag: </name>
                        self.pos += 2;
                        let name = self.read_ident();
                        self.skip_whitespace();
                        if self.peek() != Some(b'>') {
                            return Err(self.error("expected '>' in closing tag"));
                        }
                        self.pos += 1;
                        self.tokens.push(Token::CloseTag(name));
                    }
                    b'<' if self.peek_at(1).map_or(false, |c| c.is_ascii_alphabetic() || c == b'_') => {
                        // Open tag: <name
                        self.pos += 1;
                        let name = self.read_ident();
                        self.in_tag = true;
                        self.tokens.push(Token::OpenTag(name));
                    }
                    b'"' => {
                        let text = self.read_quoted_string()?;
                        self.tokens.push(Token::TextLiteral(text));
                    }
                    b'{' => {
                        let expr = self.read_brace_expression()?;
                        self.tokens.push(Token::Expression(expr));
                    }
                    _ => {
                        return Err(self.error("unexpected character in RSX body"));
                    }
                }
            }
        }

        Ok(self.tokens)
    }
}

/// Tokenize an RSX function body into a list of tokens.
pub fn tokenize(source: &str, file: PathBuf) -> Result<Vec<Token>, CompileError> {
    Tokenizer::new(source, file).tokenize()
}

#[cfg(test)]
mod tests {
    use crate::vvec;
    use super::*;

    fn tok(src: &str) -> Vec<Token> {
        tokenize(src, PathBuf::from("<test>")).unwrap()
    }

    #[test]
    fn test_tokenize_simple_element() {
        let tokens = tok(r#"<div class="foo">"hello"</div>"#);
        assert_eq!(tokens, vvec![
            Token::OpenTag(String::from("div")),
            Token::AttrName(String::from("class")),
            Token::AttrEquals,
            Token::AttrValue(String::from("foo")),
            Token::TagEnd,
            Token::TextLiteral(String::from("hello")),
            Token::CloseTag(String::from("div")),
        ]);
    }

    #[test]
    fn test_tokenize_self_closing() {
        let tokens = tok(r#"<br />"#);
        assert_eq!(tokens, vvec![
            Token::OpenTag(String::from("br")),
            Token::SelfCloseEnd,
        ]);
    }

    #[test]
    fn test_tokenize_expression() {
        let tokens = tok(r#"{sidebar_content()}"#);
        assert_eq!(tokens, vvec![
            Token::Expression(String::from("sidebar_content()")),
        ]);
    }

    #[test]
    fn test_tokenize_nested_braces() {
        let tokens = tok(r#"{vec![1, 2, 3]}"#);
        assert_eq!(tokens, vvec![
            Token::Expression(String::from("vec![1, 2, 3]")),
        ]);
    }

    #[test]
    fn test_tokenize_text_literal() {
        let tokens = tok(r#""hello world""#);
        assert_eq!(tokens, vvec![
            Token::TextLiteral(String::from("hello world")),
        ]);
    }

    #[test]
    fn test_tokenize_multiple_attrs() {
        let tokens = tok(r#"<input type="text" name="user" />"#);
        assert_eq!(tokens, vvec![
            Token::OpenTag(String::from("input")),
            Token::AttrName(String::from("type")),
            Token::AttrEquals,
            Token::AttrValue(String::from("text")),
            Token::AttrName(String::from("name")),
            Token::AttrEquals,
            Token::AttrValue(String::from("user")),
            Token::SelfCloseEnd,
        ]);
    }

    #[test]
    fn test_tokenize_attr_expression() {
        let tokens = tok(r#"<button onclick={on_click}>"ok"</button>"#);
        assert_eq!(tokens, vvec![
            Token::OpenTag(String::from("button")),
            Token::AttrName(String::from("onclick")),
            Token::AttrEquals,
            Token::AttrExpr(String::from("on_click")),
            Token::TagEnd,
            Token::TextLiteral(String::from("ok")),
            Token::CloseTag(String::from("button")),
        ]);
    }

    #[test]
    fn test_tokenize_nested_elements() {
        let tokens = tok(r#"<div><span>"hi"</span></div>"#);
        assert_eq!(tokens, vvec![
            Token::OpenTag(String::from("div")),
            Token::TagEnd,
            Token::OpenTag(String::from("span")),
            Token::TagEnd,
            Token::TextLiteral(String::from("hi")),
            Token::CloseTag(String::from("span")),
            Token::CloseTag(String::from("div")),
        ]);
    }

    #[test]
    fn test_tokenize_expression_in_element() {
        let tokens = tok(r#"<div>{content()}</div>"#);
        assert_eq!(tokens, vvec![
            Token::OpenTag(String::from("div")),
            Token::TagEnd,
            Token::Expression(String::from("content()")),
            Token::CloseTag(String::from("div")),
        ]);
    }
}
