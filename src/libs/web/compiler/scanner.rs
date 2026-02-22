//! Scanner — finds function bodies (-> Html / -> Fragment / -> Client) in source files.

use crate::core::volkiwithstds::collections::{String, Vec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsxReturnType {
    Html,
    Fragment,
    Client,
    Component,
}

/// A parameter extracted from a function signature: `(name, type)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnParam {
    pub name: String,
    pub ty: String,
}

#[derive(Debug)]
pub struct RsxFunction {
    pub return_type: RsxReturnType,
    /// Byte range of "Html", "Fragment", "Client", or "Component" in the source.
    pub return_type_span: (usize, usize),
    /// Byte range of the body content (inside the outermost braces).
    pub body_span: (usize, usize),
    /// Function name (extracted for all function types).
    pub name: Option<String>,
    /// Function parameters (extracted for Client/Component/Fragment functions, empty for Html).
    pub params: Vec<FnParam>,
}

/// Result of splitting a Component body into logic and RSX sections.
#[derive(Debug)]
pub struct ComponentBodySplit {
    /// Byte range of the logic section (before `return`).
    pub logic_span: (usize, usize),
    /// Byte range of the RSX content (inside parens after `return`).
    pub rsx_span: (usize, usize),
}

/// Split a Component function body into logic (before `return`) and RSX (inside `return (...)`).
///
/// Returns `None` if no `return (RSX)` is found (backward compat: imperative Component).
pub fn split_component_body(source: &str, body_span: (usize, usize)) -> Option<ComponentBodySplit> {
    let body = &source[body_span.0..body_span.1];
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut brace_depth: i32 = 0;

    while i < len {
        // Skip string literals
        if bytes[i] == b'"' {
            i = skip_string(bytes, i);
            continue;
        }
        // Skip line comments
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            i = skip_line_comment(bytes, i);
            continue;
        }
        // Skip block comments
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i = skip_block_comment(bytes, i);
            continue;
        }

        // Track brace depth (we want `return` at depth 0)
        if bytes[i] == b'{' { brace_depth += 1; }
        if bytes[i] == b'}' { brace_depth -= 1; }

        // Look for `return` at brace depth 0
        if brace_depth == 0 && i + 6 <= len && &bytes[i..i + 6] == b"return" {
            // Ensure it's a keyword boundary
            if i > 0 && is_ident_char(bytes[i - 1]) {
                i += 1;
                continue;
            }
            if i + 6 < len && is_ident_char(bytes[i + 6]) {
                i += 1;
                continue;
            }

            // Skip whitespace after `return`
            let mut j = i + 6;
            while j < len && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n' || bytes[j] == b'\r') {
                j += 1;
            }

            // Check for `(`
            if j < len && bytes[j] == b'(' {
                if let Some(close) = find_matching_paren(bytes, j) {
                    return Some(ComponentBodySplit {
                        logic_span: (body_span.0, body_span.0 + i),
                        rsx_span: (body_span.0 + j + 1, body_span.0 + close),
                    });
                }
            }
        }

        i += 1;
    }

    None
}

/// Find the matching closing paren for an opening paren at `start`.
/// Handles nested parens, strings, and comments.
fn find_matching_paren(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 1;
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i = skip_string(bytes, i);
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                i = skip_line_comment(bytes, i);
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i = skip_block_comment(bytes, i);
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Scan a source file for functions returning `-> Html`, `-> Fragment`, or `-> Client`.
/// Returns the list of functions found.
pub fn scan_functions(source: &str) -> Vec<RsxFunction> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut results = Vec::new();
    let mut i = 0;

    while i < len {
        // Skip string literals
        if bytes[i] == b'"' {
            i = skip_string(bytes, i);
            continue;
        }
        // Skip line comments
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            i = skip_line_comment(bytes, i);
            continue;
        }
        // Skip block comments
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i = skip_block_comment(bytes, i);
            continue;
        }

        // Look for "->" pattern
        if i + 1 < len && bytes[i] == b'-' && bytes[i + 1] == b'>' {
            let arrow_start = i;
            let arrow_end = i + 2;
            let ws_end = skip_whitespace(bytes, arrow_end);

            // Check for "Html", "Fragment", or "Client"
            if let Some((ret_type, ret_end)) = match_return_type(bytes, ws_end) {
                // Find the opening brace of the function body
                let brace_start = skip_whitespace(bytes, ret_end);
                if brace_start < len && bytes[brace_start] == b'{' {
                    // Find matching closing brace
                    if let Some(brace_end) = find_matching_brace(bytes, brace_start) {
                        // Extract name and params from the function signature
                        let (name, params) = if ret_type == RsxReturnType::Client
                            || ret_type == RsxReturnType::Component
                            || ret_type == RsxReturnType::Fragment {
                            extract_fn_signature(source, arrow_start)
                        } else {
                            (extract_fn_name_only(source, arrow_start), Vec::new())
                        };

                        results.push(RsxFunction {
                            return_type: ret_type,
                            return_type_span: (ws_end, ret_end),
                            body_span: (brace_start + 1, brace_end),
                            name,
                            params,
                        });
                        i = brace_end + 1;
                        continue;
                    }
                }
            }
        }

        i += 1;
    }

    results
}

fn match_return_type(bytes: &[u8], pos: usize) -> Option<(RsxReturnType, usize)> {
    let len = bytes.len();
    if pos + 4 <= len && &bytes[pos..pos + 4] == b"Html" {
        // Make sure it's not "HtmlDocument" or another identifier
        if pos + 4 >= len || !is_ident_char(bytes[pos + 4]) {
            return Some((RsxReturnType::Html, pos + 4));
        }
    }
    if pos + 8 <= len && &bytes[pos..pos + 8] == b"Fragment" {
        if pos + 8 >= len || !is_ident_char(bytes[pos + 8]) {
            return Some((RsxReturnType::Fragment, pos + 8));
        }
    }
    if pos + 6 <= len && &bytes[pos..pos + 6] == b"Client" {
        if pos + 6 >= len || !is_ident_char(bytes[pos + 6]) {
            return Some((RsxReturnType::Client, pos + 6));
        }
    }
    if pos + 9 <= len && &bytes[pos..pos + 9] == b"Component" {
        if pos + 9 >= len || !is_ident_char(bytes[pos + 9]) {
            return Some((RsxReturnType::Component, pos + 9));
        }
    }
    None
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn skip_whitespace(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t' || bytes[pos] == b'\n' || bytes[pos] == b'\r') {
        pos += 1;
    }
    pos
}

fn skip_string(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return i + 1;
        }
        i += 1;
    }
    i
}

fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 2;
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    if i < bytes.len() { i + 1 } else { i }
}

fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 2;
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

/// Find the matching closing brace for an opening brace at `start`.
/// Handles nested braces, strings, and comments.
fn find_matching_brace(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 1;
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i = skip_string(bytes, i);
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                i = skip_line_comment(bytes, i);
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i = skip_block_comment(bytes, i);
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Walk backward from the `->` arrow to extract the function name and parameter list.
///
/// Expects something like: `pub fn on_click(target: &str) ->` before the arrow.
/// Returns `(Some(name), params)` or `(None, [])` if extraction fails.
fn extract_fn_signature(source: &str, arrow_pos: usize) -> (Option<String>, Vec<FnParam>) {
    let before = &source[..arrow_pos];
    let bytes = before.as_bytes();

    // Walk backward past whitespace to find the closing paren ')'
    let mut pos = bytes.len();
    while pos > 0 && (bytes[pos - 1] == b' ' || bytes[pos - 1] == b'\t' || bytes[pos - 1] == b'\n' || bytes[pos - 1] == b'\r') {
        pos -= 1;
    }

    if pos == 0 || bytes[pos - 1] != b')' {
        return (None, Vec::new());
    }

    // Find matching opening paren
    let close_paren = pos - 1;
    let mut depth = 1;
    let mut open_paren = close_paren;
    while open_paren > 0 {
        open_paren -= 1;
        match bytes[open_paren] {
            b')' => depth += 1,
            b'(' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
    }

    if depth != 0 {
        return (None, Vec::new());
    }

    // Extract params string between parens
    let params_str = &source[open_paren + 1..close_paren];
    let params = parse_params(params_str);

    // Walk backward from open_paren to find the function name
    let mut name_end = open_paren;
    while name_end > 0 && (bytes[name_end - 1] == b' ' || bytes[name_end - 1] == b'\t') {
        name_end -= 1;
    }

    let mut name_start = name_end;
    while name_start > 0 && is_ident_char(bytes[name_start - 1]) {
        name_start -= 1;
    }

    if name_start == name_end {
        return (None, params);
    }

    let name = String::from(&source[name_start..name_end]);
    (Some(name), params)
}

/// Walk backward from the `->` arrow to extract just the function name.
/// Used for Html/Fragment functions where we don't need params.
fn extract_fn_name_only(source: &str, arrow_pos: usize) -> Option<String> {
    let before = &source[..arrow_pos];
    let bytes = before.as_bytes();

    // Walk backward past whitespace to find ')'
    let mut pos = bytes.len();
    while pos > 0 && (bytes[pos - 1] == b' ' || bytes[pos - 1] == b'\t' || bytes[pos - 1] == b'\n' || bytes[pos - 1] == b'\r') {
        pos -= 1;
    }

    if pos == 0 || bytes[pos - 1] != b')' {
        return None;
    }

    // Find matching '('
    let mut depth = 1;
    let mut open_paren = pos - 1;
    while open_paren > 0 {
        open_paren -= 1;
        match bytes[open_paren] {
            b')' => depth += 1,
            b'(' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
    }

    if depth != 0 {
        return None;
    }

    // Walk backward from open_paren past whitespace to find the name
    let mut name_end = open_paren;
    while name_end > 0 && (bytes[name_end - 1] == b' ' || bytes[name_end - 1] == b'\t') {
        name_end -= 1;
    }

    let mut name_start = name_end;
    while name_start > 0 && is_ident_char(bytes[name_start - 1]) {
        name_start -= 1;
    }

    if name_start == name_end {
        return None;
    }

    Some(String::from(&source[name_start..name_end]))
}

/// Parse a parameter list like `target: &str, count: i32` into `Vec<FnParam>`.
fn parse_params(params_str: &str) -> Vec<FnParam> {
    let mut result = Vec::new();
    if params_str.trim().is_empty() {
        return result;
    }

    for part in params_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // Split on first ':'
        if let Some(colon_pos) = part.find(':') {
            let name = part[..colon_pos].trim();
            let ty = part[colon_pos + 1..].trim();
            if !name.is_empty() && !ty.is_empty() {
                result.push(FnParam {
                    name: String::from(name),
                    ty: String::from(ty),
                });
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_html_function() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div class="main">"hello"</div>
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].return_type, RsxReturnType::Html);
        let rt = &source[fns[0].return_type_span.0..fns[0].return_type_span.1];
        assert_eq!(rt, "Html");
        assert_eq!(fns[0].name.as_ref().unwrap().as_str(), "page");
        assert!(fns[0].params.is_empty());
    }

    #[test]
    fn test_scan_fragment_function() {
        let source = r#"
fn sidebar() -> Fragment {
    <div>"sidebar"</div>
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].return_type, RsxReturnType::Fragment);
        let rt = &source[fns[0].return_type_span.0..fns[0].return_type_span.1];
        assert_eq!(rt, "Fragment");
        assert_eq!(fns[0].name.as_ref().unwrap().as_str(), "sidebar");
    }

    #[test]
    fn test_scan_client_function() {
        let source = r#"
pub fn on_click(target: &str) -> Client {
    let el = dom::query(target);
    el.set_text("Clicked!");
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].return_type, RsxReturnType::Client);
        let rt = &source[fns[0].return_type_span.0..fns[0].return_type_span.1];
        assert_eq!(rt, "Client");
        assert_eq!(fns[0].name.as_ref().unwrap().as_str(), "on_click");
        assert_eq!(fns[0].params.len(), 1);
        assert_eq!(fns[0].params[0].name.as_str(), "target");
        assert_eq!(fns[0].params[0].ty.as_str(), "&str");
    }

    #[test]
    fn test_scan_client_multiple_params() {
        let source = r#"
pub fn update_field(id: i32, value: &str, flag: bool) -> Client {
    dom::log("updating");
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].return_type, RsxReturnType::Client);
        assert_eq!(fns[0].name.as_ref().unwrap().as_str(), "update_field");
        assert_eq!(fns[0].params.len(), 3);
        assert_eq!(fns[0].params[0].name.as_str(), "id");
        assert_eq!(fns[0].params[0].ty.as_str(), "i32");
        assert_eq!(fns[0].params[1].name.as_str(), "value");
        assert_eq!(fns[0].params[1].ty.as_str(), "&str");
        assert_eq!(fns[0].params[2].name.as_str(), "flag");
        assert_eq!(fns[0].params[2].ty.as_str(), "bool");
    }

    #[test]
    fn test_scan_client_no_params() {
        let source = r#"
pub fn init() -> Client {
    dom::log("initialized");
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].return_type, RsxReturnType::Client);
        assert_eq!(fns[0].name.as_ref().unwrap().as_str(), "init");
        assert!(fns[0].params.is_empty());
    }

    #[test]
    fn test_scan_mixed_functions() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <button onclick="__volki.on_click('greeting')">"Click me"</button>
    <p id="greeting">"Hello"</p>
}

pub fn on_click(target: &str) -> Client {
    let el = dom::query(target);
    el.set_text("Clicked!");
}

fn sidebar() -> Fragment {
    <div>"side"</div>
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 3);
        assert_eq!(fns[0].return_type, RsxReturnType::Html);
        assert_eq!(fns[0].name.as_ref().unwrap().as_str(), "page");
        assert_eq!(fns[1].return_type, RsxReturnType::Client);
        assert_eq!(fns[1].name.as_ref().unwrap().as_str(), "on_click");
        assert_eq!(fns[2].return_type, RsxReturnType::Fragment);
        assert_eq!(fns[2].name.as_ref().unwrap().as_str(), "sidebar");
    }

    #[test]
    fn test_scan_ignores_regular_functions() {
        let source = r#"
pub fn metadata(_req: &Request) -> Metadata {
    Metadata::new().title("test")
}

pub fn handler(_req: &Request) -> Response {
    Response::ok()
}

pub fn get_name() -> String {
    String::from("hello")
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 0);
    }

    #[test]
    fn test_scan_body_span() {
        let source = "fn page() -> Html {\n    <div>\"hi\"</div>\n}";
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        let body = &source[fns[0].body_span.0..fns[0].body_span.1];
        assert!(body.contains("<div>"));
        assert!(body.contains("\"hi\""));
        // Body should NOT include the outer braces
        assert!(!body.starts_with("{"));
        assert!(!body.ends_with("}"));
    }

    #[test]
    fn test_scan_multiple_functions() {
        let source = r#"
fn page() -> Html {
    <div>"hello"</div>
}

fn sidebar() -> Fragment {
    <span>"side"</span>
}

fn handler() -> Response {
    Response::ok()
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].return_type, RsxReturnType::Html);
        assert_eq!(fns[1].return_type, RsxReturnType::Fragment);
    }

    #[test]
    fn test_scan_ignores_html_document() {
        let source = r#"
fn page() -> HtmlDocument {
    HtmlDocument::new()
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 0);
    }

    #[test]
    fn test_scan_client_body_is_raw_rust() {
        let source = r#"
pub fn toggle(el_id: &str) -> Client {
    let el = dom::query(el_id);
    el.add_class("active");
    dom::log("toggled");
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        let body = &source[fns[0].body_span.0..fns[0].body_span.1];
        assert!(body.contains("dom::query"));
        assert!(body.contains("el.add_class"));
        assert!(body.contains("dom::log"));
    }

    #[test]
    fn test_scan_component_function() {
        let source = r#"
pub fn counter() -> Component {
    let count = use_state(0_i32);
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].return_type, RsxReturnType::Component);
        let rt = &source[fns[0].return_type_span.0..fns[0].return_type_span.1];
        assert_eq!(rt, "Component");
        assert_eq!(fns[0].name.as_ref().unwrap().as_str(), "counter");
        assert!(fns[0].params.is_empty());
    }

    #[test]
    fn test_scan_mixed_with_component() {
        let source = r##"
pub fn page(_req: &Request) -> Html {
    <p id="count">"0"</p>
}

pub fn counter() -> Component {
    let count = use_state(0_i32);
    let el = dom::query("#count");
}

pub fn on_increment() -> Client {
    let count = state::get_i32("counter", 0);
}
"##;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 3);
        assert_eq!(fns[0].return_type, RsxReturnType::Html);
        assert_eq!(fns[1].return_type, RsxReturnType::Component);
        assert_eq!(fns[1].name.as_ref().unwrap().as_str(), "counter");
        assert_eq!(fns[2].return_type, RsxReturnType::Client);
        assert_eq!(fns[2].name.as_ref().unwrap().as_str(), "on_increment");
    }

    #[test]
    fn test_scan_fragment_function_with_params() {
        let source = r#"
fn counter_section(show_help: bool, dark: bool) -> Fragment {
    <div>"counter"</div>
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].return_type, RsxReturnType::Fragment);
        assert_eq!(fns[0].name.as_ref().unwrap().as_str(), "counter_section");
        assert_eq!(fns[0].params.len(), 2);
        assert_eq!(fns[0].params[0].name.as_str(), "show_help");
        assert_eq!(fns[0].params[0].ty.as_str(), "bool");
        assert_eq!(fns[0].params[1].name.as_str(), "dark");
        assert_eq!(fns[0].params[1].ty.as_str(), "bool");
    }

    #[test]
    fn test_scan_ignores_component_builder() {
        let source = r#"
fn build() -> ComponentBuilder {
    ComponentBuilder::new()
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 0);
    }

    // ── split_component_body tests ──

    #[test]
    fn test_split_component_body_with_return_rsx() {
        let source = r#"
pub fn counter() -> Component {
    let (count, set_count) = use_state(0_i32);
    let _ = set_count;

    return (
        <div class="counter">
            <span>"hello"</span>
        </div>
    )
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        let split = split_component_body(source, fns[0].body_span);
        assert!(split.is_some());
        let split = split.unwrap();

        let logic = &source[split.logic_span.0..split.logic_span.1];
        assert!(logic.contains("use_state(0_i32)"));
        assert!(logic.contains("let _ = set_count"));
        assert!(!logic.contains("<div"));

        let rsx = &source[split.rsx_span.0..split.rsx_span.1];
        assert!(rsx.contains("<div"));
        assert!(rsx.contains("<span>"));
        assert!(!rsx.contains("use_state"));
    }

    #[test]
    fn test_split_component_body_imperative_no_return() {
        let source = r##"
pub fn counter() -> Component {
    let count = use_state(0_i32);
    let el = dom::query("#count");
    el.set_text(state::fmt_i32(count));
}
"##;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        let split = split_component_body(source, fns[0].body_span);
        assert!(split.is_none());
    }

    #[test]
    fn test_split_component_body_return_in_string_ignored() {
        let source = r##"
pub fn counter() -> Component {
    let msg = "return (this should not match)";
    let el = dom::query("#count");
}
"##;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        let split = split_component_body(source, fns[0].body_span);
        assert!(split.is_none());
    }

    #[test]
    fn test_split_component_body_return_in_nested_block_ignored() {
        let source = r#"
pub fn counter() -> Component {
    let count = use_state(0_i32);
    if count > 0 {
        return (
            <div>"should not match"</div>
        )
    }
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        let split = split_component_body(source, fns[0].body_span);
        // return inside a nested block (brace_depth > 0) should not match
        assert!(split.is_none());
    }

    #[test]
    fn test_split_component_body_empty_logic() {
        let source = r#"
pub fn greeting() -> Component {
    return (
        <div>"hello"</div>
    )
}
"#;
        let fns = scan_functions(source);
        assert_eq!(fns.len(), 1);
        let split = split_component_body(source, fns[0].body_span).unwrap();

        let logic = &source[split.logic_span.0..split.logic_span.1];
        assert!(logic.trim().is_empty() || logic.trim() == "\n");

        let rsx = &source[split.rsx_span.0..split.rsx_span.1];
        assert!(rsx.contains("<div>"));
    }
}
