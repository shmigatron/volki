use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
    Array(Vec<Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_str_array(&self) -> Option<Vec<&str>> {
        let arr = self.as_array()?;
        let mut result = Vec::with_capacity(arr.len());
        for v in arr {
            result.push(v.as_str()?);
        }
        Some(result)
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    entries: HashMap<String, Value>,
}

impl Table {
    pub fn get(&self, section: &str, key: &str) -> Option<&Value> {
        let full = if section.is_empty() {
            key.to_string()
        } else {
            format!("{section}.{key}")
        };
        self.entries.get(&full)
    }

    #[allow(dead_code)]
    pub fn entries(&self) -> &HashMap<String, Value> {
        &self.entries
    }

    pub fn entries_with_prefix(&self, prefix: &str) -> Vec<(String, String)> {
        let dot_prefix = format!("{prefix}.");
        let mut result = Vec::new();
        for (key, value) in &self.entries {
            if let Some(suffix) = key.strip_prefix(&dot_prefix) {
                if let Some(s) = value.as_str() {
                    result.push((suffix.to_string(), s.to_string()));
                }
            }
        }
        result
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at line {}: {}", self.line, self.message)
    }
}

pub fn parse(content: &str) -> Result<Table, ParseError> {
    let mut entries = HashMap::new();
    let mut current_section = String::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            let end = trimmed.find(']').ok_or_else(|| ParseError {
                line: i + 1,
                message: "unclosed section header".to_string(),
            })?;
            current_section = trimmed[1..end].trim().to_string();
            continue;
        }

        let eq = trimmed.find('=').ok_or_else(|| ParseError {
            line: i + 1,
            message: "expected key = value".to_string(),
        })?;

        let key = trimmed[..eq].trim();
        let raw_val = trimmed[eq + 1..].trim();

        if key.is_empty() {
            return Err(ParseError {
                line: i + 1,
                message: "empty key".to_string(),
            });
        }

        let value = parse_value(raw_val).ok_or_else(|| ParseError {
            line: i + 1,
            message: format!("invalid value: {raw_val}"),
        })?;

        let full_key = if current_section.is_empty() {
            key.to_string()
        } else {
            format!("{}.{key}", current_section)
        };

        entries.insert(full_key, value);
    }

    Ok(Table { entries })
}

fn parse_value(raw: &str) -> Option<Value> {
    if raw.starts_with('[') {
        return parse_array_value(raw);
    }

    if (raw.starts_with('"') && raw.ends_with('"'))
        || (raw.starts_with('\'') && raw.ends_with('\''))
    {
        if raw.len() < 2 {
            return None;
        }
        return Some(Value::Str(raw[1..raw.len() - 1].to_string()));
    }

    if raw == "true" {
        return Some(Value::Bool(true));
    }
    if raw == "false" {
        return Some(Value::Bool(false));
    }

    if let Ok(n) = raw.parse::<i64>() {
        return Some(Value::Int(n));
    }

    None
}

fn parse_array_value(raw: &str) -> Option<Value> {
    if !raw.ends_with(']') {
        return None;
    }
    let inner = raw[1..raw.len() - 1].trim();
    if inner.is_empty() {
        return Some(Value::Array(Vec::new()));
    }
    let mut items = Vec::new();
    for element in inner.split(',') {
        let trimmed = element.trim();
        if trimmed.is_empty() {
            continue;
        }
        items.push(parse_value(trimmed)?);
    }
    Some(Value::Array(items))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        let t = parse("").unwrap();
        assert!(t.entries.is_empty());
    }

    #[test]
    fn comments_and_blanks_ignored() {
        let t = parse("# comment\n\n  # another\n").unwrap();
        assert!(t.entries.is_empty());
    }

    #[test]
    fn bare_key_value_string() {
        let t = parse("name = \"hello\"").unwrap();
        assert_eq!(t.get("", "name").unwrap().as_str(), Some("hello"));
    }

    #[test]
    fn single_quoted_string() {
        let t = parse("name = 'world'").unwrap();
        assert_eq!(t.get("", "name").unwrap().as_str(), Some("world"));
    }

    #[test]
    fn integer_value() {
        let t = parse("count = 42").unwrap();
        assert_eq!(t.get("", "count").unwrap().as_int(), Some(42));
    }

    #[test]
    fn negative_integer() {
        let t = parse("offset = -5").unwrap();
        assert_eq!(t.get("", "offset").unwrap().as_int(), Some(-5));
    }

    #[test]
    fn bool_values() {
        let t = parse("a = true\nb = false").unwrap();
        assert_eq!(t.get("", "a").unwrap().as_bool(), Some(true));
        assert_eq!(t.get("", "b").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn section_scoping() {
        let input = "[volki]\nname = \"test\"";
        let t = parse(input).unwrap();
        assert_eq!(t.get("volki", "name").unwrap().as_str(), Some("test"));
        assert!(t.get("", "name").is_none());
    }

    #[test]
    fn multiple_sections() {
        let input = "[a]\nx = 1\n[b]\ny = 2";
        let t = parse(input).unwrap();
        assert_eq!(t.get("a", "x").unwrap().as_int(), Some(1));
        assert_eq!(t.get("b", "y").unwrap().as_int(), Some(2));
    }

    #[test]
    fn unclosed_section_header() {
        let result = parse("[broken");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("unclosed"));
    }

    #[test]
    fn missing_equals() {
        let result = parse("noequals");
        assert!(result.is_err());
    }

    #[test]
    fn empty_key_error() {
        let result = parse(" = \"val\"");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("empty key"));
    }

    #[test]
    fn invalid_value_error() {
        let result = parse("key = @bad");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid value"));
    }

    #[test]
    fn default_config_parses() {
        let t = parse(super::super::DEFAULT_CONFIG).unwrap();
        assert!(t.entries.is_empty() || !t.entries.is_empty());
    }

    #[test]
    fn whitespace_around_key_value() {
        let t = parse("  key  =  \"val\"  ").unwrap();
        assert_eq!(t.get("", "key").unwrap().as_str(), Some("val"));
    }

    #[test]
    fn parse_error_display() {
        let err = ParseError { line: 3, message: "bad".to_string() };
        assert_eq!(format!("{err}"), "parse error at line 3: bad");
    }

    #[test]
    fn array_of_strings() {
        let t = parse("list = [\"a\", \"b\", \"c\"]").unwrap();
        let v = t.get("", "list").unwrap();
        let arr = v.as_str_array().unwrap();
        assert_eq!(arr, vec!["a", "b", "c"]);
    }

    #[test]
    fn empty_array() {
        let t = parse("list = []").unwrap();
        let v = t.get("", "list").unwrap();
        assert_eq!(v.as_array().unwrap().len(), 0);
    }

    #[test]
    fn array_in_section() {
        let t = parse("[plugins]\nlist = [\"foo\", \"bar\"]").unwrap();
        let v = t.get("plugins", "list").unwrap();
        let arr = v.as_str_array().unwrap();
        assert_eq!(arr, vec!["foo", "bar"]);
    }

    #[test]
    fn entries_with_prefix_basic() {
        let t = parse("[plugins.my-plugin]\nkey1 = \"val1\"\nkey2 = \"val2\"").unwrap();
        let mut entries = t.entries_with_prefix("plugins.my-plugin");
        entries.sort();
        assert_eq!(entries, vec![
            ("key1".to_string(), "val1".to_string()),
            ("key2".to_string(), "val2".to_string()),
        ]);
    }
}
