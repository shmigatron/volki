//! Dev-mode route scanner — discovers `.volki` pages and parses them at runtime.
//!
//! Reuses the compiler's route discovery conventions (page.volki, not_found.volki)
//! but instead of generating Rust code, parses the RSX AST and wraps it in
//! `DynamicPageData` for runtime interpretation.

use crate::core::volkiwithstds::collections::{HashMap, String, Vec};
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::{Path, PathBuf};
use crate::core::volkiwithstds::sync::Arc;
use crate::libs::web::compiler::js_codegen;
use crate::libs::web::compiler::minify;
use crate::libs::web::compiler::scanner::{scan_functions, RsxFunction, RsxReturnType};
use crate::libs::web::compiler::tokenizer;
use crate::libs::web::compiler::wasm_build;
use crate::libs::web::compiler::wasm_codegen;
use crate::libs::web::compiler::parser;
use crate::libs::web::volkistyle;
use super::{DynamicPageData, extract_metadata};

/// The kind of dynamic route discovered.
pub enum DynamicRouteKind {
    Page,
    NotFound,
}

/// A route discovered from the filesystem, ready for dynamic serving.
pub struct DynamicRoute {
    pub kind: DynamicRouteKind,
    pub url_path: String,
    pub data: Arc<DynamicPageData>,
}

/// Discover all `.volki` page routes under a web app's source directory.
///
/// Scans for `page.volki` and `not_found.volki` files, parses their RSX,
/// collects CSS classes, extracts metadata, and returns ready-to-serve routes.
pub fn discover_dynamic_routes(root: &Path) -> Result<Vec<DynamicRoute>, String> {
    let app_dir = root.join("app");
    if !app_dir.as_path().exists() {
        return Ok(Vec::new());
    }
    let mut routes = Vec::new();
    scan_dir(app_dir.as_path(), root, &mut routes)?;
    Ok(routes)
}

fn scan_dir(
    dir: &Path,
    root: &Path,
    routes: &mut Vec<DynamicRoute>,
) -> Result<(), String> {
    // Page route: page.volki
    let page_volki = dir.join("page.volki");
    if page_volki.as_path().exists() {
        let url = dir_to_url(dir, root);
        if let Some(data) = parse_volki_file(page_volki.as_path(), root)? {
            routes.push(DynamicRoute {
                kind: DynamicRouteKind::Page,
                url_path: url,
                data: Arc::new(data),
            });
        }
    }

    // Not-found handler: not_found.volki
    let nf_volki = dir.join("not_found.volki");
    if nf_volki.as_path().exists() {
        if let Some(data) = parse_volki_file(nf_volki.as_path(), root)? {
            routes.push(DynamicRoute {
                kind: DynamicRouteKind::NotFound,
                url_path: String::new(),
                data: Arc::new(data),
            });
        }
    }

    // Recurse into subdirectories
    let entries = fs::read_dir(dir).map_err(|e| {
        crate::vformat!("failed to read directory {}: {}", dir, e)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            crate::vformat!("failed to read entry in {}: {}", dir, e)
        })?;
        if entry.file_type() == fs::FileType::Directory {
            scan_dir(entry.path(), root, routes)?;
        }
    }

    Ok(())
}

/// Parse a single `.volki` file into DynamicPageData.
///
/// Scans for Html/Fragment/metadata functions, parses RSX bodies,
/// collects CSS classes, and extracts metadata.
fn parse_volki_file(path: &Path, root: &Path) -> Result<Option<DynamicPageData>, String> {
    let source = fs::read_to_string(path).map_err(|e| {
        crate::vformat!("failed to read {}: {}", path, e)
    })?;

    let functions = scan_functions(source.as_str());
    if functions.is_empty() {
        return Ok(None);
    }

    let client_fns: Vec<&RsxFunction> = functions
        .iter()
        .filter(|f| f.return_type == RsxReturnType::Client)
        .collect();
    let component_fns: Vec<&RsxFunction> = functions
        .iter()
        .filter(|f| f.return_type == RsxReturnType::Component)
        .collect();

    let client_glue_url = if !client_fns.is_empty() || !component_fns.is_empty() {
        Some(generate_dynamic_client_assets(
            path,
            root,
            source.as_str(),
            &client_fns,
            &component_fns,
        )?)
    } else {
        None
    };

    let file_buf = PathBuf::from(path.as_str());
    let mut html_nodes = Vec::new();
    let mut fragments = HashMap::new();
    let mut all_classes = Vec::new();
    let mut metadata = None;

    for func in &functions {
        let body = &source.as_str()[func.body_span.0..func.body_span.1];

        match func.return_type {
            RsxReturnType::Html => {
                let tokens = tokenizer::tokenize(body.trim(), file_buf.clone())
                    .map_err(|e| crate::vformat!("tokenize error in {}: {}", path, e))?;
                let nodes = parser::parse(&tokens, file_buf.clone())
                    .map_err(|e| crate::vformat!("parse error in {}: {}", path, e))?;

                let fn_classes = volkistyle::collector::collect_classes(&nodes);
                for c in fn_classes.iter() {
                    all_classes.push(c.clone());
                }

                html_nodes = nodes;
            }
            RsxReturnType::Fragment => {
                let tokens = tokenizer::tokenize(body.trim(), file_buf.clone())
                    .map_err(|e| crate::vformat!("tokenize error in {}: {}", path, e))?;
                let nodes = parser::parse(&tokens, file_buf.clone())
                    .map_err(|e| crate::vformat!("parse error in {}: {}", path, e))?;

                let fn_classes = volkistyle::collector::collect_classes(&nodes);
                for c in fn_classes.iter() {
                    all_classes.push(c.clone());
                }

                // Extract function name by scanning backward from the arrow
                if let Some(ref name) = func.name {
                    fragments.insert(name.clone(), nodes);
                } else {
                    // Try to extract name from source before the return type span
                    let before = &source.as_str()[..func.return_type_span.0];
                    if let Some(name) = extract_fn_name_before_arrow(before) {
                        fragments.insert(name, nodes);
                    }
                }
            }
            RsxReturnType::Client | RsxReturnType::Component => {
                // Client/Component functions can't be interpreted — skip
            }
        }
    }

    // Resolve imported Fragment functions (one hop) so `{foo()}` from `use ...::foo`
    // renders instead of falling back to placeholders.
    load_imported_fragments(
        source.as_str(),
        path,
        &mut fragments,
        &mut all_classes,
    )?;

    // Check for metadata function (not an RsxReturnType, so scan source directly)
    if let Some(meta_start) = source.as_str().find("fn metadata") {
        let after = &source.as_str()[meta_start..];
        if let Some(brace_pos) = after.find('{') {
            let after_brace = &after[brace_pos + 1..];
            if let Some(end_brace) = find_matching_brace_simple(after_brace) {
                let meta_body = &after_brace[..end_brace];
                metadata = extract_metadata(meta_body);
            }
        }
    }

    let style_cfg = volkistyle::config::load_for_source_file(path);
    let style_report = volkistyle::generate_css_with_config(&all_classes, &style_cfg);
    if style_cfg.unknown_class_policy == volkistyle::config::UnknownClassPolicy::Error
        && !style_report.diagnostics.is_empty()
    {
        let first = &style_report.diagnostics[0];
        let (line, col) = find_class_occurrence(source.as_str(), first.class_name.as_str()).unwrap_or((0, 0));
        return Err(crate::vformat!(
            "style error: {} ({})",
            first.message,
            crate::core::cli::format_trace(path.as_str(), line, col)
        ));
    }
    for d in style_report.diagnostics.iter() {
        let (line, col) = find_class_occurrence(source.as_str(), d.class_name.as_str()).unwrap_or((0, 0));
        crate::core::cli::print_warn_trace(path.as_str(), line, col, d.message.as_str());
    }
    let css = style_report.css;

    Ok(Some(DynamicPageData {
        nodes: html_nodes,
        css,
        fragments,
        metadata,
        client_glue_url,
    }))
}

fn generate_dynamic_client_assets(
    source_file: &Path,
    source_root: &Path,
    source: &str,
    client_fns: &[&RsxFunction],
    component_fns: &[&RsxFunction],
) -> Result<String, String> {
    let wasm_dir = source_root.join(".volki").join("public").join("wasm");
    fs::create_dir_all(wasm_dir.as_path()).map_err(|e| {
        crate::vformat!("failed to create wasm output directory {}: {}", wasm_dir, e)
    })?;

    let stem = dynamic_asset_stem(source_file, source_root);
    let client_rs_name = crate::vformat!("{}_client.rs", stem);
    let wasm_name = crate::vformat!("{}_client.wasm", stem);
    let glue_name = crate::vformat!("{}_glue.js", stem);

    let client_rs_path = wasm_dir.join(client_rs_name.as_str());
    let wasm_path = wasm_dir.join(wasm_name.as_str());
    let glue_path = wasm_dir.join(glue_name.as_str());

    let empty_rsx: Vec<Option<Vec<parser::RsxNode>>> =
        component_fns.iter().map(|_| None).collect();
    let wasm_rs_raw = wasm_codegen::generate_wasm_module(client_fns, component_fns, source, &empty_rsx);
    let wasm_rs = match minify::minify_rust_generated(wasm_rs_raw.as_str()) {
        Ok(s) => s,
        Err(e) => {
            crate::core::cli::print_warn_trace(
                source_file.as_str(),
                e.line,
                e.col,
                crate::vformat!("minify fallback (dynamic wasm_rs): {}", e).as_str(),
            );
            wasm_rs_raw
        }
    };
    fs::write_str(client_rs_path.as_path(), wasm_rs.as_str()).map_err(|e| {
        crate::vformat!("failed to write wasm client source {}: {}", client_rs_path, e)
    })?;

    wasm_build::compile_wasm(client_rs_path.as_path(), wasm_path.as_path()).map_err(|e| {
        let mapped = map_wasm_compile_error_to_volki(source_file, source, e.message.as_str());
        crate::vformat!("failed to compile dynamic wasm for {}: {}", source_file, mapped)
    })?;

    let wasm_url = crate::vformat!("/wasm/{}", wasm_name);
    let glue_js_raw = js_codegen::generate_js_glue(client_fns, component_fns, source, wasm_url.as_str(), false);
    let glue_js = match minify::minify_js_generated(glue_js_raw.as_str()) {
        Ok(s) => s,
        Err(e) => {
            crate::core::cli::print_warn_trace(
                source_file.as_str(),
                e.line,
                e.col,
                crate::vformat!("minify fallback (dynamic glue_js): {}", e).as_str(),
            );
            glue_js_raw
        }
    };
    fs::write_str(glue_path.as_path(), glue_js.as_str()).map_err(|e| {
        crate::vformat!("failed to write wasm glue JS {}: {}", glue_path, e)
    })?;

    Ok(crate::vformat!("/wasm/{}", glue_name))
}

fn dynamic_asset_stem(source_file: &Path, source_root: &Path) -> String {
    let rel = source_file
        .strip_prefix(source_root.as_str())
        .unwrap_or(source_file.as_str());
    let base = if let Some(dot_pos) = rel.rfind('.') {
        &rel[..dot_pos]
    } else {
        rel
    };

    let mut out = String::from("dyn_");
    let mut last_was_sep = false;
    for &b in base.as_bytes() {
        let is_alnum = b.is_ascii_alphanumeric();
        if is_alnum {
            out.push((b as char).to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    while out.ends_with("_") {
        let new_len = out.len().saturating_sub(1);
        out = String::from(&out.as_str()[..new_len]);
    }
    if out.as_str() == "dyn" || out.as_str().is_empty() {
        out.push_str("page");
    }
    out
}

fn map_wasm_compile_error_to_volki(source_file: &Path, source: &str, message: &str) -> String {
    let mut primary_error: Option<&str> = None;
    let mut snippet: Option<&str> = None;

    for raw in message.lines() {
        let line = raw.trim();
        if line.starts_with("error[") || line.starts_with("error:") {
            if primary_error.is_none() {
                primary_error = Some(line);
            }
        }

        // rustc snippet line format: "79 | let term = input.get_value();"
        if let Some(pipe_idx) = line.find('|') {
            let lhs = line[..pipe_idx].trim();
            if !lhs.is_empty() && lhs.as_bytes().iter().all(|b| b.is_ascii_digit()) {
                let rhs = line[pipe_idx + 1..].trim();
                if !rhs.is_empty() {
                    snippet = Some(rhs);
                    break;
                }
            }
        }
    }

    if let Some(code) = snippet {
        for (idx, src_line) in source.lines().enumerate() {
            if src_line.trim() == code {
                let line_no = idx + 1;
                let col = src_line.find(code).map(|c| c + 1).unwrap_or(1);
                let msg = primary_error.unwrap_or("error: dynamic wasm compile failed");
                return crate::vformat!("{}:{}:{}: {}", source_file, line_no, col, msg);
            }
        }
    }

    String::from(message)
}

fn find_class_occurrence(source: &str, class_name: &str) -> Option<(usize, usize)> {
    let idx = source.find(class_name)?;
    let mut line = 1usize;
    let mut col = 1usize;
    for b in source.as_bytes().iter().take(idx) {
        if *b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    Some((line, col))
}

fn load_imported_fragments(
    source: &str,
    source_file: &Path,
    fragments: &mut HashMap<String, Vec<parser::RsxNode>>,
    all_classes: &mut Vec<String>,
) -> Result<(), String> {
    let imports = parse_use_imports(source);
    if imports.is_empty() {
        return Ok(());
    }

    for (module_segments, symbols) in imports {
        let Some(module_file) = resolve_module_file(source_file, &module_segments) else {
            continue;
        };
        let module_source = fs::read_to_string(module_file.as_path()).map_err(|e| {
            crate::vformat!("failed to read imported module {}: {}", module_file, e)
        })?;
        let module_functions = scan_functions(module_source.as_str());
        let module_buf = PathBuf::from(module_file.as_str());

        for func in &module_functions {
            if func.return_type != RsxReturnType::Fragment {
                continue;
            }
            let Some(name) = &func.name else { continue };
            if !symbols.iter().any(|s| s.as_str() == name.as_str()) {
                continue;
            }
            if fragments.contains_key(name) {
                continue;
            }

            let body = &module_source.as_str()[func.body_span.0..func.body_span.1];
            let tokens = tokenizer::tokenize(body.trim(), module_buf.clone())
                .map_err(|e| crate::vformat!("tokenize error in {}: {}", module_file, e))?;
            let nodes = parser::parse(&tokens, module_buf.clone())
                .map_err(|e| crate::vformat!("parse error in {}: {}", module_file, e))?;

            let fn_classes = volkistyle::collector::collect_classes(&nodes);
            for c in fn_classes.iter() {
                all_classes.push(c.clone());
            }

            fragments.insert(name.clone(), nodes);
        }
    }

    Ok(())
}

fn parse_use_imports(source: &str) -> Vec<(Vec<String>, Vec<String>)> {
    let mut out = Vec::new();

    for raw in source.split(';') {
        let stmt = raw.trim();
        if !stmt.starts_with("use ") {
            continue;
        }
        let body = stmt[4..].trim();
        if body.is_empty() {
            continue;
        }

        if let Some(open) = body.find('{') {
            let Some(close_rel) = body[open..].find('}') else { continue };
            let close = open + close_rel;
            let module = body[..open].trim().trim_end_matches("::");
            let symbols_raw = body[open + 1..close].trim();

            let mut module_segments = Vec::new();
            for seg in module.split("::") {
                let seg = seg.trim();
                if !seg.is_empty() {
                    module_segments.push(String::from(seg));
                }
            }
            if module_segments.is_empty() {
                continue;
            }

            let mut symbols = Vec::new();
            for sym in symbols_raw.split(',') {
                let sym = sym.trim();
                if sym.is_empty() || sym == "*" || sym == "self" {
                    continue;
                }
                symbols.push(String::from(sym));
            }
            if !symbols.is_empty() {
                out.push((module_segments, symbols));
            }
            continue;
        }

        let mut parts = Vec::new();
        for seg in body.split("::") {
            let seg = seg.trim();
            if !seg.is_empty() {
                parts.push(String::from(seg));
            }
        }
        if parts.len() < 2 {
            continue;
        }
        let symbol = parts.pop().unwrap();
        if symbol.as_str() == "*" || symbol.as_str() == "self" {
            continue;
        }
        let mut symbols = Vec::new();
        symbols.push(symbol);
        out.push((parts, symbols));
    }

    out
}

fn resolve_module_file(source_file: &Path, segments: &[String]) -> Option<PathBuf> {
    let mut base = find_src_root(source_file)?;
    let mut start_idx = 0usize;

    if !segments.is_empty() {
        let first = segments[0].as_str();
        if first == "crate" {
            start_idx = 1;
        } else if first == "self" {
            base = source_file.parent()?.to_path_buf();
            start_idx = 1;
        } else if first == "super" {
            base = source_file.parent()?.parent()?.to_path_buf();
            start_idx = 1;
        }
    }

    let mut current = base;
    for seg in &segments[start_idx..] {
        current = resolve_segment(current.as_path(), seg.as_str())?;
    }

    if current.as_path().is_file() {
        Some(current)
    } else {
        let mod_rs = current.as_path().join("mod.rs");
        if mod_rs.as_path().exists() {
            Some(mod_rs)
        } else {
            let mod_volki = current.as_path().join("mod.volki");
            if mod_volki.as_path().exists() {
                Some(mod_volki)
            } else {
                None
            }
        }
    }
}

fn resolve_segment(parent: &Path, segment: &str) -> Option<PathBuf> {
    let dir = if parent.is_dir() {
        parent.to_path_buf()
    } else {
        parent.parent()?.to_path_buf()
    };

    let volki = dir.join(&crate::vformat!("{segment}.volki"));
    if volki.as_path().exists() {
        return Some(volki);
    }
    let rs = dir.join(&crate::vformat!("{segment}.rs"));
    if rs.as_path().exists() {
        return Some(rs);
    }

    let nested = dir.join(segment);
    if nested.as_path().is_dir() {
        let mod_rs = nested.as_path().join("mod.rs");
        if mod_rs.as_path().exists() {
            return Some(mod_rs);
        }
        let mod_volki = nested.as_path().join("mod.volki");
        if mod_volki.as_path().exists() {
            return Some(mod_volki);
        }
        return Some(nested);
    }

    None
}

fn find_src_root(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };

    loop {
        if current.as_path().file_name() == Some("src") {
            return Some(current);
        }
        let parent = current.as_path().parent()?;
        if parent == current.as_path() {
            return None;
        }
        current = parent.to_path_buf();
    }
}

/// Extract function name from source text before `-> Fragment`.
/// Scans backward to find `fn name(...)` pattern.
fn extract_fn_name_before_arrow(before: &str) -> Option<String> {
    let trimmed = before.trim_end();
    // Should end with `)`
    if !trimmed.ends_with(')') {
        return None;
    }
    // Find matching `(`
    let bytes = trimmed.as_bytes();
    let mut depth = 1;
    let mut pos = bytes.len() - 1;
    while pos > 0 {
        pos -= 1;
        match bytes[pos] {
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

    // Now pos is at `(`, walk backward to find the function name
    let before_paren = &trimmed[..pos];
    let before_paren = before_paren.trim_end();
    // Take the last identifier
    let name_end = before_paren.len();
    let mut name_start = name_end;
    let bytes = before_paren.as_bytes();
    while name_start > 0 && (bytes[name_start - 1].is_ascii_alphanumeric() || bytes[name_start - 1] == b'_') {
        name_start -= 1;
    }
    if name_start == name_end {
        return None;
    }

    Some(String::from(&before_paren[name_start..name_end]))
}

/// Simple brace matcher for metadata body extraction.
fn find_matching_brace_simple(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 1;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
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

/// Convert directory path to URL path relative to `app/`.
fn dir_to_url(dir: &Path, root: &Path) -> String {
    let app_path = root.join("app");
    match dir.strip_prefix(app_path.as_path().as_str()) {
        Some(rel) if rel.is_empty() => String::from("/"),
        Some(rel) => {
            let mut url = String::from("/");
            url.push_str(rel);
            url
        }
        None => String::from("/"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_fn_name_before_arrow() {
        assert_eq!(
            extract_fn_name_before_arrow("fn sidebar() ").unwrap().as_str(),
            "sidebar"
        );
        assert_eq!(
            extract_fn_name_before_arrow("fn sidebar_content() ").unwrap().as_str(),
            "sidebar_content"
        );
        assert_eq!(
            extract_fn_name_before_arrow("pub fn my_frag() ").unwrap().as_str(),
            "my_frag"
        );
        assert!(extract_fn_name_before_arrow("invalid").is_none());
    }

    #[test]
    fn test_find_matching_brace_simple() {
        assert_eq!(find_matching_brace_simple("hello}"), Some(5));
        assert_eq!(find_matching_brace_simple("{inner}}rest"), Some(7));
        assert_eq!(find_matching_brace_simple("\"}\"}rest"), Some(3));
        assert!(find_matching_brace_simple("no closing brace").is_none());
    }

    #[test]
    fn test_dir_to_url_root() {
        let root = Path::new("/project");
        let dir = Path::new("/project/app");
        assert_eq!(dir_to_url(dir, root).as_str(), "/");
    }

    #[test]
    fn test_dir_to_url_nested() {
        let root = Path::new("/project");
        let dir = Path::new("/project/app/about");
        assert_eq!(dir_to_url(dir, root).as_str(), "/about");
    }

    #[test]
    fn test_dynamic_asset_stem() {
        let root = Path::new("/project");
        let page = Path::new("/project/app/admin/users/page.volki");
        assert_eq!(
            dynamic_asset_stem(page, root).as_str(),
            "dyn_app_admin_users_page"
        );
    }

    #[test]
    fn test_map_wasm_compile_error_to_volki() {
        let source = "pub fn f() -> Client {\n    let input = dom::query(\"#x\");\n    let term = input.get_value();\n}\n";
        let err = "rustc failed:\nerror[E0599]: no method named `get_value`\n79 |     let term = input.get_value();";
        let mapped = map_wasm_compile_error_to_volki(Path::new("/tmp/page.volki"), source, err);
        assert!(mapped.contains("/tmp/page.volki:3:"));
        assert!(mapped.contains("error[E0599]"));
    }
}
