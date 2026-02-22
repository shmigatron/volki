//! Incremental HTTP/1.1 request parser.

use super::headers::Headers;
use super::method::Method;
use super::request::Request;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::libs::web::security::SizeLimits;

pub enum ParseResult {
    Complete(Request, usize),
    Incomplete,
    Error(&'static str),
}

pub fn parse_request(buf: &[u8], limits: &SizeLimits) -> ParseResult {
    // Find header terminator \r\n\r\n
    let header_end = match find_header_end(buf) {
        Some(pos) => pos,
        None => {
            if buf.len() > limits.max_header_size {
                return ParseResult::Error("headers too large");
            }
            return ParseResult::Incomplete;
        }
    };

    let header_bytes = &buf[..header_end];
    if header_bytes.len() > limits.max_header_size {
        return ParseResult::Error("headers too large");
    }

    // Parse request line
    let first_line_end = match find_crlf(header_bytes) {
        Some(pos) => pos,
        None => return ParseResult::Error("malformed request line"),
    };

    let request_line = &header_bytes[..first_line_end];

    // Parse: METHOD SP PATH SP HTTP/x.x
    let (method, path) = match parse_request_line(request_line) {
        Some(v) => v,
        None => return ParseResult::Error("malformed request line"),
    };

    // URI length check
    if path.len() > limits.max_uri_length {
        return ParseResult::Error("URI too long");
    }

    // Parse headers
    let mut headers = Headers::new();
    let mut pos = first_line_end + 2; // skip \r\n
    while pos < header_bytes.len() {
        let line_end = match find_crlf(&header_bytes[pos..]) {
            Some(p) => pos + p,
            None => header_bytes.len(),
        };
        let line = &header_bytes[pos..line_end];
        if line.is_empty() {
            break;
        }
        if let Some(colon) = memchr(b':', line) {
            let name = trim_bytes(&line[..colon]);
            let value = trim_bytes(&line[colon + 1..]);
            if let (Ok(n), Ok(v)) = (core::str::from_utf8(name), core::str::from_utf8(value)) {
                headers.set(n, v);
            }
        }
        pos = line_end + 2; // skip \r\n
    }

    // Body handling
    let headers_total = header_end + 4; // include \r\n\r\n
    let content_length = headers.content_length().unwrap_or(0);

    if content_length > limits.max_body_size {
        return ParseResult::Error("body too large");
    }

    let total_needed = headers_total + content_length;
    if buf.len() < total_needed {
        return ParseResult::Incomplete;
    }

    let body = if content_length > 0 {
        let mut b = Vec::with_capacity(content_length);
        b.extend_from_slice(&buf[headers_total..headers_total + content_length]);
        b
    } else {
        Vec::new()
    };

    let request = Request::new(method, path, headers, body);
    ParseResult::Complete(request, total_needed)
}

fn parse_request_line(line: &[u8]) -> Option<(Method, String)> {
    let first_sp = memchr(b' ', line)?;
    let method = Method::from_bytes(&line[..first_sp])?;

    let rest = &line[first_sp + 1..];
    let second_sp = memchr(b' ', rest)?;
    let path_bytes = &rest[..second_sp];

    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => String::from(s),
        Err(_) => return None,
    };

    Some((method, path))
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    if buf.len() < 4 {
        return None;
    }
    for i in 0..buf.len() - 3 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' && buf[i + 2] == b'\r' && buf[i + 3] == b'\n' {
            return Some(i);
        }
    }
    None
}

fn find_crlf(buf: &[u8]) -> Option<usize> {
    if buf.len() < 2 {
        return None;
    }
    for i in 0..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            return Some(i);
        }
    }
    None
}

fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    for (i, &b) in haystack.iter().enumerate() {
        if b == needle {
            return Some(i);
        }
    }
    None
}

fn trim_bytes(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && (bytes[start] == b' ' || bytes[start] == b'\t') {
        start += 1;
    }
    while end > start && (bytes[end - 1] == b' ' || bytes[end - 1] == b'\t') {
        end -= 1;
    }
    &bytes[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> SizeLimits {
        SizeLimits::default()
    }

    #[test]
    fn test_parse_simple_get() {
        let raw = b"GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n";
        match parse_request(raw, &defaults()) {
            ParseResult::Complete(req, consumed) => {
                assert_eq!(req.method, Method::Get);
                assert_eq!(req.route_path.as_str(), "/hello");
                assert_eq!(consumed, raw.len());
                assert_eq!(req.headers.get("host"), Some("localhost"));
            }
            _ => panic!("expected Complete"),
        }
    }

    #[test]
    fn test_parse_with_body() {
        let raw = b"POST /data HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello";
        match parse_request(raw, &defaults()) {
            ParseResult::Complete(req, consumed) => {
                assert_eq!(req.method, Method::Post);
                assert_eq!(req.body.as_slice(), b"hello");
                assert_eq!(consumed, raw.len());
            }
            _ => panic!("expected Complete"),
        }
    }

    #[test]
    fn test_parse_incomplete() {
        let raw = b"GET /hello HTTP/1.1\r\nHost: local";
        match parse_request(raw, &defaults()) {
            ParseResult::Incomplete => {}
            _ => panic!("expected Incomplete"),
        }
    }

    #[test]
    fn test_parse_with_query() {
        let raw = b"GET /search?q=rust&page=1 HTTP/1.1\r\nHost: localhost\r\n\r\n";
        match parse_request(raw, &defaults()) {
            ParseResult::Complete(req, _) => {
                assert_eq!(req.route_path.as_str(), "/search");
                assert_eq!(req.query_string.as_str(), "q=rust&page=1");
                let params = req.query_params();
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], ("q", "rust"));
                assert_eq!(params[1], ("page", "1"));
            }
            _ => panic!("expected Complete"),
        }
    }

    #[test]
    fn test_parse_malformed() {
        let raw = b"INVALID\r\n\r\n";
        match parse_request(raw, &defaults()) {
            ParseResult::Error(_) => {}
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn test_uri_too_long() {
        let limits = SizeLimits {
            max_uri_length: 10,
            ..SizeLimits::default()
        };
        let raw = b"GET /this-is-a-very-long-uri-path HTTP/1.1\r\nHost: localhost\r\n\r\n";
        match parse_request(raw, &limits) {
            ParseResult::Error(msg) => assert_eq!(msg, "URI too long"),
            _ => panic!("expected Error"),
        }
    }
}
