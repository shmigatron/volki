use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Str(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
    Null,
    Other,
}

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
                        if let Some(ch) = std::str::from_utf8(remaining)
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
        let hex = std::str::from_utf8(&self.data[self.pos..self.pos + 4]).ok()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_object() {
        let map = extract_top_level("{}");
        assert!(map.is_empty());
    }

    #[test]
    fn single_string() {
        let map = extract_top_level(r#"{"name": "hello"}"#);
        assert_eq!(map.get("name").unwrap().as_str(), Some("hello"));
    }

    #[test]
    fn multiple_fields() {
        let map = extract_top_level(r#"{"a": "1", "b": "2"}"#);
        assert_eq!(map.get("a").unwrap().as_str(), Some("1"));
        assert_eq!(map.get("b").unwrap().as_str(), Some("2"));
    }

    #[test]
    fn nested_object() {
        let map = extract_top_level(r#"{"outer": {"inner": "val"}}"#);
        let outer = map.get("outer").unwrap().as_object().unwrap();
        assert_eq!(outer.get("inner").unwrap().as_str(), Some("val"));
    }

    #[test]
    fn array_of_objects() {
        let map = extract_top_level(r#"{"items": [{"a": "1"}, {"b": "2"}]}"#);
        let arr = map.get("items").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn null_value() {
        let map = extract_top_level(r#"{"x": null}"#);
        assert_eq!(*map.get("x").unwrap(), JsonValue::Null);
    }

    #[test]
    fn number_value() {
        let map = extract_top_level(r#"{"n": 42}"#);
        assert_eq!(*map.get("n").unwrap(), JsonValue::Other);
    }

    #[test]
    fn boolean_value() {
        let map = extract_top_level(r#"{"b": true}"#);
        assert_eq!(*map.get("b").unwrap(), JsonValue::Other);
    }

    #[test]
    fn empty_string_value() {
        let map = extract_top_level(r#"{"e": ""}"#);
        assert_eq!(map.get("e").unwrap().as_str(), Some(""));
    }

    #[test]
    fn unicode_escape() {
        let map = extract_top_level(r#"{"u": "\u0041"}"#);
        assert_eq!(map.get("u").unwrap().as_str(), Some("A"));
    }

    #[test]
    fn backslash_escapes() {
        let map = extract_top_level(r#"{"s": "a\"b\\c\/d"}"#);
        assert_eq!(map.get("s").unwrap().as_str(), Some("a\"b\\c/d"));
    }

    #[test]
    fn bom_prefix() {
        let json = "\u{FEFF}{\"name\": \"bom\"}";
        let map = extract_top_level(json);
        assert_eq!(map.get("name").unwrap().as_str(), Some("bom"));
    }

    #[test]
    fn whitespace_variations() {
        let json = "  {\n  \"a\" :  \"1\"  ,\n  \"b\"  :  \"2\" \n }  ";
        let map = extract_top_level(json);
        assert_eq!(map.get("a").unwrap().as_str(), Some("1"));
        assert_eq!(map.get("b").unwrap().as_str(), Some("2"));
    }

    #[test]
    fn depth_limit() {
        // 9 levels of nesting should hit the depth limit (>8)
        let json = r#"{"a":{"b":{"c":{"d":{"e":{"f":{"g":{"h":{"i":"deep"}}}}}}}}}"#;
        let map = extract_top_level(json);
        // Should parse but the deepest value will be Other
        assert!(map.contains_key("a"));
    }

    #[test]
    fn non_object_input() {
        let map = extract_top_level("[1,2,3]");
        assert!(map.is_empty());
    }

    #[test]
    fn empty_input() {
        let map = extract_top_level("");
        assert!(map.is_empty());
    }

    #[test]
    fn realistic_package_json() {
        let json = r#"{
            "name": "my-app",
            "version": "1.2.3",
            "license": "MIT",
            "dependencies": {
                "lodash": "^4.17.21",
                "express": "^4.18.0"
            },
            "devDependencies": {
                "jest": "^29.0.0"
            }
        }"#;
        let map = extract_top_level(json);
        assert_eq!(map.get("name").unwrap().as_str(), Some("my-app"));
        assert_eq!(map.get("version").unwrap().as_str(), Some("1.2.3"));
        assert_eq!(map.get("license").unwrap().as_str(), Some("MIT"));
        let deps = map.get("dependencies").unwrap().as_object().unwrap();
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn as_str_on_str() {
        assert_eq!(JsonValue::Str("hi".to_string()).as_str(), Some("hi"));
    }

    #[test]
    fn as_str_on_non_str() {
        assert_eq!(JsonValue::Null.as_str(), None);
        assert_eq!(JsonValue::Other.as_str(), None);
    }

    #[test]
    fn as_array_on_array() {
        let arr = JsonValue::Array(vec![JsonValue::Null]);
        assert_eq!(arr.as_array().unwrap().len(), 1);
    }

    #[test]
    fn as_array_on_non_array() {
        assert!(JsonValue::Str("x".to_string()).as_array().is_none());
    }

    #[test]
    fn as_object_on_object() {
        let obj = JsonValue::Object(HashMap::new());
        assert!(obj.as_object().unwrap().is_empty());
    }

    #[test]
    fn as_object_on_non_object() {
        assert!(JsonValue::Null.as_object().is_none());
    }
}
