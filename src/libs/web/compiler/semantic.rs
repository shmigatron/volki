//! Semantic validation for RSX component usage.
//!
//! This pass enforces strict component resolution:
//! - Custom component tags (`<MyComponent />`) must resolve to a function.
//! - Resolved component functions must return `Fragment`.
//! - Component props must match function parameters.

use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::{Path, PathBuf};

use super::parser::{RsxAttrValue, RsxNode};
use super::scanner::{FnParam, RsxFunction, RsxReturnType};
use super::CompileError;

struct UseStmt {
    module_segments: Vec<String>,
    symbols: Vec<String>,
}

/// Collect all Fragment component info (local + imported): (snake_case name, params).
/// Used by both validation and component resolution in mod.rs.
pub fn collect_fragment_components(
    source: &str,
    file: &Path,
    functions: &[RsxFunction],
) -> Result<Vec<(String, Vec<FnParam>)>, CompileError> {
    let mut components = Vec::new();

    // Local Fragment functions
    for f in functions {
        if f.return_type == RsxReturnType::Fragment {
            if let Some(name) = &f.name {
                components.push((name.clone(), f.params.clone()));
            }
        }
    }

    // Imported Fragment functions
    let use_stmts = parse_use_statements(source);
    for stmt in use_stmts {
        let Some(module_file) = resolve_module_file(file, &stmt.module_segments) else {
            continue;
        };

        let module_src = match fs::read_to_string(module_file.as_path()) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let module_functions = super::scanner::scan_functions(module_src.as_str());
        for mf in &module_functions {
            if mf.return_type == RsxReturnType::Fragment {
                if let Some(name) = &mf.name {
                    if stmt.symbols.iter().any(|s| s.as_str() == name.as_str()) {
                        let exists = components.iter().any(|(n, _)| n.as_str() == name.as_str());
                        if !exists {
                            components.push((name.clone(), mf.params.clone()));
                        }
                    }
                }
            }
        }
    }

    Ok(components)
}

pub fn validate_component_resolution(
    source: &str,
    file: &Path,
    functions: &[RsxFunction],
    parsed_bodies: &[Option<Vec<RsxNode>>],
    component_map: &[(String, Vec<FnParam>)],
    rsx_component_names: &[String],
) -> Result<(), CompileError> {
    let local_symbols = collect_local_symbols(functions);
    let imported_symbols = collect_imported_symbols(source, file)?;
    let client_symbols = collect_client_symbols(functions);

    for (idx, func) in functions.iter().enumerate() {
        if func.return_type != RsxReturnType::Html && func.return_type != RsxReturnType::Fragment {
            continue;
        }

        let Some(nodes) = parsed_bodies.get(idx).and_then(|n| n.as_ref()) else {
            continue;
        };

        validate_event_bindings(
            source,
            file,
            func.body_span,
            nodes,
            &client_symbols,
        )?;

        let mut component_tags = Vec::new();
        collect_component_tags(nodes, &mut component_tags);

        for tag in &component_tags {
            let snake = super::pascal_to_snake(tag.as_str());

            // Check if it's in the component map (Fragment functions)
            let in_component_map = component_map.iter().any(|(n, _)| n.as_str() == snake.as_str());
            let is_rsx_component = rsx_component_names.iter().any(|n| n.as_str() == snake.as_str());

            if !in_component_map && !is_rsx_component {
                // Check if it resolves to a non-Fragment function
                let resolved = find_symbol_return_type(&local_symbols, snake.as_str())
                    .or_else(|| find_symbol_return_type(&imported_symbols, snake.as_str()));

                if let Some(rt) = resolved {
                    if rt != RsxReturnType::Fragment {
                        let offset = find_component_tag_offset(source, func.body_span, tag.as_str())
                            .unwrap_or(func.body_span.0);
                        let (line, col) = line_col_at(source, offset);
                        return Err(CompileError {
                            file: file.to_path_buf(),
                            line,
                            col,
                            message: crate::vformat!(
                                "component `{}` must return Fragment (found {})",
                                tag,
                                return_type_name(rt)
                            ),
                        });
                    }
                } else {
                    let offset = find_component_tag_offset(source, func.body_span, tag.as_str())
                        .unwrap_or(func.body_span.0);
                    let (line, col) = line_col_at(source, offset);
                    return Err(CompileError {
                        file: file.to_path_buf(),
                        line,
                        col,
                        message: crate::vformat!(
                            "unresolved component `{}`; expected a function returning Fragment",
                            tag
                        ),
                    });
                }
            }
        }

        // Validate props for component tags
        validate_component_props(source, file, func.body_span, nodes, component_map)?;
    }

    Ok(())
}

fn validate_component_props(
    source: &str,
    file: &Path,
    body_span: (usize, usize),
    nodes: &[RsxNode],
    component_map: &[(String, Vec<FnParam>)],
) -> Result<(), CompileError> {
    for node in nodes {
        match node {
            RsxNode::Element { tag, attrs, children, .. } => {
                if is_custom_component_tag(tag.as_str()) && !is_special_builtin_component(tag.as_str()) {
                    let snake = super::pascal_to_snake(tag.as_str());
                    if let Some(params) = component_map.iter()
                        .find(|(n, _)| n.as_str() == snake.as_str())
                        .map(|(_, p)| p)
                    {
                        // Check each attr has a matching param
                        for attr in attrs {
                            let has_param = params.iter().any(|p| p.name.as_str() == attr.name.as_str());
                            if !has_param {
                                let offset = find_attr_offset(source, body_span, attr.name.as_str())
                                    .unwrap_or(body_span.0);
                                let (line, col) = line_col_at(source, offset);
                                return Err(CompileError {
                                    file: file.to_path_buf(),
                                    line,
                                    col,
                                    message: crate::vformat!(
                                        "unknown prop `{}` on component `{}`",
                                        attr.name, tag
                                    ),
                                });
                            }
                        }
                        // Check each required param has a matching attr (skip children)
                        for param in params {
                            if param.name.as_str() == "children" {
                                continue;
                            }
                            let has_attr = attrs.iter().any(|a| a.name.as_str() == param.name.as_str());
                            if !has_attr {
                                let offset = find_component_tag_offset(source, body_span, tag.as_str())
                                    .unwrap_or(body_span.0);
                                let (line, col) = line_col_at(source, offset);
                                return Err(CompileError {
                                    file: file.to_path_buf(),
                                    line,
                                    col,
                                    message: crate::vformat!(
                                        "missing required prop `{}` on component `{}`",
                                        param.name, tag
                                    ),
                                });
                            }
                        }
                        // Validate children: error if tag has children but function has no children param
                        let has_children_param = params.iter().any(|p| p.name.as_str() == "children");
                        if !children.is_empty() && !has_children_param {
                            let offset = find_component_tag_offset(source, body_span, tag.as_str())
                                .unwrap_or(body_span.0);
                            let (line, col) = line_col_at(source, offset);
                            return Err(CompileError {
                                file: file.to_path_buf(),
                                line,
                                col,
                                message: crate::vformat!(
                                    "component `{}` does not accept children; add a `children: Vec<HtmlNode>` parameter",
                                    tag
                                ),
                            });
                        }
                    }
                }
                // Recurse into children
                validate_component_props(source, file, body_span, children, component_map)?;
            }
            RsxNode::CondAnd { body, .. } => {
                validate_component_props(source, file, body_span, body, component_map)?;
            }
            RsxNode::Ternary { if_true, if_false, .. } => {
                validate_component_props(source, file, body_span, if_true, component_map)?;
                validate_component_props(source, file, body_span, if_false, component_map)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn collect_client_symbols(functions: &[RsxFunction]) -> Vec<(String, usize)> {
    let mut symbols = Vec::new();
    for f in functions {
        if f.return_type == RsxReturnType::Client {
            if let Some(name) = &f.name {
                symbols.push((name.clone(), f.params.len()));
            }
        }
    }
    symbols
}

fn validate_event_bindings(
    source: &str,
    file: &Path,
    body_span: (usize, usize),
    nodes: &[RsxNode],
    client_symbols: &[(String, usize)],
) -> Result<(), CompileError> {
    for node in nodes {
        validate_node_event_bindings(source, file, body_span, node, client_symbols)?;
    }
    Ok(())
}

fn validate_node_event_bindings(
    source: &str,
    file: &Path,
    body_span: (usize, usize),
    node: &RsxNode,
    client_symbols: &[(String, usize)],
) -> Result<(), CompileError> {
    match node {
        RsxNode::Element { tag, attrs, children, .. } => {
            let is_component = is_custom_component_tag(tag.as_str())
                && !is_special_builtin_component(tag.as_str());
            for attr in attrs {
                let name = attr.name.as_str();
                let is_event = name.starts_with("on") && name.len() > 2;
                match (&attr.value, is_event) {
                    // Allow expression attrs on component tags (they are props)
                    (RsxAttrValue::Expr(_), false) if !is_component => {
                        return attr_error(
                            source,
                            file,
                            body_span,
                            name,
                            "only event attributes support expression values; use a quoted string for non-event attrs",
                        );
                    }
                    (RsxAttrValue::Literal(v), true) => {
                        if v.contains("__volki.") || v.contains("window.__volki") {
                            return attr_error(
                                source,
                                file,
                                body_span,
                                name,
                                "legacy __volki inline handlers are removed; use event bindings like onclick={on_click}",
                            );
                        }
                        return attr_error(
                            source,
                            file,
                            body_span,
                            name,
                            "event attributes must use expression syntax like onclick={on_click}",
                        );
                    }
                    (RsxAttrValue::Expr(expr), true) if !is_component => {
                        if !is_identifier(expr.as_str()) {
                            return attr_error(
                                source,
                                file,
                                body_span,
                                name,
                                "event handler expression must be a top-level Client function identifier",
                            );
                        }
                        let Some((_, arity)) = client_symbols.iter().find(|(n, _)| n.as_str() == expr.as_str()) else {
                            return attr_error(
                                source,
                                file,
                                body_span,
                                name,
                                crate::vformat!("event handler `{}` not found as a top-level Client function", expr).as_str(),
                            );
                        };
                        if *arity > 1 {
                            return attr_error(
                                source,
                                file,
                                body_span,
                                name,
                                crate::vformat!("event handler `{}` has {} params; only 0 or 1 are supported", expr, arity).as_str(),
                            );
                        }
                    }
                    _ => {}
                }
            }
            for child in children {
                validate_node_event_bindings(source, file, body_span, child, client_symbols)?;
            }
        }
        RsxNode::CondAnd { body, .. } => {
            for child in body {
                validate_node_event_bindings(source, file, body_span, child, client_symbols)?;
            }
        }
        RsxNode::Ternary { if_true, if_false, .. } => {
            for child in if_true {
                validate_node_event_bindings(source, file, body_span, child, client_symbols)?;
            }
            for child in if_false {
                validate_node_event_bindings(source, file, body_span, child, client_symbols)?;
            }
        }
        RsxNode::Text(_) | RsxNode::Expr(_) => {}
    }
    Ok(())
}

fn attr_error(
    source: &str,
    file: &Path,
    body_span: (usize, usize),
    attr_name: &str,
    message: &str,
) -> Result<(), CompileError> {
    let offset = find_attr_offset(source, body_span, attr_name).unwrap_or(body_span.0);
    let (line, col) = line_col_at(source, offset);
    Err(CompileError {
        file: file.to_path_buf(),
        line,
        col,
        message: String::from(message),
    })
}

fn find_attr_offset(source: &str, body_span: (usize, usize), attr_name: &str) -> Option<usize> {
    if body_span.1 <= body_span.0 || body_span.1 > source.len() {
        return None;
    }
    let body = &source[body_span.0..body_span.1];
    let needle = crate::vformat!("{}=", attr_name);
    body.find(needle.as_str()).map(|idx| body_span.0 + idx)
}

fn is_identifier(expr: &str) -> bool {
    let s = expr.trim();
    if s.is_empty() {
        return false;
    }
    let mut bytes = s.bytes();
    let Some(first) = bytes.next() else { return false; };
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }
    bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn collect_local_symbols(functions: &[RsxFunction]) -> Vec<(String, RsxReturnType)> {
    let mut symbols = Vec::new();
    for f in functions {
        if let Some(name) = &f.name {
            symbols.push((name.clone(), f.return_type));
        }
    }
    symbols
}

fn collect_component_tags(nodes: &[RsxNode], out: &mut Vec<String>) {
    for node in nodes {
        match node {
            RsxNode::Element { tag, children, .. } => {
                if is_custom_component_tag(tag.as_str()) && !is_special_builtin_component(tag.as_str()) {
                    out.push(tag.clone());
                }
                collect_component_tags(children, out);
            }
            RsxNode::CondAnd { body, .. } => collect_component_tags(body, out),
            RsxNode::Ternary { if_true, if_false, .. } => {
                collect_component_tags(if_true, out);
                collect_component_tags(if_false, out);
            }
            RsxNode::Text(_) | RsxNode::Expr(_) => {}
        }
    }
}

fn is_custom_component_tag(tag: &str) -> bool {
    let first = tag.as_bytes().first().copied().unwrap_or(b'\0');
    first.is_ascii_uppercase()
}

fn is_special_builtin_component(tag: &str) -> bool {
    tag == "Style" || tag == "Head" || tag == "Stylesheet"
}

fn find_symbol_return_type(
    symbols: &[(String, RsxReturnType)],
    name: &str,
) -> Option<RsxReturnType> {
    for (sym, ty) in symbols {
        if sym.as_str() == name {
            return Some(*ty);
        }
    }
    None
}

fn parse_use_statements(source: &str) -> Vec<UseStmt> {
    let mut result = Vec::new();
    for raw in source.split(';') {
        let stmt = raw.trim();
        if !stmt.starts_with("use ") {
            continue;
        }
        let path = stmt[4..].trim();
        if path.is_empty() {
            continue;
        }
        if let Some(open) = path.find('{') {
            let Some(close) = path[open..].find('}') else {
                continue;
            };
            let close = open + close;
            let module_part = path[..open].trim().trim_end_matches("::");
            let symbols_part = path[open + 1..close].trim();
            let mut symbols = Vec::new();
            for sym in symbols_part.split(',') {
                let sym = sym.trim();
                if sym.is_empty() || sym == "*" || sym == "self" {
                    continue;
                }
                symbols.push(String::from(sym));
            }
            if symbols.is_empty() {
                continue;
            }
            let mut module_segments = Vec::new();
            for seg in module_part.split("::") {
                let seg = seg.trim();
                if !seg.is_empty() {
                    module_segments.push(String::from(seg));
                }
            }
            if !module_segments.is_empty() {
                result.push(UseStmt {
                    module_segments,
                    symbols,
                });
            }
            continue;
        }

        // Non-brace import: use a::b::Name
        let mut parts = Vec::new();
        for seg in path.split("::") {
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
        result.push(UseStmt {
            module_segments: parts,
            symbols: {
                let mut v = Vec::new();
                v.push(symbol);
                v
            },
        });
    }
    result
}

fn collect_imported_symbols(
    source: &str,
    source_file: &Path,
) -> Result<Vec<(String, RsxReturnType)>, CompileError> {
    let use_stmts = parse_use_statements(source);
    if use_stmts.is_empty() {
        return Ok(Vec::new());
    }

    let mut imported = Vec::new();
    for stmt in use_stmts {
        let Some(module_file) = resolve_module_file(source_file, &stmt.module_segments) else {
            continue;
        };

        let module_src = fs::read_to_string(module_file.as_path()).map_err(|e| CompileError {
            file: source_file.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!(
                "failed to read imported module `{}`: {}",
                module_file,
                e
            ),
        })?;
        let module_functions = super::scanner::scan_functions(module_src.as_str());
        let module_symbols = collect_local_symbols(&module_functions);

        for sym in stmt.symbols {
            if let Some(ty) = find_symbol_return_type(&module_symbols, sym.as_str()) {
                imported.push((sym, ty));
            }
        }
    }

    Ok(imported)
}

fn resolve_module_file(source_file: &Path, segments: &[String]) -> Option<PathBuf> {
    let mut src_root = find_src_root(source_file)?;
    let mut start = 0usize;

    if !segments.is_empty() {
        let first = segments[0].as_str();
        if first == "crate" {
            start = 1;
        } else if first == "self" {
            src_root = source_file.parent()?.to_path_buf();
            start = 1;
        } else if first == "super" {
            src_root = source_file.parent()?.parent()?.to_path_buf();
            start = 1;
        }
    }

    let mut current = src_root;
    for seg in &segments[start..] {
        current = resolve_segment(current.as_path(), seg.as_str())?;
    }

    if current.as_path().is_file() {
        Some(current)
    } else {
        current.as_path().join("mod.rs").as_path().exists()
            .then(|| current.as_path().join("mod.rs"))
            .or_else(|| current.as_path().join("mod.volki").as_path().exists().then(|| current.as_path().join("mod.volki")))
    }
}

fn resolve_segment(parent: &Path, segment: &str) -> Option<PathBuf> {
    let dir = if parent.is_dir() {
        parent.to_path_buf()
    } else {
        parent.parent()?.to_path_buf()
    };

    let direct_volki = dir.join(&crate::vformat!("{segment}.volki"));
    if direct_volki.as_path().exists() {
        return Some(direct_volki);
    }
    let direct_rs = dir.join(&crate::vformat!("{segment}.rs"));
    if direct_rs.as_path().exists() {
        return Some(direct_rs);
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
        current = current.as_path().parent()?.to_path_buf();
    }
}

fn return_type_name(rt: RsxReturnType) -> &'static str {
    match rt {
        RsxReturnType::Html => "Html",
        RsxReturnType::Fragment => "Fragment",
        RsxReturnType::Client => "Client",
        RsxReturnType::Component => "Component",
    }
}

fn find_component_tag_offset(source: &str, body_span: (usize, usize), tag: &str) -> Option<usize> {
    if body_span.1 <= body_span.0 || body_span.1 > source.len() {
        return None;
    }
    let body = &source[body_span.0..body_span.1];
    let needle = crate::vformat!("<{}", tag);
    body.find(needle.as_str()).map(|idx| body_span.0 + idx)
}

fn line_col_at(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let end = offset.min(bytes.len());
    let mut line = 1usize;
    let mut col = 1usize;
    for &b in &bytes[..end] {
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}
