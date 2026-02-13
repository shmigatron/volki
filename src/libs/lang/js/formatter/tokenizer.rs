use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    StringLiteral,
    TemplateLiteral,
    TemplateHead,
    TemplateMiddle,
    TemplateTail,
    NumericLiteral,
    RegexLiteral,
    Identifier,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Semicolon,
    Comma,
    Dot,
    Colon,
    QuestionMark,
    Arrow,
    Spread,
    Operator,
    Assignment,
    LineComment,
    BlockComment,
    Whitespace,
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug)]
pub struct TokenizeError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tokenize error at {}:{}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for TokenizeError {}

struct Tokenizer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    tokens: Vec<Token>,
    template_depth: Vec<usize>,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            tokens: Vec::new(),
            template_depth: Vec::new(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn emit(&mut self, kind: TokenKind, text: String, line: usize, col: usize) {
        self.tokens.push(Token { kind, text, line, col });
    }

    fn last_significant_kind(&self) -> Option<&TokenKind> {
        self.tokens.iter().rev()
            .find(|t| !matches!(t.kind, TokenKind::Whitespace | TokenKind::Newline | TokenKind::LineComment | TokenKind::BlockComment))
            .map(|t| &t.kind)
    }

    fn slash_is_regex(&self) -> bool {
        match self.last_significant_kind() {
            None => true,
            Some(kind) => matches!(kind,
                TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace |
                TokenKind::Comma | TokenKind::Semicolon | TokenKind::Colon |
                TokenKind::Arrow | TokenKind::Operator | TokenKind::Assignment |
                TokenKind::QuestionMark | TokenKind::Spread
            ) || self.last_significant_is_keyword(),
        }
    }

    fn last_significant_is_keyword(&self) -> bool {
        let last = self.tokens.iter().rev()
            .find(|t| !matches!(t.kind, TokenKind::Whitespace | TokenKind::Newline | TokenKind::LineComment | TokenKind::BlockComment));
        match last {
            Some(t) if t.kind == TokenKind::Identifier => {
                matches!(t.text.as_str(),
                    "return" | "typeof" | "instanceof" | "in" | "of" | "new" |
                    "delete" | "void" | "throw" | "case" | "yield" | "await" | "else"
                )
            }
            _ => false,
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, TokenizeError> {
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            let line = self.line;
            let col = self.col;

            match ch {
                '\n' => {
                    self.advance();
                    self.emit(TokenKind::Newline, "\n".into(), line, col);
                }
                '\r' => {
                    self.advance();
                    let text = if self.peek() == Some('\n') {
                        self.advance();
                        "\r\n".into()
                    } else {
                        "\r".into()
                    };
                    self.emit(TokenKind::Newline, text, line, col);
                }
                ' ' | '\t' => {
                    let mut ws = String::new();
                    while let Some(c) = self.peek() {
                        if c == ' ' || c == '\t' {
                            ws.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.emit(TokenKind::Whitespace, ws, line, col);
                }
                '/' => {
                    if self.peek_at(1) == Some('/') {
                        self.read_line_comment(line, col);
                    } else if self.peek_at(1) == Some('*') {
                        self.read_block_comment(line, col);
                    } else if self.slash_is_regex() {
                        self.read_regex(line, col)?;
                    } else {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            self.emit(TokenKind::Assignment, "/=".into(), line, col);
                        } else {
                            self.emit(TokenKind::Operator, "/".into(), line, col);
                        }
                    }
                }
                '\'' | '"' => {
                    self.read_string(line, col)?;
                }
                '`' => {
                    self.read_template(line, col)?;
                }
                '(' => { self.advance(); self.emit(TokenKind::OpenParen, "(".into(), line, col); }
                ')' => { self.advance(); self.emit(TokenKind::CloseParen, ")".into(), line, col); }
                '[' => { self.advance(); self.emit(TokenKind::OpenBracket, "[".into(), line, col); }
                ']' => { self.advance(); self.emit(TokenKind::CloseBracket, "]".into(), line, col); }
                '{' => {
                    self.advance();
                    if let Some(depth) = self.template_depth.last_mut() {
                        *depth += 1;
                    }
                    self.emit(TokenKind::OpenBrace, "{".into(), line, col);
                }
                '}' => {
                    if let Some(depth) = self.template_depth.last_mut() {
                        if *depth == 0 {
                            self.template_depth.pop();
                            self.read_template_continuation(line, col)?;
                            continue;
                        }
                        *depth -= 1;
                    }
                    self.advance();
                    self.emit(TokenKind::CloseBrace, "}".into(), line, col);
                }
                ';' => { self.advance(); self.emit(TokenKind::Semicolon, ";".into(), line, col); }
                ',' => { self.advance(); self.emit(TokenKind::Comma, ",".into(), line, col); }
                '?' => {
                    self.advance();
                    if self.peek() == Some('?') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            self.emit(TokenKind::Assignment, "??=".into(), line, col);
                        } else {
                            self.emit(TokenKind::Operator, "??".into(), line, col);
                        }
                    } else if self.peek() == Some('.') && !self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
                        self.advance();
                        self.emit(TokenKind::Operator, "?.".into(), line, col);
                    } else {
                        self.emit(TokenKind::QuestionMark, "?".into(), line, col);
                    }
                }
                ':' => { self.advance(); self.emit(TokenKind::Colon, ":".into(), line, col); }
                '.' => {
                    if self.peek_at(1) == Some('.') && self.peek_at(2) == Some('.') {
                        self.advance(); self.advance(); self.advance();
                        self.emit(TokenKind::Spread, "...".into(), line, col);
                    } else if self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
                        self.read_number(line, col);
                    } else {
                        self.advance();
                        self.emit(TokenKind::Dot, ".".into(), line, col);
                    }
                }
                '=' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        self.emit(TokenKind::Arrow, "=>".into(), line, col);
                    } else if self.peek() == Some('=') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            self.emit(TokenKind::Operator, "===".into(), line, col);
                        } else {
                            self.emit(TokenKind::Operator, "==".into(), line, col);
                        }
                    } else {
                        self.emit(TokenKind::Assignment, "=".into(), line, col);
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            self.emit(TokenKind::Operator, "!==".into(), line, col);
                        } else {
                            self.emit(TokenKind::Operator, "!=".into(), line, col);
                        }
                    } else {
                        self.emit(TokenKind::Operator, "!".into(), line, col);
                    }
                }
                '<' | '>' => { self.read_comparison(line, col); }
                '+' | '-' => {
                    self.advance();
                    let s = String::from(ch);
                    if self.peek() == Some(ch) {
                        self.advance();
                        let mut doubled = s.clone();
                        doubled.push(ch);
                        self.emit(TokenKind::Operator, doubled, line, col);
                    } else if self.peek() == Some('=') {
                        self.advance();
                        let mut ass = s;
                        ass.push('=');
                        self.emit(TokenKind::Assignment, ass, line, col);
                    } else {
                        self.emit(TokenKind::Operator, s, line, col);
                    }
                }
                '*' => {
                    self.advance();
                    if self.peek() == Some('*') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            self.emit(TokenKind::Assignment, "**=".into(), line, col);
                        } else {
                            self.emit(TokenKind::Operator, "**".into(), line, col);
                        }
                    } else if self.peek() == Some('=') {
                        self.advance();
                        self.emit(TokenKind::Assignment, "*=".into(), line, col);
                    } else {
                        self.emit(TokenKind::Operator, "*".into(), line, col);
                    }
                }
                '%' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        self.emit(TokenKind::Assignment, "%=".into(), line, col);
                    } else {
                        self.emit(TokenKind::Operator, "%".into(), line, col);
                    }
                }
                '&' => {
                    self.advance();
                    if self.peek() == Some('&') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            self.emit(TokenKind::Assignment, "&&=".into(), line, col);
                        } else {
                            self.emit(TokenKind::Operator, "&&".into(), line, col);
                        }
                    } else if self.peek() == Some('=') {
                        self.advance();
                        self.emit(TokenKind::Assignment, "&=".into(), line, col);
                    } else {
                        self.emit(TokenKind::Operator, "&".into(), line, col);
                    }
                }
                '|' => {
                    self.advance();
                    if self.peek() == Some('|') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            self.emit(TokenKind::Assignment, "||=".into(), line, col);
                        } else {
                            self.emit(TokenKind::Operator, "||".into(), line, col);
                        }
                    } else if self.peek() == Some('=') {
                        self.advance();
                        self.emit(TokenKind::Assignment, "|=".into(), line, col);
                    } else {
                        self.emit(TokenKind::Operator, "|".into(), line, col);
                    }
                }
                '^' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        self.emit(TokenKind::Assignment, "^=".into(), line, col);
                    } else {
                        self.emit(TokenKind::Operator, "^".into(), line, col);
                    }
                }
                '~' => { self.advance(); self.emit(TokenKind::Operator, "~".into(), line, col); }
                '#' => {
                    // Private field prefix — treat as part of identifier
                    self.advance();
                    let mut ident = "#".to_string();
                    while let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' || c == '$' {
                            ident.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.emit(TokenKind::Identifier, ident, line, col);
                }
                '@' => {
                    // Decorator — emit as identifier
                    self.advance();
                    let mut ident = "@".to_string();
                    while let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' || c == '$' || c == '.' {
                            ident.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.emit(TokenKind::Identifier, ident, line, col);
                }
                '0'..='9' => { self.read_number(line, col); }
                _ if is_ident_start(ch) => { self.read_identifier(line, col); }
                _ => {
                    self.advance();
                    self.emit(TokenKind::Operator, ch.to_string(), line, col);
                }
            }
        }

        self.emit(TokenKind::Eof, String::new(), self.line, self.col);
        Ok(self.tokens)
    }

    fn read_line_comment(&mut self, line: usize, col: usize) {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            text.push(c);
            self.advance();
        }
        self.emit(TokenKind::LineComment, text, line, col);
    }

    fn read_block_comment(&mut self, line: usize, col: usize) {
        let mut text = String::new();
        text.push(self.advance().unwrap()); // /
        text.push(self.advance().unwrap()); // *
        loop {
            match self.advance() {
                None => break,
                Some('*') if self.peek() == Some('/') => {
                    text.push('*');
                    text.push(self.advance().unwrap());
                    break;
                }
                Some(c) => text.push(c),
            }
        }
        self.emit(TokenKind::BlockComment, text, line, col);
    }

    fn read_string(&mut self, line: usize, col: usize) -> Result<(), TokenizeError> {
        let quote = self.advance().unwrap();
        let mut text = String::new();
        text.push(quote);
        loop {
            match self.advance() {
                None => break,
                Some('\\') => {
                    text.push('\\');
                    if let Some(c) = self.advance() {
                        text.push(c);
                    }
                }
                Some(c) if c == quote => {
                    text.push(c);
                    break;
                }
                Some(c) if c == '\n' || c == '\r' => {
                    text.push(c);
                    break;
                }
                Some(c) => text.push(c),
            }
        }
        self.emit(TokenKind::StringLiteral, text, line, col);
        Ok(())
    }

    fn read_template(&mut self, line: usize, col: usize) -> Result<(), TokenizeError> {
        self.advance(); // consume `
        let mut text = String::from('`');
        loop {
            match self.advance() {
                None => {
                    self.emit(TokenKind::TemplateLiteral, text, line, col);
                    return Ok(());
                }
                Some('\\') => {
                    text.push('\\');
                    if let Some(c) = self.advance() {
                        text.push(c);
                    }
                }
                Some('`') => {
                    text.push('`');
                    self.emit(TokenKind::TemplateLiteral, text, line, col);
                    return Ok(());
                }
                Some('$') if self.peek() == Some('{') => {
                    text.push('$');
                    text.push(self.advance().unwrap()); // {
                    self.template_depth.push(0);
                    self.emit(TokenKind::TemplateHead, text, line, col);
                    return Ok(());
                }
                Some(c) => text.push(c),
            }
        }
    }

    fn read_template_continuation(&mut self, line: usize, col: usize) -> Result<(), TokenizeError> {
        self.advance(); // consume }
        let mut text = String::from('}');
        loop {
            match self.advance() {
                None => {
                    self.emit(TokenKind::TemplateTail, text, line, col);
                    return Ok(());
                }
                Some('\\') => {
                    text.push('\\');
                    if let Some(c) = self.advance() {
                        text.push(c);
                    }
                }
                Some('`') => {
                    text.push('`');
                    self.emit(TokenKind::TemplateTail, text, line, col);
                    return Ok(());
                }
                Some('$') if self.peek() == Some('{') => {
                    text.push('$');
                    text.push(self.advance().unwrap());
                    self.template_depth.push(0);
                    self.emit(TokenKind::TemplateMiddle, text, line, col);
                    return Ok(());
                }
                Some(c) => text.push(c),
            }
        }
    }

    fn read_regex(&mut self, line: usize, col: usize) -> Result<(), TokenizeError> {
        let mut text = String::new();
        text.push(self.advance().unwrap()); // /
        let mut in_class = false;
        loop {
            match self.advance() {
                None => break,
                Some('\\') => {
                    text.push('\\');
                    if let Some(c) = self.advance() {
                        text.push(c);
                    }
                }
                Some('[') => {
                    in_class = true;
                    text.push('[');
                }
                Some(']') if in_class => {
                    in_class = false;
                    text.push(']');
                }
                Some('/') if !in_class => {
                    text.push('/');
                    // Read flags
                    while let Some(c) = self.peek() {
                        if c.is_ascii_alphabetic() {
                            text.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    break;
                }
                Some(c) if c == '\n' || c == '\r' => {
                    // Unterminated regex — bail, emit what we have
                    break;
                }
                Some(c) => text.push(c),
            }
        }
        self.emit(TokenKind::RegexLiteral, text, line, col);
        Ok(())
    }

    fn read_number(&mut self, line: usize, col: usize) {
        let mut text = String::new();
        if self.peek() == Some('0') {
            text.push(self.advance().unwrap());
            match self.peek() {
                Some('x') | Some('X') => {
                    text.push(self.advance().unwrap());
                    self.read_digits(&mut text, |c| c.is_ascii_hexdigit());
                    self.maybe_read_bigint(&mut text);
                    self.emit(TokenKind::NumericLiteral, text, line, col);
                    return;
                }
                Some('o') | Some('O') => {
                    text.push(self.advance().unwrap());
                    self.read_digits(&mut text, |c| ('0'..='7').contains(&c));
                    self.maybe_read_bigint(&mut text);
                    self.emit(TokenKind::NumericLiteral, text, line, col);
                    return;
                }
                Some('b') | Some('B') => {
                    text.push(self.advance().unwrap());
                    self.read_digits(&mut text, |c| c == '0' || c == '1');
                    self.maybe_read_bigint(&mut text);
                    self.emit(TokenKind::NumericLiteral, text, line, col);
                    return;
                }
                _ => {}
            }
        }
        self.read_digits(&mut text, |c| c.is_ascii_digit());
        if self.peek() == Some('.') && self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
            text.push(self.advance().unwrap());
            self.read_digits(&mut text, |c| c.is_ascii_digit());
        } else if self.peek() == Some('.') && !self.peek_at(1).is_some_and(|c| c == '.') {
            // Trailing dot like `1.`
            text.push(self.advance().unwrap());
        }
        if let Some('e') | Some('E') = self.peek() {
            text.push(self.advance().unwrap());
            if let Some('+') | Some('-') = self.peek() {
                text.push(self.advance().unwrap());
            }
            self.read_digits(&mut text, |c| c.is_ascii_digit());
        }
        self.maybe_read_bigint(&mut text);
        self.emit(TokenKind::NumericLiteral, text, line, col);
    }

    fn read_digits(&mut self, text: &mut String, predicate: fn(char) -> bool) {
        while let Some(c) = self.peek() {
            if predicate(c) || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
    }

    fn maybe_read_bigint(&mut self, text: &mut String) {
        if self.peek() == Some('n') {
            text.push(self.advance().unwrap());
        }
    }

    fn read_identifier(&mut self, line: usize, col: usize) {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if is_ident_continue(c) {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        self.emit(TokenKind::Identifier, text, line, col);
    }

    fn read_comparison(&mut self, line: usize, col: usize) {
        let ch = self.advance().unwrap();
        let mut text = String::from(ch);
        if self.peek() == Some(ch) {
            text.push(self.advance().unwrap());
            if ch == '>' && self.peek() == Some('>') {
                text.push(self.advance().unwrap()); // >>>
            }
            if self.peek() == Some('=') {
                text.push(self.advance().unwrap());
                self.emit(TokenKind::Assignment, text, line, col);
            } else {
                self.emit(TokenKind::Operator, text, line, col);
            }
        } else if self.peek() == Some('=') {
            text.push(self.advance().unwrap());
            self.emit(TokenKind::Operator, text, line, col);
        } else {
            self.emit(TokenKind::Operator, text, line, col);
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$' || ch > '\x7f'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || ch > '\x7f'
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError> {
    Tokenizer::new(input).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Vec<TokenKind> {
        tokenize(input).unwrap().into_iter().map(|t| t.kind).collect()
    }

    fn texts(input: &str) -> Vec<String> {
        tokenize(input).unwrap().into_iter().map(|t| t.text).collect()
    }

    #[test]
    fn simple_identifiers() {
        let k = kinds("foo bar");
        assert_eq!(k, vec![
            TokenKind::Identifier, TokenKind::Whitespace, TokenKind::Identifier, TokenKind::Eof
        ]);
    }

    #[test]
    fn string_literals() {
        let t = texts(r#""hello" 'world'"#);
        assert_eq!(t[0], "\"hello\"");
        assert_eq!(t[2], "'world'");
    }

    #[test]
    fn template_literal_no_interpolation() {
        let k = kinds("`hello`");
        assert_eq!(k, vec![TokenKind::TemplateLiteral, TokenKind::Eof]);
    }

    #[test]
    fn template_with_interpolation() {
        let k = kinds("`a${x}b`");
        assert_eq!(k, vec![
            TokenKind::TemplateHead,
            TokenKind::Identifier,
            TokenKind::TemplateTail,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn regex_after_equals() {
        let k = kinds("x = /foo/g");
        assert!(k.contains(&TokenKind::RegexLiteral));
    }

    #[test]
    fn division_after_identifier() {
        let k = kinds("a / b");
        assert!(!k.contains(&TokenKind::RegexLiteral));
        assert_eq!(k[2], TokenKind::Operator);
    }

    #[test]
    fn regex_after_return() {
        let k = kinds("return /foo/");
        assert!(k.contains(&TokenKind::RegexLiteral));
    }

    #[test]
    fn arrow_function() {
        let k = kinds("(x) => x");
        assert!(k.contains(&TokenKind::Arrow));
    }

    #[test]
    fn spread_operator() {
        let k = kinds("...args");
        assert_eq!(k[0], TokenKind::Spread);
    }

    #[test]
    fn numeric_literals() {
        let t = texts("42 0xFF 0b101 0o77 3.14 1e10 100n");
        assert_eq!(t[0], "42");
        assert_eq!(t[2], "0xFF");
        assert_eq!(t[4], "0b101");
        assert_eq!(t[6], "0o77");
        assert_eq!(t[8], "3.14");
        assert_eq!(t[10], "1e10");
        assert_eq!(t[12], "100n");
    }

    #[test]
    fn line_and_block_comments() {
        let k = kinds("// line\n/* block */");
        assert_eq!(k[0], TokenKind::LineComment);
        assert_eq!(k[1], TokenKind::Newline);
        assert_eq!(k[2], TokenKind::BlockComment);
    }

    #[test]
    fn comparison_operators() {
        let t = texts("< > <= >= << >> >>>");
        assert!(t.contains(&"<<<".to_string()) == false);
        assert!(t.contains(&">>>".to_string()));
    }

    #[test]
    fn assignment_operators() {
        let k = kinds("x += 1");
        assert_eq!(k[2], TokenKind::Assignment);
    }

    #[test]
    fn optional_chaining() {
        let t = texts("a?.b");
        assert_eq!(t[0], "a");
        assert_eq!(t[1], "?.");
        assert_eq!(t[2], "b");
    }

    #[test]
    fn nullish_coalescing() {
        let t = texts("a ?? b");
        assert_eq!(t[0], "a");
        assert_eq!(t[2], "??");
        assert_eq!(t[4], "b");
    }

    #[test]
    fn private_field() {
        let t = texts("this.#foo");
        assert_eq!(t[2], "#foo");
    }

    #[test]
    fn string_with_escapes() {
        let t = texts(r#""he\"llo""#);
        assert_eq!(t[0], r#""he\"llo""#);
    }

    #[test]
    fn empty_input() {
        let k = kinds("");
        assert_eq!(k, vec![TokenKind::Eof]);
    }

    #[test]
    fn newlines_tracked() {
        let tokens = tokenize("a\nb").unwrap();
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[2].line, 2);
    }
}
