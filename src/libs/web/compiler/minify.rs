//! Generated output compactors (Rust/JS).
//!
//! These compactors are intentionally conservative:
//! - preserve string/raw-string/template literal contents
//! - remove comments
//! - collapse whitespace runs to a single ASCII space
//! - return structured errors on unterminated tokens

use crate::core::volkiwithstds::collections::String;

#[derive(Debug, Clone)]
pub struct MinifyError {
    pub kind: &'static str,
    pub offset: usize,
    pub line: usize,
    pub col: usize,
}

impl core::fmt::Display for MinifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} at {}:{}", self.kind, self.line, self.col)
    }
}

fn line_col_at(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for &b in source.as_bytes().iter().take(offset.min(source.len())) {
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn minify_err(source: &str, offset: usize, kind: &'static str) -> MinifyError {
    let (line, col) = line_col_at(source, offset);
    MinifyError { kind, offset, line, col }
}

fn emit_pending_space(out: &mut crate::core::volkiwithstds::collections::Vec<u8>, pending_ws: &mut bool) {
    if *pending_ws && !out.is_empty() {
        out.push(b' ');
    }
    *pending_ws = false;
}

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

pub fn minify_route_mod_generated(input: &str) -> Result<String, MinifyError> {
    minify_rust_generated(input)
}

pub fn minify_rust_generated(input: &str) -> Result<String, MinifyError> {
    let bytes = input.as_bytes();
    let mut out = crate::core::volkiwithstds::collections::Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    let mut pending_ws = false;

    while i < bytes.len() {
        let b = bytes[i];

        // whitespace collapse
        if is_ws(b) {
            pending_ws = true;
            i += 1;
            continue;
        }

        // line comment
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            pending_ws = true;
            continue;
        }

        // block comment (nested in Rust)
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            let mut depth = 1i32;
            while i < bytes.len() {
                if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                    continue;
                }
                if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                i += 1;
            }
            if depth != 0 {
                return Err(minify_err(input, i.saturating_sub(1), "unterminated block comment"));
            }
            pending_ws = true;
            continue;
        }

        // raw string: r###" ... "###
        if b == b'r' || (b == b'b' && i + 1 < bytes.len() && bytes[i + 1] == b'r') {
            let start = i;
            let mut k = i;
            if bytes[k] == b'b' {
                k += 1;
            }
            if bytes[k] == b'r' {
                k += 1;
                let mut hashes = 0usize;
                while k < bytes.len() && bytes[k] == b'#' {
                    hashes += 1;
                    k += 1;
                }
                if k < bytes.len() && bytes[k] == b'"' {
                    emit_pending_space(&mut out, &mut pending_ws);
                    // copy prefix: r###"
                    for &ch in &bytes[start..=k] {
                        out.push(ch);
                    }
                    i = k + 1;
                    // copy body until closing quote + hashes
                    let mut found = false;
                    while i < bytes.len() {
                        out.push(bytes[i]);
                        if bytes[i] == b'"' {
                            let mut ok = true;
                            for h in 0..hashes {
                                if i + 1 + h >= bytes.len() || bytes[i + 1 + h] != b'#' {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                for h in 0..hashes {
                                    out.push(bytes[i + 1 + h]);
                                }
                                i += 1 + hashes;
                                found = true;
                                break;
                            }
                        }
                        i += 1;
                    }
                    if !found {
                        return Err(minify_err(input, i.saturating_sub(1), "unterminated raw string"));
                    }
                    continue;
                }
            }
        }

        // normal string
        if b == b'"' {
            emit_pending_space(&mut out, &mut pending_ws);
            out.push(b'"');
            i += 1;
            while i < bytes.len() {
                out.push(bytes[i]);
                if bytes[i] == b'\\' {
                    i += 1;
                    if i < bytes.len() {
                        out.push(bytes[i]);
                        i += 1;
                        continue;
                    }
                    return Err(minify_err(input, i.saturating_sub(1), "unterminated escape"));
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            if i > bytes.len() || (i == bytes.len() && *out.last().unwrap_or(&0) != b'"') {
                return Err(minify_err(input, i.saturating_sub(1), "unterminated string"));
            }
            continue;
        }

        emit_pending_space(&mut out, &mut pending_ws);
        out.push(b);
        i += 1;
    }

    let s = core::str::from_utf8(out.as_slice()).map_err(|_| minify_err(input, 0, "invalid utf8 after minify"))?;
    Ok(String::from(s.trim()))
}

pub fn minify_js_generated(input: &str) -> Result<String, MinifyError> {
    let bytes = input.as_bytes();
    let mut out = crate::core::volkiwithstds::collections::Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    let mut pending_ws = false;

    while i < bytes.len() {
        let b = bytes[i];

        if is_ws(b) {
            pending_ws = true;
            i += 1;
            continue;
        }

        // line comment
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            pending_ws = true;
            continue;
        }

        // block comment
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            let mut closed = false;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                return Err(minify_err(input, i.saturating_sub(1), "unterminated block comment"));
            }
            pending_ws = true;
            continue;
        }

        // single or double quote strings
        if b == b'"' || b == b'\'' {
            let q = b;
            emit_pending_space(&mut out, &mut pending_ws);
            out.push(q);
            i += 1;
            let mut closed = false;
            while i < bytes.len() {
                out.push(bytes[i]);
                if bytes[i] == b'\\' {
                    i += 1;
                    if i < bytes.len() {
                        out.push(bytes[i]);
                        i += 1;
                        continue;
                    }
                    return Err(minify_err(input, i.saturating_sub(1), "unterminated escape"));
                }
                if bytes[i] == q {
                    i += 1;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                return Err(minify_err(input, i.saturating_sub(1), "unterminated string"));
            }
            continue;
        }

        // template literals
        if b == b'`' {
            emit_pending_space(&mut out, &mut pending_ws);
            out.push(b'`');
            i += 1;
            let mut closed = false;
            while i < bytes.len() {
                out.push(bytes[i]);
                if bytes[i] == b'\\' {
                    i += 1;
                    if i < bytes.len() {
                        out.push(bytes[i]);
                        i += 1;
                        continue;
                    }
                    return Err(minify_err(input, i.saturating_sub(1), "unterminated template escape"));
                }
                if bytes[i] == b'`' {
                    i += 1;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                return Err(minify_err(input, i.saturating_sub(1), "unterminated template literal"));
            }
            continue;
        }

        emit_pending_space(&mut out, &mut pending_ws);
        out.push(b);
        i += 1;
    }

    let s = core::str::from_utf8(out.as_slice()).map_err(|_| minify_err(input, 0, "invalid utf8 after minify"))?;
    Ok(String::from(s.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_minify_removes_comments_and_newlines() {
        let src = "fn a() {\n// comment\nlet x = 1; /* b */ let y = \"a b\";\n}\n";
        let out = minify_rust_generated(src).unwrap();
        assert!(!out.contains("comment"));
        assert!(out.contains("let y = \"a b\";"));
    }

    #[test]
    fn rust_minify_preserves_raw_string() {
        let src = "let s = r#\"a // not comment\\n\"#; // c\n";
        let out = minify_rust_generated(src).unwrap();
        assert!(out.contains("r#\"a // not comment\\n\"#;"));
        assert!(!out.ends_with("// c"));
    }

    #[test]
    fn js_minify_preserves_template_literal() {
        let src = "const a = `x ${b}`; // c\n";
        let out = minify_js_generated(src).unwrap();
        assert!(out.contains("`x ${b}`;"));
        assert!(!out.contains("// c"));
    }
}
