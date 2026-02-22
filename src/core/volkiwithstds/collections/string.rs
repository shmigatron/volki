//! String — owned UTF-8 string, wraps Vec<u8>.

use super::vec::Vec;
use core::borrow::Borrow;
use core::fmt;
use core::ops::Deref;

/// An owned, growable UTF-8 string.
pub struct String {
    bytes: Vec<u8>,
}

impl String {
    /// Creates an empty String.
    pub const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Creates a String with pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(cap),
        }
    }

    /// Creates a String from a &str.
    pub fn from(s: &str) -> Self {
        let mut string = Self::with_capacity(s.len());
        string.push_str(s);
        string
    }

    /// Appends a string slice.
    pub fn push_str(&mut self, s: &str) {
        self.bytes.extend_from_slice(s.as_bytes());
    }

    /// Appends a single character.
    pub fn push(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        self.push_str(encoded);
    }

    /// Returns a &str view.
    pub fn as_str(&self) -> &str {
        // Safety: we only allow valid UTF-8 to be inserted
        unsafe { core::str::from_utf8_unchecked(self.bytes.as_slice()) }
    }

    /// Returns the byte length.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Returns the underlying bytes as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// Returns an iterator over the lines of this string.
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.as_str().lines()
    }

    /// Returns a trimmed view.
    pub fn trim(&self) -> &str {
        self.as_str().trim()
    }

    /// Whether the string starts with a pattern.
    pub fn starts_with(&self, pat: &str) -> bool {
        self.as_str().starts_with(pat)
    }

    /// Whether the string ends with a pattern.
    pub fn ends_with(&self, pat: &str) -> bool {
        self.as_str().ends_with(pat)
    }

    /// Whether the string contains a pattern.
    pub fn contains(&self, pat: &str) -> bool {
        self.as_str().contains(pat)
    }

    /// Find the first occurrence of a pattern.
    pub fn find(&self, pat: &str) -> Option<usize> {
        self.as_str().find(pat)
    }

    /// Split by a pattern.
    pub fn split<'a>(&'a self, pat: &'a str) -> impl Iterator<Item = &'a str> {
        self.as_str().split(pat)
    }

    /// Replace all occurrences of `from` with `to`.
    pub fn replace(&self, from: &str, to: &str) -> String {
        let s = self.as_str();
        let mut result = String::new();
        let mut last_end = 0;
        for (start, _) in s.match_indices(from) {
            result.push_str(&s[last_end..start]);
            result.push_str(to);
            last_end = start + from.len();
        }
        result.push_str(&s[last_end..]);
        result
    }

    /// Convert to lowercase (ASCII only).
    pub fn to_lowercase(&self) -> String {
        let mut result = String::with_capacity(self.len());
        for c in self.as_str().chars() {
            if c.is_ascii_uppercase() {
                result.push((c as u8 + 32) as char);
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Convert to uppercase (ASCII only).
    pub fn to_uppercase(&self) -> String {
        let mut result = String::with_capacity(self.len());
        for c in self.as_str().chars() {
            if c.is_ascii_lowercase() {
                result.push((c as u8 - 32) as char);
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Repeat the string `n` times.
    pub fn repeat(&self, n: usize) -> String {
        let mut result = String::with_capacity(self.len() * n);
        for _ in 0..n {
            result.push_str(self.as_str());
        }
        result
    }

    /// Creates a String from a byte vector, returning Err if invalid UTF-8.
    pub fn from_utf8(bytes: Vec<u8>) -> Result<Self, Vec<u8>> {
        match core::str::from_utf8(bytes.as_slice()) {
            Ok(_) => Ok(Self { bytes }),
            Err(_) => Err(bytes),
        }
    }

    /// Creates a String from bytes, replacing invalid sequences with U+FFFD.
    pub fn from_utf8_lossy(bytes: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;
        while i < bytes.len() {
            match core::str::from_utf8(&bytes[i..]) {
                Ok(valid) => {
                    result.push_str(valid);
                    break;
                }
                Err(e) => {
                    let valid_up_to = e.valid_up_to();
                    if valid_up_to > 0 {
                        let valid =
                            unsafe { core::str::from_utf8_unchecked(&bytes[i..i + valid_up_to]) };
                        result.push_str(valid);
                    }
                    result.push('\u{FFFD}');
                    i += valid_up_to + e.error_len().unwrap_or(1);
                }
            }
        }
        result
    }

    /// Converts the String into its byte vector.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Returns an owned clone of this string.
    pub fn to_owned(&self) -> String {
        self.clone()
    }

    /// Clear the string.
    pub fn clear(&mut self) {
        self.bytes.clear();
    }

    /// Truncate to `new_len` bytes (panics if not on char boundary).
    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.len() {
            assert!(self.as_str().is_char_boundary(new_len));
            self.bytes.truncate(new_len);
        }
    }

    /// Reserve capacity for at least `additional` more bytes.
    pub fn reserve(&mut self, additional: usize) {
        self.bytes.reserve(additional);
    }

    /// Returns chars iterator.
    pub fn chars(&self) -> core::str::Chars<'_> {
        self.as_str().chars()
    }

    /// Splits the string at the given byte index.
    pub fn split_off(&mut self, at: usize) -> String {
        assert!(self.as_str().is_char_boundary(at));
        let mut other = String::new();
        let remaining = &self.as_str()[at..];
        other.push_str(remaining);
        self.bytes.truncate(at);
        other
    }
}

// ── Trait implementations ───────────────────────────────────────────────────

impl Deref for String {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for String {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for String {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Clone for String {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
        }
    }
}

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for String {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for String {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl Eq for String {}

impl PartialOrd for String {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for String {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl core::hash::Hash for String {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        // Hash the &str, matching std behavior
        self.as_str().hash(state);
    }
}

impl core::fmt::Write for String {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl Default for String {
    fn default() -> Self {
        String::new()
    }
}

impl From<&str> for String {
    fn from(s: &str) -> Self {
        String::from(s)
    }
}

/// String conversion trait that returns the custom volkiwithstds `String`.
pub trait ToString {
    fn to_vstring(&self) -> String;
}

impl<T: core::fmt::Display + ?Sized> ToString for T {
    fn to_vstring(&self) -> String {
        let mut s = String::new();
        let _ = core::fmt::write(&mut s, format_args!("{}", self));
        s
    }
}

impl<'a> FromIterator<&'a str> for String {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut s = String::new();
        for piece in iter {
            s.push_str(piece);
        }
        s
    }
}

impl FromIterator<char> for String {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut s = String::new();
        for c in iter {
            s.push(c);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_string() {
        let mut s = String::from("hello");
        s.push_str(" world");
        assert_eq!(s.as_str(), "hello world");
        assert_eq!(s.len(), 11);
    }

    #[test]
    fn test_contains_find() {
        let s = String::from("hello world");
        assert!(s.contains("world"));
        assert_eq!(s.find("world"), Some(6));
        assert!(!s.contains("xyz"));
    }

    #[test]
    fn test_replace() {
        let s = String::from("hello hello");
        let r = s.replace("hello", "world");
        assert_eq!(r.as_str(), "world world");
    }

    #[test]
    fn test_lowercase() {
        let s = String::from("Hello WORLD");
        assert_eq!(s.to_lowercase().as_str(), "hello world");
    }

    #[test]
    fn test_hash() {
        use crate::core::volkiwithstds::collections::hash::SipHasher;
        use core::hash::{Hash, Hasher};
        // Verify that String hashes the same as &str
        let s = String::from("test");
        let mut h1 = SipHasher::new();
        let mut h2 = SipHasher::new();
        s.hash(&mut h1);
        "test".hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_write_trait() {
        use core::fmt::Write;
        let mut s = String::new();
        write!(s, "hello {}", 42).unwrap();
        assert_eq!(s.as_str(), "hello 42");
    }
}
