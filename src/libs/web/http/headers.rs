//! HTTP headers — case-insensitive header map.

use crate::core::volkiwithstds::collections::{String, Vec};

pub struct Headers {
    entries: Vec<(String, String)>,
}

impl Headers {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        let lower = ascii_lowercase(name);
        for (k, v) in self.entries.iter() {
            if ascii_lowercase(k.as_str()) == lower {
                return Some(v.as_str());
            }
        }
        None
    }

    pub fn set(&mut self, name: &str, value: &str) {
        let lower = ascii_lowercase(name);
        for (k, v) in self.entries.iter_mut() {
            if ascii_lowercase(k.as_str()) == lower {
                *v = String::from(value);
                return;
            }
        }
        self.entries.push((String::from(name), String::from(value)));
    }

    pub fn append(&mut self, name: &str, value: &str) {
        self.entries.push((String::from(name), String::from(value)));
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn content_length(&self) -> Option<usize> {
        self.get("content-length").and_then(|v| parse_usize(v))
    }

    pub fn connection_keep_alive(&self) -> bool {
        match self.get("connection") {
            Some(v) => {
                let lower = ascii_lowercase(v);
                lower.as_str() != "close"
            }
            None => true,
        }
    }

    pub fn write_to(&self, buf: &mut Vec<u8>) {
        for (k, v) in self.entries.iter() {
            buf.extend_from_slice(k.as_bytes());
            buf.extend_from_slice(b": ");
            buf.extend_from_slice(v.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }
    }
}

fn ascii_lowercase(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            result.push((c as u8 + 32) as char);
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_usize(s: &str) -> Option<usize> {
    let mut result: usize = 0;
    for b in s.as_bytes() {
        if *b < b'0' || *b > b'9' {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((*b - b'0') as usize)?;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive_get() {
        let mut h = Headers::new();
        h.set("Content-Type", "text/html");
        assert_eq!(h.get("content-type"), Some("text/html"));
        assert_eq!(h.get("CONTENT-TYPE"), Some("text/html"));
    }

    #[test]
    fn test_content_length() {
        let mut h = Headers::new();
        h.set("Content-Length", "42");
        assert_eq!(h.content_length(), Some(42));
    }

    #[test]
    fn test_set_overwrites() {
        let mut h = Headers::new();
        h.set("Host", "old.com");
        h.set("host", "new.com");
        assert_eq!(h.get("Host"), Some("new.com"));
    }
}
