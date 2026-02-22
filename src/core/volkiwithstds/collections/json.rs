use crate::core::volkiwithstds::collections::{HashMap, String, Vec};

#[derive(Debug, Clone)]
pub enum JsonValue {
    Str(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
    Null,
    Other,
}

impl PartialEq for JsonValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonValue::Str(a), JsonValue::Str(b)) => a == b,
            (JsonValue::Array(a), JsonValue::Array(b)) => a == b,
            (JsonValue::Object(a), JsonValue::Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (k, v) in a.iter() {
                    match b.get(k) {
                        Some(other_v) if v == other_v => {}
                        _ => return false,
                    }
                }
                true
            }
            (JsonValue::Null, JsonValue::Null) => true,
            (JsonValue::Other, JsonValue::Other) => true,
            _ => false,
        }
    }
}

impl Eq for JsonValue {}

impl JsonValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(o) => Some(o),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Colon,
    Comma,
    Str(String),
    Number,
    Bool,
    Null,
}

struct Tokenizer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(data: &'a [u8]) -> Self {
        let mut pos = 0;
        // Skip BOM
        if data.len() >= 3 && data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF {
            pos = 3;
        }
        Tokenizer { data, pos }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.data.len() {
            match self.data[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let b = self.peek()?;
        match b {
            b'{' => {
                self.pos += 1;
                Some(Token::ObjectStart)
            }
            b'}' => {
                self.pos += 1;
                Some(Token::ObjectEnd)
            }
            b'[' => {
                self.pos += 1;
                Some(Token::ArrayStart)
            }
            b']' => {
                self.pos += 1;
                Some(Token::ArrayEnd)
            }
            b':' => {
                self.pos += 1;
                Some(Token::Colon)
            }
            b',' => {
                self.pos += 1;
                Some(Token::Comma)
            }
            b'"' => self.read_string().map(Token::Str),
            b't' | b'f' => {
                self.skip_literal();
                Some(Token::Bool)
            }
            b'n' => {
                self.skip_literal();
                Some(Token::Null)
            }
            b'0'..=b'9' | b'-' => {
                self.skip_number();
                Some(Token::Number)
            }
            _ => {
                self.pos += 1;
                self.next_token()
            }
        }
    }

    fn read_string(&mut self) -> Option<String> {
        self.pos += 1;
        let mut result = String::new();
        while self.pos < self.data.len() {
            let b = self.data[self.pos];
            match b {
                b'"' => {
                    self.pos += 1;
                    return Some(result);
                }
                b'\\' => {
                    self.pos += 1;
                    if self.pos >= self.data.len() {
                        return Some(result);
                    }
                    match self.data[self.pos] {
                        b'"' => result.push('"'),
                        b'\\' => result.push('\\'),
                        b'/' => result.push('/'),
                        b'n' => result.push('\n'),
                        b'r' => result.push('\r'),
                        b't' => result.push('\t'),
                        b'b' => result.push('\u{0008}'),
                        b'f' => result.push('\u{000C}'),
                        b'u' => {
                            self.pos += 1;
                            if let Some(ch) = self.read_unicode_escape() {
                                result.push(ch);
                                continue;
                            }
                            continue;
                        }
                        other => {
                            result.push('\\');
                            result.push(other as char);
                        }
                    }
                    self.pos += 1;
                }
                _ => {
                    if b < 0x80 {
                        result.push(b as char);
                        self.pos += 1;
                    } else {
                        let remaining = &self.data[self.pos..];
                        if let Some(ch) = core::str::from_utf8(remaining)
                            .ok()
                            .and_then(|s| s.chars().next())
                        {
                            result.push(ch);
                            self.pos += ch.len_utf8();
                        } else {
                            self.pos += 1;
                        }
                    }
                }
            }
        }
        Some(result)
    }

    fn read_unicode_escape(&mut self) -> Option<char> {
        if self.pos + 4 > self.data.len() {
            return None;
        }
        let hex = core::str::from_utf8(&self.data[self.pos..self.pos + 4]).ok()?;
        let code = u32::from_str_radix(hex, 16).ok()?;
        self.pos += 4;
        char::from_u32(code)
    }

    fn skip_literal(&mut self) {
        while self.pos < self.data.len() && self.data[self.pos].is_ascii_alphabetic() {
            self.pos += 1;
        }
    }

    fn skip_number(&mut self) {
        while self.pos < self.data.len() {
            match self.data[self.pos] {
                b'0'..=b'9' | b'.' | b'-' | b'+' | b'e' | b'E' => self.pos += 1,
                _ => break,
            }
        }
    }
}

fn parse_value(tok: &mut Tokenizer, depth: u32) -> JsonValue {
    if depth > 8 {
        skip_value(tok);
        return JsonValue::Other;
    }

    let token = match tok.next_token() {
        Some(t) => t,
        None => return JsonValue::Null,
    };

    match token {
        Token::Str(s) => JsonValue::Str(s),
        Token::ObjectStart => parse_object(tok, depth),
        Token::ArrayStart => parse_array(tok, depth),
        Token::Null => JsonValue::Null,
        Token::Number | Token::Bool => JsonValue::Other,
        _ => JsonValue::Other,
    }
}

fn parse_object(tok: &mut Tokenizer, depth: u32) -> JsonValue {
    let mut map = HashMap::new();
    loop {
        tok.skip_whitespace();
        match tok.peek() {
            Some(b'}') => {
                tok.pos += 1;
                return JsonValue::Object(map);
            }
            None => return JsonValue::Object(map),
            _ => {}
        }

        let key = match tok.next_token() {
            Some(Token::Str(k)) => k,
            Some(Token::ObjectEnd) => return JsonValue::Object(map),
            _ => return JsonValue::Object(map),
        };

        match tok.next_token() {
            Some(Token::Colon) => {}
            _ => return JsonValue::Object(map),
        }

        let value = parse_value(tok, depth + 1);
        map.insert(key, value);

        tok.skip_whitespace();
        match tok.peek() {
            Some(b',') => {
                tok.pos += 1;
            }
            Some(b'}') => {
                tok.pos += 1;
                return JsonValue::Object(map);
            }
            _ => return JsonValue::Object(map),
        }
    }
}

fn parse_array(tok: &mut Tokenizer, depth: u32) -> JsonValue {
    let mut items = Vec::new();
    loop {
        tok.skip_whitespace();
        match tok.peek() {
            Some(b']') => {
                tok.pos += 1;
                return JsonValue::Array(items);
            }
            None => return JsonValue::Array(items),
            _ => {}
        }

        let value = parse_value(tok, depth + 1);
        items.push(value);

        tok.skip_whitespace();
        match tok.peek() {
            Some(b',') => {
                tok.pos += 1;
            }
            Some(b']') => {
                tok.pos += 1;
                return JsonValue::Array(items);
            }
            _ => return JsonValue::Array(items),
        }
    }
}

fn skip_value(tok: &mut Tokenizer) {
    let token = match tok.next_token() {
        Some(t) => t,
        None => return,
    };
    match token {
        Token::ObjectStart => {
            let mut depth = 1u32;
            while depth > 0 {
                match tok.next_token() {
                    Some(Token::ObjectStart) => depth += 1,
                    Some(Token::ObjectEnd) => depth -= 1,
                    None => return,
                    _ => {}
                }
            }
        }
        Token::ArrayStart => {
            let mut depth = 1u32;
            while depth > 0 {
                match tok.next_token() {
                    Some(Token::ArrayStart) => depth += 1,
                    Some(Token::ArrayEnd) => depth -= 1,
                    None => return,
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

pub fn extract_top_level(json: &str) -> HashMap<String, JsonValue> {
    let mut tok = Tokenizer::new(json.as_bytes());
    match parse_value(&mut tok, 0) {
        JsonValue::Object(map) => map,
        _ => HashMap::new(),
    }
}
