//! HTTP response builder.

use super::headers::Headers;
use super::status::StatusCode;
use crate::core::volkiwithstds::collections::Vec;

pub struct Response {
    pub status: StatusCode,
    pub headers: Headers,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: Headers::new(),
            body: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    pub fn not_found() -> Self {
        let mut r = Self::new(StatusCode::NOT_FOUND);
        r.headers.set("Content-Type", "text/plain");
        r.body.extend_from_slice(b"404 Not Found");
        r
    }

    pub fn internal_error() -> Self {
        let mut r = Self::new(StatusCode::INTERNAL_SERVER_ERROR);
        r.headers.set("Content-Type", "text/plain");
        r.body.extend_from_slice(b"500 Internal Server Error");
        r
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.set(name, value);
        self
    }

    pub fn html(mut self, html: &str) -> Self {
        self.headers.set("Content-Type", "text/html; charset=utf-8");
        self.body = Vec::new();
        self.body.extend_from_slice(html.as_bytes());
        self
    }

    pub fn json(mut self, json: &str) -> Self {
        self.headers.set("Content-Type", "application/json");
        self.body = Vec::new();
        self.body.extend_from_slice(json.as_bytes());
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.headers.set("Content-Type", "text/plain; charset=utf-8");
        self.body = Vec::new();
        self.body.extend_from_slice(text.as_bytes());
        self
    }

    pub fn document(self, doc: &crate::libs::web::html::document::HtmlDocument) -> Self {
        let rendered = doc.render();
        self.html(rendered.as_str())
    }

    pub fn redirect(mut self, location: &str) -> Self {
        self.status = StatusCode::FOUND;
        self.headers.set("Location", location);
        self
    }

    pub fn body_bytes(mut self, bytes: &[u8]) -> Self {
        self.body = Vec::new();
        self.body.extend_from_slice(bytes);
        self
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(256 + self.body.len());

        // Status line
        buf.extend_from_slice(b"HTTP/1.1 ");
        write_u16(self.status.code(), &mut buf);
        buf.extend_from_slice(b" ");
        buf.extend_from_slice(self.status.reason_phrase().as_bytes());
        buf.extend_from_slice(b"\r\n");

        // Content-Length (auto-set)
        let mut has_content_length = false;
        for (k, _) in self.headers.iter() {
            let lower = k.to_ascii_lowercase();
            if lower == "content-length" {
                has_content_length = true;
                break;
            }
        }

        // Write user headers
        self.headers.write_to(&mut buf);

        // Auto content-length
        if !has_content_length {
            buf.extend_from_slice(b"Content-Length: ");
            write_usize(self.body.len(), &mut buf);
            buf.extend_from_slice(b"\r\n");
        }

        // End of headers
        buf.extend_from_slice(b"\r\n");

        // Body
        buf.extend_from_slice(self.body.as_slice());

        buf
    }
}

fn write_u16(val: u16, buf: &mut Vec<u8>) {
    let mut tmp = [0u8; 5];
    let mut pos = 5;
    let mut v = val as u32;
    if v == 0 {
        buf.push(b'0');
        return;
    }
    while v > 0 {
        pos -= 1;
        tmp[pos] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    buf.extend_from_slice(&tmp[pos..]);
}

fn write_usize(val: usize, buf: &mut Vec<u8>) {
    let mut tmp = [0u8; 20];
    let mut pos = 20;
    let mut v = val;
    if v == 0 {
        buf.push(b'0');
        return;
    }
    while v > 0 {
        pos -= 1;
        tmp[pos] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    buf.extend_from_slice(&tmp[pos..]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_basic() {
        let resp = Response::ok().text("hello");
        let bytes = resp.serialize();
        let s = core::str::from_utf8(bytes.as_slice()).unwrap();
        assert!(s.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(s.contains("Content-Type: text/plain"));
        assert!(s.contains("Content-Length: 5"));
        assert!(s.ends_with("hello"));
    }

    #[test]
    fn test_not_found() {
        let resp = Response::not_found();
        assert_eq!(resp.status, StatusCode::NOT_FOUND);
    }
}
