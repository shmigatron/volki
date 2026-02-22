//! Server/client boundary validation — ensures APIs are used in the correct
//! function context (server vs client vs component).

use crate::core::volkiwithstds::collections::{String, Vec};
use super::scanner::{RsxFunction, RsxReturnType};

/// A single boundary violation found during validation.
pub struct BoundaryViolation {
    pub line: usize,
    pub col: usize,
    pub pattern: String,
    pub fn_type: String,
    pub fn_name: Option<String>,
    pub message: String,
    pub help: String,
}

// ── Pattern lists ──────────────────────────────────────────────────────

/// Client/browser-only APIs — forbidden in `-> Html` and `-> Fragment`.
const CLIENT_ONLY: &[(&str, &str)] = &[
    ("dom::query(", "dom::query"),
    ("dom::log(", "dom::log"),
    (".set_text(", ".set_text"),
    (".get_value(", ".get_value"),
    (".set_attr(", ".set_attr"),
    (".add_class(", ".add_class"),
    (".remove_class(", ".remove_class"),
    ("use_state(", "use_state"),
    ("state::get_i32(", "state::get_i32"),
    ("state::set_i32(", "state::set_i32"),
    ("state::get_f32(", "state::get_f32"),
    ("state::set_f32(", "state::set_f32"),
    ("state::get_str(", "state::get_str"),
    ("state::set_str(", "state::set_str"),
    ("state::fmt_i32(", "state::fmt_i32"),
    ("state::fmt_f32(", "state::fmt_f32"),
    // New DOM operations
    ("dom::create(", "dom::create"),
    ("dom::append(", "dom::append"),
    ("dom::remove(", "dom::remove"),
    ("dom::set_html(", "dom::set_html"),
    (".toggle_class(", ".toggle_class"),
    (".get_attr(", ".get_attr"),
    (".remove_attr(", ".remove_attr"),
    ("dom::query_all_count(", "dom::query_all_count"),
    ("dom::query_all_get(", "dom::query_all_get"),
    // Ref operations
    ("use_ref(", "use_ref"),
    ("use_ref_el(", "use_ref_el"),
    ("ref::get_i32(", "ref::get_i32"),
    ("ref::set_i32(", "ref::set_i32"),
    ("ref::get_f32(", "ref::get_f32"),
    ("ref::set_f32(", "ref::set_f32"),
    // Effect
    ("use_effect(", "use_effect"),
    ("use_memo_i32(", "use_memo_i32"),
    ("use_memo_f32(", "use_memo_f32"),
];

/// Server-only APIs — forbidden in `-> Client` and `-> Component`.
const SERVER_ONLY: &[(&str, &str)] = &[
    ("Response::", "Response::"),
    ("HtmlDocument::", "HtmlDocument::"),
    ("Metadata::new", "Metadata::new"),
    ("StatusCode::", "StatusCode::"),
    ("Headers::", "Headers::"),
];

/// Component-only APIs — forbidden in `-> Client`.
const COMPONENT_ONLY: &[(&str, &str)] = &[
    ("use_state(", "use_state"),
    ("use_ref(", "use_ref"),
    ("use_ref_el(", "use_ref_el"),
    ("use_effect(", "use_effect"),
    ("use_memo_i32(", "use_memo_i32"),
    ("use_memo_f32(", "use_memo_f32"),
];

// ── Public API ─────────────────────────────────────────────────────────

/// Validate that no function uses APIs from the wrong side of the boundary.
/// Called after `scan_functions()`, before parsing/codegen.
pub fn validate_boundaries(
    functions: &[RsxFunction],
    source: &str,
) -> Vec<BoundaryViolation> {
    let mut violations = Vec::new();

    for func in functions {
        let body = &source[func.body_span.0..func.body_span.1];
        let fn_name = func.name.as_ref().map(|s| s.as_str());

        match func.return_type {
            RsxReturnType::Html | RsxReturnType::Fragment => {
                let fn_type = match func.return_type {
                    RsxReturnType::Html => "Html",
                    _ => "Fragment",
                };
                scan_body(
                    body, CLIENT_ONLY, func.body_span.0, source,
                    fn_type, fn_name, ViolationKind::ClientInServer,
                    &mut violations,
                );
            }
            RsxReturnType::Client => {
                scan_body(
                    body, SERVER_ONLY, func.body_span.0, source,
                    "Client", fn_name, ViolationKind::ServerInClient,
                    &mut violations,
                );
                scan_body(
                    body, COMPONENT_ONLY, func.body_span.0, source,
                    "Client", fn_name, ViolationKind::ComponentOnlyInClient,
                    &mut violations,
                );
            }
            RsxReturnType::Component => {
                scan_body(
                    body, SERVER_ONLY, func.body_span.0, source,
                    "Component", fn_name, ViolationKind::ServerInClient,
                    &mut violations,
                );
            }
        }
    }

    violations
}

/// Validate that no top-level code (outside any function body) uses
/// client/state APIs.  These are runtime APIs that only make sense
/// inside a `-> Component` or `-> Client` function body.
pub fn validate_top_level(
    functions: &[RsxFunction],
    source: &str,
) -> Vec<BoundaryViolation> {
    let mut violations = Vec::new();
    let source_len = source.len();

    // Build sorted list of function body spans (already non-overlapping from scanner)
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for func in functions {
        // Exclude the entire function declaration, not just the body.
        // Walk back from return_type_span to find `fn`/`pub fn` start.
        let fn_start = find_fn_keyword_start(source, func.return_type_span.0);
        // body_span.1 is the byte after the closing `}` content; the `}` itself
        // is at body_span.1, so we skip past it (+1).
        let fn_end = func.body_span.1 + 1;
        let end = if fn_end < source_len { fn_end } else { source_len };
        spans.push((fn_start, end));
    }
    spans.sort();

    // Compute gap regions: [0..first_fn_start], [fn1_end..fn2_start], [last_fn_end..source_len]
    let mut gaps: Vec<(usize, usize)> = Vec::new();
    let mut cursor = 0;
    for &(start, end) in spans.iter() {
        if cursor < start {
            gaps.push((cursor, start));
        }
        if end > cursor {
            cursor = end;
        }
    }
    if cursor < source_len {
        gaps.push((cursor, source_len));
    }

    // Scan each gap region for forbidden patterns
    for &(gap_start, gap_end) in gaps.iter() {
        let region = &source[gap_start..gap_end];
        scan_body(
            region,
            CLIENT_ONLY,
            gap_start,
            source,
            "top-level",
            None,
            ViolationKind::TopLevelForbidden,
            &mut violations,
        );
    }

    violations
}

/// Walk backward from a byte position to find where `fn ` or `pub fn ` starts.
fn find_fn_keyword_start(source: &str, pos: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = pos;
    while i > 0 {
        if i >= 3 && &bytes[i - 3..i] == b"fn " {
            let fn_start = i - 3;
            if fn_start >= 4 && &bytes[fn_start - 4..fn_start] == b"pub " {
                return fn_start - 4;
            }
            return fn_start;
        }
        i -= 1;
    }
    0
}

// ── Internals ──────────────────────────────────────────────────────────

enum ViolationKind {
    ClientInServer,
    ServerInClient,
    ComponentOnlyInClient,
    TopLevelForbidden,
}

fn scan_body(
    body: &str,
    patterns: &[(&str, &str)],
    body_offset: usize,
    source: &str,
    fn_type: &str,
    fn_name: Option<&str>,
    kind: ViolationKind,
    violations: &mut Vec<BoundaryViolation>,
) {
    let bytes = body.as_bytes();
    let len = bytes.len();
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

        let mut matched = false;
        for &(pattern, display) in patterns {
            let pat = pattern.as_bytes();
            if i + pat.len() <= len && &bytes[i..i + pat.len()] == pat {
                let abs_offset = body_offset + i;
                let (line, col) = line_col_at(source, abs_offset);

                let (message, help) = build_message(display, fn_type, fn_name, &kind);

                violations.push(BoundaryViolation {
                    line,
                    col,
                    pattern: String::from(display),
                    fn_type: String::from(fn_type),
                    fn_name: fn_name.map(String::from),
                    message,
                    help,
                });
                // Skip past this match so we don't double-report the same token
                i += pat.len();
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        i += 1;
    }
}

fn build_message(
    display: &str,
    fn_type: &str,
    fn_name: Option<&str>,
    kind: &ViolationKind,
) -> (String, String) {
    let name_part = match fn_name {
        Some(n) => crate::vformat!(" `{}`", n),
        None => String::from(""),
    };

    match kind {
        ViolationKind::ClientInServer => {
            let msg = crate::vformat!(
                "client-only API `{}` used in server function{} (-> {})",
                display, name_part, fn_type
            );
            let help = crate::vformat!(
                "`{}` only works in `-> Client` or `-> Component` functions.\n           Move this code to a Client function, or use server-side alternatives.",
                display
            );
            (msg, help)
        }
        ViolationKind::ServerInClient => {
            let msg = crate::vformat!(
                "server-only API `{}` used in client function{} (-> {})",
                display, name_part, fn_type
            );
            let help = crate::vformat!(
                "`{}` only works in server functions (-> Html, -> Fragment).\n           Client functions run in the browser and cannot use server APIs.",
                display
            );
            (msg, help)
        }
        ViolationKind::ComponentOnlyInClient => {
            let msg = crate::vformat!(
                "`use_state` can only be used in `-> Component` functions, not `-> Client`"
            );
            let help = String::from(
                "`use_state` initializes component state slots and requires a Component function.\n           Change `-> Client` to `-> Component`, or use `state::get_i32`/`state::set_i32`\n           to access state from a Client function."
            );
            (msg, help)
        }
        ViolationKind::TopLevelForbidden => {
            let msg = crate::vformat!("`{}` cannot be used at the top level of a .volki file", display);
            let help = crate::vformat!(
                "`{}` is a runtime API that must be called inside a function body.\n           Move it inside a `-> Component` or `-> Client` function.",
                display
            );
            (msg, help)
        }
    }
}

/// Compute 1-based line and column from a byte offset in the source.
fn line_col_at(source: &str, byte_offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1;
    let mut col = 1;
    let end = if byte_offset <= bytes.len() { byte_offset } else { bytes.len() };

    for i in 0..end {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
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

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::web::compiler::scanner::scan_functions;

    #[test]
    fn test_client_api_in_html_detected() {
        let source = r##"
pub fn page(_req: &Request) -> Html {
    let el = dom::query("#btn");
}
"##;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern.as_str(), "dom::query");
        assert!(violations[0].message.as_str().contains("client-only API"));
        assert!(violations[0].message.as_str().contains("Html"));
    }

    #[test]
    fn test_use_state_in_html_detected() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    let count = use_state(0_i32);
}
"#;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern.as_str(), "use_state");
        assert!(violations[0].message.as_str().contains("client-only API"));
    }

    #[test]
    fn test_server_api_in_client_detected() {
        let source = r#"
pub fn on_click() -> Client {
    let r = Response::ok();
}
"#;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern.as_str(), "Response::");
        assert!(violations[0].message.as_str().contains("server-only API"));
        assert!(violations[0].message.as_str().contains("Client"));
    }

    #[test]
    fn test_use_state_in_client_detected() {
        let source = r#"
pub fn on_click() -> Client {
    let count = use_state(0_i32);
}
"#;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern.as_str(), "use_state");
        assert!(violations[0].message.as_str().contains("Component"));
        assert!(violations[0].message.as_str().contains("not `-> Client`"));
    }

    #[test]
    fn test_valid_html_no_violations() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    let title = "hello";
    let x = 42;
}
"#;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_valid_client_no_violations() {
        let source = r##"
pub fn on_click() -> Client {
    let el = dom::query("#btn");
    let v = state::get_i32("counter", 0);
}
"##;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_valid_component_no_violations() {
        let source = r##"
pub fn counter() -> Component {
    let count = use_state(0_i32);
    let el = dom::query("#count");
    el.set_text(state::fmt_i32(count));
}
"##;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_pattern_in_string_literal_ignored() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    let msg = "dom::query is a client API";
}
"#;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_pattern_in_comment_ignored() {
        let source = "
pub fn page(_req: &Request) -> Html {\n    // dom::query(\"#btn\");\n    let x = 1;\n}\n";
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_violations_collected() {
        let source = r##"
pub fn page(_req: &Request) -> Html {
    let el = dom::query("#btn");
    el.set_text("hi");
    dom::log("debug");
}
"##;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn test_server_api_in_component_detected() {
        let source = r#"
pub fn counter() -> Component {
    let doc = HtmlDocument::new();
}
"#;
        let fns = scan_functions(source);
        let violations = validate_boundaries(&fns, source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern.as_str(), "HtmlDocument::");
        assert!(violations[0].message.as_str().contains("server-only API"));
        assert!(violations[0].message.as_str().contains("Component"));
    }

    // ── Top-level validation tests ──────────────────────────────────

    #[test]
    fn test_top_level_use_state_detected() {
        let source = r#"
let count = use_state(0_i32);

pub fn page(_req: &Request) -> Html {
    let x = 1;
}
"#;
        let fns = scan_functions(source);
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern.as_str(), "use_state");
        assert!(violations[0].message.as_str().contains("cannot be used at the top level"));
    }

    #[test]
    fn test_top_level_state_set_detected() {
        let source = r#"
state::set_i32("counter", 0, 42);

pub fn page(_req: &Request) -> Html {
    let x = 1;
}
"#;
        let fns = scan_functions(source);
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern.as_str(), "state::set_i32");
        assert!(violations[0].message.as_str().contains("cannot be used at the top level"));
    }

    #[test]
    fn test_top_level_dom_query_detected() {
        let source = r##"
let el = dom::query("#btn");

pub fn on_click() -> Client {
    dom::log("ok");
}
"##;
        let fns = scan_functions(source);
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern.as_str(), "dom::query");
    }

    #[test]
    fn test_top_level_between_functions_detected() {
        let source = r##"
pub fn page(_req: &Request) -> Html {
    let x = 1;
}

let el = dom::query("#middle");

pub fn on_click() -> Client {
    dom::log("ok");
}
"##;
        let fns = scan_functions(source);
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern.as_str(), "dom::query");
    }

    #[test]
    fn test_top_level_no_false_positives_inside_functions() {
        let source = r##"
pub fn on_click() -> Client {
    let el = dom::query("#btn");
    state::set_i32("counter", 0, 1);
}

pub fn counter() -> Component {
    let count = use_state(0_i32);
}
"##;
        let fns = scan_functions(source);
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_top_level_string_literal_ignored() {
        let source = r#"
let msg = "use_state(0_i32) is for components";

pub fn page(_req: &Request) -> Html {
    let x = 1;
}
"#;
        let fns = scan_functions(source);
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_top_level_comment_ignored() {
        let source = "
// use_state(0_i32);\n/* dom::query(\"#x\"); */\n\npub fn page(_req: &Request) -> Html {\n    let x = 1;\n}\n";
        let fns = scan_functions(source);
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_top_level_multiple_violations() {
        let source = r##"
let el = dom::query("#a");
let count = use_state(0_i32);
state::set_i32("x", 0, 1);

pub fn page(_req: &Request) -> Html {
    let x = 1;
}
"##;
        let fns = scan_functions(source);
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn test_top_level_no_functions_entire_file() {
        let source = r##"
let el = dom::query("#btn");
let count = use_state(0_i32);
"##;
        let fns = scan_functions(source);
        assert!(fns.is_empty());
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern.as_str(), "dom::query");
        assert_eq!(violations[1].pattern.as_str(), "use_state");
    }

    #[test]
    fn test_top_level_help_message() {
        let source = r#"
let count = use_state(0_i32);
"#;
        let fns = scan_functions(source);
        let violations = validate_top_level(&fns, source);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].help.as_str().contains("runtime API"));
        assert!(violations[0].help.as_str().contains("Component"));
    }
}
