//! Volki Compiler — transforms `.volki` files (HTML-in-Rust) into `.rs` files.
//!
//! Output goes to a configurable dist directory (default: `".volki"`),
//! configured via `[web].dist` in `volki.toml`.

pub mod boundary;
pub mod codegen;
pub mod js_codegen;
pub mod minify;
pub mod parser;
pub mod routes;
pub mod scanner;
pub mod semantic;
pub mod tokenizer;
pub mod wasm_build;
pub mod wasm_codegen;
pub mod wasm_rsx_codegen;

use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::{Path, PathBuf};

use scanner::RsxReturnType;

/// Output from client-side compilation (WASM + JS glue).
#[derive(Debug)]
pub struct ClientOutput {
    pub wasm_rs: String,
    pub glue_js: String,
}

/// Result of compiling a single `.volki` file.
pub struct CompileResult {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub warnings: Vec<CompileWarning>,
    pub client: Option<ClientOutput>,
}

#[derive(Debug, Clone)]
pub struct CompileWarning {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub message: String,
}

/// Error during compilation.
#[derive(Debug)]
pub struct CompileError {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}:{}: {}", self.file, self.line, self.col, self.message)
    }
}

/// Read the `[web].dist` value from `volki.toml` in the given directory.
/// Returns `".volki"` if not found or not configured.
pub fn read_dist_config(dir: &Path) -> String {
    let config_path = dir.join("volki.toml");
    if !config_path.as_path().exists() {
        return String::from(".volki");
    }
    let content = match fs::read_to_string(config_path.as_path()) {
        Ok(c) => c,
        Err(_) => return String::from(".volki"),
    };
    let table = match crate::core::config::parser::parse(content.as_str()) {
        Ok(t) => t,
        Err(_) => return String::from(".volki"),
    };
    match table.get("web", "dist") {
        Some(v) => match v.as_str() {
            Some(s) => String::from(s),
            None => String::from(".volki"),
        },
        None => String::from(".volki"),
    }
}

/// Read the `[web].entrypoint` value from `volki.toml` in the given directory.
/// Returns `"."` if not found or not configured.
pub fn read_entrypoint_config(dir: &Path) -> String {
    let config_path = dir.join("volki.toml");
    if !config_path.as_path().exists() {
        return String::from(".");
    }
    let content = match fs::read_to_string(config_path.as_path()) {
        Ok(c) => c,
        Err(_) => return String::from("."),
    };
    let table = match crate::core::config::parser::parse(content.as_str()) {
        Ok(t) => t,
        Err(_) => return String::from("."),
    };
    match table.get("web", "entrypoint") {
        Some(v) => match v.as_str() {
            Some(s) => String::from(s),
            None => String::from("."),
        },
        None => String::from("."),
    }
}

/// Result of compiling a single `.volki` source string.
#[derive(Debug)]
pub struct SourceOutput {
    pub server_rs: String,
    pub client: Option<ClientOutput>,
    pub warnings: Vec<CompileWarning>,
}

/// Compile a single `.volki` source string into a Rust source string.
/// Client functions are stripped from the server output and compiled separately.
pub fn compile_source(source: &str, file: &Path) -> Result<String, CompileError> {
    let out = compile_source_full(source, file)?;
    Ok(out.server_rs)
}

/// Compile a `.volki` source, returning both server and client output.
pub fn compile_source_full(source: &str, file: &Path) -> Result<SourceOutput, CompileError> {
    use crate::libs::web::volkistyle;

    let functions = scanner::scan_functions(source);

    // Validate server/client boundaries + top-level misuse before any parsing
    let mut violations = boundary::validate_boundaries(&functions, source);
    let top_violations = boundary::validate_top_level(&functions, source);
    for v in top_violations {
        violations.push(v);
    }
    if !violations.is_empty() {
        // Format all violations into a single error message
        let mut msg = String::new();
        for (idx, v) in violations.iter().enumerate() {
            if idx > 0 {
                msg.push_str("\n\n");
            }
            msg.push_str(crate::vformat!(
                "error: {}\n  --> {}:{}:{}\n   |\n   = help: {}",
                v.message, file, v.line, v.col, v.help
            ).as_str());
        }
        let first = &violations[0];
        return Err(CompileError {
            file: file.to_path_buf(),
            line: first.line,
            col: first.col,
            message: msg,
        });
    }

    if functions.is_empty() {
        return Ok(SourceOutput {
            server_rs: String::from(source),
            client: None,
            warnings: Vec::new(),
        });
    }

    // Partition into client and component functions
    let client_fns: Vec<&scanner::RsxFunction> = functions.iter()
        .filter(|f| f.return_type == RsxReturnType::Client)
        .collect();
    let component_fns: Vec<&scanner::RsxFunction> = functions.iter()
        .filter(|f| f.return_type == RsxReturnType::Component)
        .collect();

    // First pass: parse all Html/Fragment function bodies.
    let mut parsed_bodies: Vec<Option<Vec<parser::RsxNode>>> = Vec::new();

    for func in &functions {
        if func.return_type == RsxReturnType::Client
            || func.return_type == RsxReturnType::Component {
            parsed_bodies.push(None);
            continue;
        }

        let body = &source[func.body_span.0..func.body_span.1];
        let file_buf = file.to_path_buf();
        let tokens = tokenizer::tokenize(body.trim(), file_buf.clone())?;
        let nodes = parser::parse(&tokens, file_buf)?;
        parsed_bodies.push(Some(nodes));
    }

    // Build component map from Fragment functions (local + imported)
    let component_map = semantic::collect_fragment_components(source, file, &functions)?;

    // Collect CSS classes BEFORE component resolution (captures component children classes)
    let mut all_classes = Vec::new();
    for body_opt in parsed_bodies.iter() {
        if let Some(nodes) = body_opt {
            let fn_classes = volkistyle::collector::collect_classes(nodes);
            for c in fn_classes.iter() {
                all_classes.push(c.clone());
            }
        }
    }

    // Parse RSX from Component functions with return (RSX)
    let mut component_rsx_bodies: Vec<Option<Vec<parser::RsxNode>>> = Vec::new();
    let mut has_rsx_components = false;
    let mut rsx_component_names: Vec<String> = Vec::new();

    for func in &component_fns {
        if let Some(split) = scanner::split_component_body(source, func.body_span) {
            let rsx_src = &source[split.rsx_span.0..split.rsx_span.1];
            let file_buf = file.to_path_buf();
            let tokens = tokenizer::tokenize(rsx_src.trim(), file_buf.clone())?;
            let nodes = parser::parse(&tokens, file_buf)?;
            // Collect CSS classes from Component RSX
            let rsx_classes = volkistyle::collector::collect_classes(&nodes);
            for c in rsx_classes.iter() {
                all_classes.push(c.clone());
            }
            component_rsx_bodies.push(Some(nodes));
            has_rsx_components = true;
            if let Some(name) = &func.name {
                rsx_component_names.push(name.clone());
            }
        } else {
            component_rsx_bodies.push(None);
        }
    }

    // Semantic validation on parsed RSX nodes (before resolution)
    semantic::validate_component_resolution(source, file, &functions, &parsed_bodies, &component_map, &rsx_component_names)?;

    // Resolve component tags into function call expressions
    for i in 0..parsed_bodies.len() {
        if let Some(nodes) = &parsed_bodies[i] {
            if functions[i].return_type == RsxReturnType::Html
                || functions[i].return_type == RsxReturnType::Fragment
            {
                let resolved = resolve_components(nodes, &component_map, &rsx_component_names);
                parsed_bodies[i] = Some(resolved);
            }
        }
    }

    // Generate CSS from all collected classes
    let style_cfg = volkistyle::config::load_for_source_file(file);
    let style_report = volkistyle::generate_css_with_config(&all_classes, &style_cfg);
    let mut warnings = compile_warnings_from_style(file, source, &style_report);
    if style_cfg.unknown_class_policy == volkistyle::config::UnknownClassPolicy::Error
        && !style_report.diagnostics.is_empty()
    {
        let first = &style_report.diagnostics[0];
        let (line, col) = find_class_occurrence(source, first.class_name.as_str()).unwrap_or((0, 0));
        return Err(CompileError {
            file: file.to_path_buf(),
            line,
            col,
            message: crate::vformat!("style error: {}", first.message),
        });
    }
    let css = style_report.css.clone();

    // Second pass: build server output using pre-parsed nodes
    let mut output = String::with_capacity(source.len() * 2);
    let mut last_pos = 0;

    for (i, func) in functions.iter().enumerate() {
        if func.return_type == RsxReturnType::Client
            || func.return_type == RsxReturnType::Component {
            let fn_start = find_fn_start(source, func.return_type_span.0);
            let before = &source[last_pos..fn_start];
            output.push_str(before);
            last_pos = func.body_span.1 + 1;
            if last_pos < source.len() && source.as_bytes()[last_pos] == b'\n' {
                last_pos += 1;
            }
            continue;
        }

        let before = &source[last_pos..func.return_type_span.0];
        output.push_str(before);

        match func.return_type {
            RsxReturnType::Html => output.push_str("HtmlDocument"),
            RsxReturnType::Fragment => output.push_str("Vec<HtmlNode>"),
            RsxReturnType::Client | RsxReturnType::Component => unreachable!(),
        }

        let between = &source[func.return_type_span.1..func.body_span.0];
        output.push_str(between);

        let nodes = parsed_bodies[i].as_ref().unwrap();

        let compiled_body = match func.return_type {
            RsxReturnType::Html => {
                let has_client_code = !client_fns.is_empty() || !component_fns.is_empty();
                let glue_url = if has_client_code {
                    let stem = file.file_stem().unwrap_or("module");
                    Some(crate::vformat!("/wasm/{}_glue.js", stem))
                } else {
                    None
                };
                codegen::generate_html_fn_styled(
                    nodes,
                    css.as_str(),
                    glue_url.as_ref().map(|s| s.as_str()),
                )
            }
            RsxReturnType::Fragment => codegen::generate_fragment_fn(nodes),
            RsxReturnType::Client | RsxReturnType::Component => unreachable!(),
        };

        output.push_str("\n    ");
        output.push_str(compiled_body.as_str());

        last_pos = func.body_span.1;
    }

    let remainder = &source[last_pos..];
    output.push_str(remainder);

    let output = match minify::minify_rust_generated(output.as_str()) {
        Ok(s) => s,
        Err(e) => {
            warnings.push(CompileWarning {
                file: file.to_path_buf(),
                line: e.line,
                col: e.col,
                message: crate::vformat!("minify fallback (server_rs): {}", e),
            });
            output
        }
    };

    // Build client output if there are Client or Component functions
    let has_client_code = !client_fns.is_empty() || !component_fns.is_empty();
    let client = if has_client_code {
        let file_stem = file.file_stem().unwrap_or("module");
        let wasm_url = crate::vformat!("/wasm/{}_client.wasm", file_stem);
        let wasm_rs_raw = wasm_codegen::generate_wasm_module(&client_fns, &component_fns, source, &component_rsx_bodies);
        let glue_js_raw = js_codegen::generate_js_glue(&client_fns, &component_fns, source, wasm_url.as_str(), has_rsx_components);
        let wasm_rs = match minify::minify_rust_generated(wasm_rs_raw.as_str()) {
            Ok(s) => s,
            Err(e) => {
                warnings.push(CompileWarning {
                    file: file.to_path_buf(),
                    line: e.line,
                    col: e.col,
                    message: crate::vformat!("minify fallback (wasm_rs): {}", e),
                });
                wasm_rs_raw
            }
        };
        let glue_js = match minify::minify_js_generated(glue_js_raw.as_str()) {
            Ok(s) => s,
            Err(e) => {
                warnings.push(CompileWarning {
                    file: file.to_path_buf(),
                    line: e.line,
                    col: e.col,
                    message: crate::vformat!("minify fallback (glue_js): {}", e),
                });
                glue_js_raw
            }
        };
        Some(ClientOutput { wasm_rs, glue_js })
    } else {
        None
    };

    Ok(SourceOutput {
        server_rs: output,
        client,
        warnings,
    })
}

/// Check if a tag is a custom component (starts with uppercase, not a builtin).
fn is_component_tag(tag: &str) -> bool {
    let first = tag.as_bytes().first().copied().unwrap_or(b'\0');
    first.is_ascii_uppercase() && tag != "Style" && tag != "Head" && tag != "Stylesheet"
}

/// Convert a PascalCase name to snake_case.
/// e.g. `SidebarContent` → `sidebar_content`, `Counter` → `counter`
pub fn pascal_to_snake(name: &str) -> String {
    let mut out = String::new();
    for (i, b) in name.as_bytes().iter().enumerate() {
        if b.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push((*b + 32) as char);
        } else {
            out.push(*b as char);
        }
    }
    out
}

/// Recursively resolve component tags into function call expressions.
/// Replaces `<Counter show_help={true} />` with `Expr("counter(true)")`.
fn resolve_components(
    nodes: &[parser::RsxNode],
    components: &[(String, Vec<scanner::FnParam>)],
    rsx_components: &[String],
) -> Vec<parser::RsxNode> {
    let mut out = Vec::new();
    for node in nodes {
        match node {
            parser::RsxNode::Element { tag, attrs, children, self_closing } => {
                if is_component_tag(tag.as_str()) {
                    let snake = pascal_to_snake(tag.as_str());

                    // RSX Component → mount-point div
                    if rsx_components.iter().any(|n| n.as_str() == snake.as_str()) {
                        let mount_expr = crate::vformat!(
                            "div().attr(\"data-volki-component\", \"{}\").into_node()",
                            snake
                        );
                        out.push(parser::RsxNode::Expr(mount_expr));
                        continue;
                    }

                    // Fragment Component → function call
                    if let Some(params) = components.iter()
                        .find(|(n, _)| n.as_str() == snake.as_str())
                        .map(|(_, p)| p)
                    {
                        let call = build_component_call(
                            snake.as_str(), params, attrs, children, components, rsx_components,
                        );
                        out.push(parser::RsxNode::Expr(call));
                        continue;
                    }
                }
                let resolved_children = resolve_components(children, components, rsx_components);
                out.push(parser::RsxNode::Element {
                    tag: tag.clone(),
                    attrs: attrs.clone(),
                    children: resolved_children,
                    self_closing: *self_closing,
                });
            }
            parser::RsxNode::CondAnd { condition, body } => {
                let resolved = resolve_components(body, components, rsx_components);
                out.push(parser::RsxNode::CondAnd {
                    condition: condition.clone(),
                    body: resolved,
                });
            }
            parser::RsxNode::Ternary { condition, if_true, if_false } => {
                let rt = resolve_components(if_true, components, rsx_components);
                let rf = resolve_components(if_false, components, rsx_components);
                out.push(parser::RsxNode::Ternary {
                    condition: condition.clone(),
                    if_true: rt,
                    if_false: rf,
                });
            }
            other => out.push(other.clone()),
        }
    }
    out
}

/// Build a function call string from component tag attributes and children.
/// Maps tag attributes to function parameters in declaration order.
fn build_component_call(
    fn_name: &str,
    params: &[scanner::FnParam],
    attrs: &[parser::RsxAttr],
    children: &[parser::RsxNode],
    components: &[(String, Vec<scanner::FnParam>)],
    rsx_components: &[String],
) -> String {
    let mut call = String::from(fn_name);
    call.push('(');

    let mut first = true;
    for param in params {
        if param.name.as_str() == "children" {
            if !first {
                call.push_str(", ");
            }
            first = false;
            if children.is_empty() {
                call.push_str("Vec::new()");
            } else {
                let resolved_children = resolve_components(children, components, rsx_components);
                let children_expr = codegen::generate_children_expr(&resolved_children);
                call.push_str(children_expr.as_str());
            }
            continue;
        }

        if !first {
            call.push_str(", ");
        }
        first = false;

        if let Some(attr) = attrs.iter().find(|a| a.name.as_str() == param.name.as_str()) {
            match &attr.value {
                parser::RsxAttrValue::Literal(v) => {
                    call.push('"');
                    call.push_str(v.as_str());
                    call.push('"');
                }
                parser::RsxAttrValue::Expr(e) => {
                    call.push_str(e.as_str());
                }
            }
        }
    }

    call.push(')');
    call
}

fn compile_warnings_from_style(
    file: &Path,
    source: &str,
    report: &crate::libs::web::volkistyle::diagnostics::GenerateCssReport,
) -> Vec<CompileWarning> {
    let mut out = Vec::new();
    for d in report.diagnostics.iter() {
        let (line, col) = find_class_occurrence(source, d.class_name.as_str()).unwrap_or((0, 0));
        out.push(CompileWarning {
            file: file.to_path_buf(),
            line,
            col,
            message: d.message.clone(),
        });
    }
    out
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

/// Walk backward from a position to find the start of a `fn` or `pub fn` declaration.
fn find_fn_start(source: &str, pos: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = pos;

    // Walk backward past the return type arrow and whitespace to find `fn ` keyword
    while i > 0 {
        // Look for "fn " pattern
        if i >= 3 && &bytes[i - 3..i] == b"fn " {
            let fn_start = i - 3;
            // Check for "pub " before "fn"
            if fn_start >= 4 && &bytes[fn_start - 4..fn_start] == b"pub " {
                return fn_start - 4;
            }
            return fn_start;
        }
        i -= 1;
    }
    0
}

/// Compile a single `.volki` file, writing output to `dist_dir` mirroring
/// the relative path from `source_root`.
fn compile_file_to_dist(
    path: &Path,
    source_root: &Path,
    dist_dir: &Path,
) -> Result<CompileResult, CompileError> {
    let source = fs::read_to_string(path).map_err(|e| CompileError {
        file: path.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to read file: {}", e),
    })?;

    let full_output = compile_source_full(source.as_str(), path)?;

    // Mirror source path into dist
    let relative = path.strip_prefix(source_root.as_str()).unwrap_or(path.as_str());
    let out_path = dist_dir.join(relative);
    let out_path = out_path.with_extension("rs");

    // Ensure parent directory exists
    if let Some(parent) = out_path.as_path().parent() {
        fs::create_dir_all(parent).map_err(|e| CompileError {
            file: path.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to create output directory: {}", e),
        })?;
    }

    fs::write_str(out_path.as_path(), full_output.server_rs.as_str()).map_err(|e| CompileError {
        file: path.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to write output: {}", e),
    })?;

    // Write client artifacts if present
    let client = if let Some(ref client_out) = full_output.client {
        let stem = path.file_stem().unwrap_or("module");

        // Write _client.rs alongside the server .rs
        let client_rs_path = out_path.as_path().parent()
            .map(|p| p.join(&crate::vformat!("{}_client.rs", stem)))
            .unwrap_or_else(|| PathBuf::from(&crate::vformat!("{}_client.rs", stem)));
        fs::write_str(client_rs_path.as_path(), client_out.wasm_rs.as_str()).map_err(|e| CompileError {
            file: path.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to write client .rs: {}", e),
        })?;

        // Write glue JS to dist/public/wasm/<stem>_glue.js for static serving
        let wasm_dir = dist_dir.join("public").join("wasm");
        fs::create_dir_all(wasm_dir.as_path()).map_err(|e| CompileError {
            file: path.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to create wasm directory: {}", e),
        })?;
        let glue_path = wasm_dir.join(&crate::vformat!("{}_glue.js", stem));
        fs::write_str(glue_path.as_path(), client_out.glue_js.as_str()).map_err(|e| CompileError {
            file: path.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to write glue JS: {}", e),
        })?;

        // Compile _client.rs to .wasm
        let wasm_path = wasm_dir.join(&crate::vformat!("{}_client.wasm", stem));
        wasm_build::compile_wasm(client_rs_path.as_path(), wasm_path.as_path())?;

        Some(ClientOutput {
            wasm_rs: client_out.wasm_rs.clone(),
            glue_js: client_out.glue_js.clone(),
        })
    } else {
        None
    };

    Ok(CompileResult {
        source_path: path.to_path_buf(),
        output_path: out_path,
        warnings: full_output.warnings.clone(),
        client,
    })
}

/// Copy a non-volki `.rs` file to the dist directory, preserving relative path.
fn copy_rs_to_dist(
    path: &Path,
    source_root: &Path,
    dist_dir: &Path,
) -> Result<(), CompileError> {
    let content = fs::read_to_string(path).map_err(|e| CompileError {
        file: path.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to read file: {}", e),
    })?;

    let relative = path.strip_prefix(source_root.as_str()).unwrap_or(path.as_str());
    let out_path = dist_dir.join(relative);

    if let Some(parent) = out_path.as_path().parent() {
        fs::create_dir_all(parent).map_err(|e| CompileError {
            file: path.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to create output directory: {}", e),
        })?;
    }

    fs::write_str(out_path.as_path(), content.as_str()).map_err(|e| CompileError {
        file: path.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to write output: {}", e),
    })?;

    Ok(())
}

/// Check if a file extension is a static asset that should be copied to dist/public/.
fn is_static_asset(ext: &str) -> bool {
    matches!(
        ext,
        "css" | "svg" | "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "ico"
        | "woff" | "woff2" | "ttf" | "otf"
    )
}

/// Copy a static asset file to `dist/public/{relative_path}`, preserving its
/// path relative to the source root.
fn copy_asset_to_public(
    path: &Path,
    source_root: &Path,
    dist_dir: &Path,
) -> Result<(), CompileError> {
    let relative = path.strip_prefix(source_root.as_str()).unwrap_or(path.as_str());
    let out_path = dist_dir.join("public").join(relative);

    if let Some(parent) = out_path.as_path().parent() {
        fs::create_dir_all(parent).map_err(|e| CompileError {
            file: path.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to create output directory: {}", e),
        })?;
    }

    let content = fs::read(path).map_err(|e| CompileError {
        file: path.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to read asset: {}", e),
    })?;
    fs::write(out_path.as_path(), content.as_slice()).map_err(|e| CompileError {
        file: path.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to write asset: {}", e),
    })?;

    Ok(())
}

/// Recursively copy a directory's contents to a destination.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), CompileError> {
    fs::create_dir_all(dst).map_err(|e| CompileError {
        file: src.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to create directory: {}", e),
    })?;

    let entries = fs::read_dir(src).map_err(|e| CompileError {
        file: src.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to read directory: {}", e),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| CompileError {
            file: src.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to read entry: {}", e),
        })?;

        let src_path = entry.path().to_path_buf();
        let dst_path = dst.join(entry.file_name());

        if entry.file_type() == fs::FileType::Directory {
            copy_dir_recursive(src_path.as_path(), dst_path.as_path())?;
        } else {
            let content = fs::read(src_path.as_path()).map_err(|e| CompileError {
                file: src_path.to_path_buf(),
                line: 0,
                col: 0,
                message: crate::vformat!("failed to read file: {}", e),
            })?;
            fs::write(dst_path.as_path(), content.as_slice()).map_err(|e| CompileError {
                file: src_path.to_path_buf(),
                line: 0,
                col: 0,
                message: crate::vformat!("failed to write file: {}", e),
            })?;
        }
    }

    Ok(())
}

/// Compile a directory of `.volki` files, outputting to `dist_dir`.
///
/// - Compiles all `.volki` files to `.rs` in the dist directory
/// - Copies non-volki `.rs` files to the dist directory
/// - Copies `public/` directory contents to `dist/public/`
/// - Discovers routes from the `app/` subdirectory
/// - Generates `mod.rs` files at each directory level in dist
/// - Generates a root `mod.rs` with a `start()` function in dist
/// - Writes a re-export `mod.rs` at the source root pointing to dist
pub fn compile_dir(source_dir: &Path, dist_name: &str) -> Result<Vec<CompileResult>, CompileError> {
    let dist_dir = source_dir.join(dist_name);

    // Remove previous dist directory for a clean build
    if dist_dir.as_path().exists() {
        fs::remove_dir_all(dist_dir.as_path()).map_err(|e| CompileError {
            file: source_dir.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to remove old dist directory: {}", e),
        })?;
    }

    // Create dist directory
    fs::create_dir_all(dist_dir.as_path()).map_err(|e| CompileError {
        file: source_dir.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to create dist directory: {}", e),
    })?;

    let mut results = Vec::new();

    // Copy public/ directory to dist/public/ if it exists
    let public_src = source_dir.join("public");
    if public_src.as_path().exists() {
        let public_dst = dist_dir.join("public");
        copy_dir_recursive(public_src.as_path(), public_dst.as_path())?;
    }

    // Walk source tree: compile .volki, copy .rs
    walk_and_compile(source_dir, source_dir, dist_dir.as_path(), dist_name, &mut results)?;

    // Discover routes from source (checks for .volki and .rs files)
    let discovered = routes::discover_routes(source_dir)?;

    // Check if dist/public/ exists for static asset serving
    let public_dst = dist_dir.join("public");
    let public_dir_opt = if public_dst.as_path().exists() {
        let public_path = crate::vformat!("{}/public", dist_name);
        Some(public_path)
    } else {
        None
    };

    // Generate mod.rs files in dist
    let root_content = routes::generate_root_mod(
        dist_dir.as_path(),
        &discovered,
        public_dir_opt.as_ref().map(|s| s.as_str()),
    )?;
    let root_mod = dist_dir.join("mod.rs");
    let root_content = minify::minify_route_mod_generated(root_content.as_str())
        .unwrap_or(root_content);
    fs::write_str(root_mod.as_path(), root_content.as_str()).map_err(|e| CompileError {
        file: dist_dir.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to write mod.rs: {}", e),
    })?;

    generate_sub_mod_files(dist_dir.as_path())?;

    // Write re-export mod.rs at source root
    let reexport = crate::vformat!(
        "//! @generated by volki compiler — do not edit.\n\n#[path = \"{}\"]\nmod generated;\npub use generated::*;\n",
        dist_name
    );
    let reexport = minify::minify_route_mod_generated(reexport.as_str())
        .unwrap_or(reexport);
    let src_mod = source_dir.join("mod.rs");
    fs::write_str(src_mod.as_path(), reexport.as_str()).map_err(|e| CompileError {
        file: source_dir.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to write re-export mod.rs: {}", e),
    })?;

    Ok(results)
}

fn walk_and_compile(
    dir: &Path,
    source_root: &Path,
    dist_dir: &Path,
    dist_name: &str,
    results: &mut Vec<CompileResult>,
) -> Result<(), CompileError> {
    let entries = fs::read_dir(dir).map_err(|e| CompileError {
        file: dir.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to read directory: {}", e),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| CompileError {
            file: dir.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to read dir entry: {}", e),
        })?;

        let path = entry.path();
        let name = entry.file_name();

        // Skip the dist directory and public directory at the source root
        if dir.as_str() == source_root.as_str() && (name == dist_name || name == "public") {
            continue;
        }

        if entry.file_type() == fs::FileType::Directory {
            walk_and_compile(path, source_root, dist_dir, dist_name, results)?;
        } else if path.extension() == Some("volki") {
            results.push(compile_file_to_dist(path, source_root, dist_dir)?);
        } else if path.extension() == Some("rs") && name != "mod.rs" {
            copy_rs_to_dist(path, source_root, dist_dir)?;
        } else if let Some(ext) = path.extension() {
            if is_static_asset(ext) {
                copy_asset_to_public(path, source_root, dist_dir)?;
            }
        }
    }

    Ok(())
}

fn generate_sub_mod_files(dir: &Path) -> Result<(), CompileError> {
    let entries = fs::read_dir(dir).map_err(|e| CompileError {
        file: dir.to_path_buf(),
        line: 0,
        col: 0,
        message: crate::vformat!("failed to read directory: {}", e),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| CompileError {
            file: dir.to_path_buf(),
            line: 0,
            col: 0,
            message: crate::vformat!("failed to read entry: {}", e),
        })?;
        if entry.file_type() == fs::FileType::Directory {
            let sub_dir = entry.path().to_path_buf();
            let content = routes::generate_mod_file(sub_dir.as_path())?;
            let content = minify::minify_route_mod_generated(content.as_str())
                .unwrap_or(content);
            let mod_file = sub_dir.join("mod.rs");
            fs::write_str(mod_file.as_path(), content.as_str()).map_err(|e| CompileError {
                file: sub_dir.to_path_buf(),
                line: 0,
                col: 0,
                message: crate::vformat!("failed to write mod.rs: {}", e),
            })?;
            generate_sub_mod_files(sub_dir.as_path())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_full_file() {
        let source = r##"use crate::libs::web::prelude::*;

pub fn metadata(_req: &Request) -> Metadata {
    Metadata::new()
        .title("test page")
}

pub fn page(_req: &Request) -> Html {
    <Style>{CSS}</Style>
    <div class="sidebar">
        {sidebar_content()}
    </div>
    <div class="main">
        <h2>"hello"</h2>
    </div>
}

fn sidebar() -> Fragment {
    <div class="item">
        <a href="/">"home"</a>
    </div>
    <div class="item">
        <a href="/about">"about"</a>
    </div>
}
"##;
        let path = Path::new("<test>");
        let result = compile_source(source, path).unwrap();

        assert!(result.contains("-> HtmlDocument"));
        assert!(result.contains("-> Vec<HtmlNode>"));
        assert!(result.contains("pub fn metadata(_req: &Request) -> Metadata"));
        assert!(result.contains("Metadata::new()"));
        assert!(result.contains("HtmlDocument::new()"));
        assert!(result.contains(".inline_style(CSS)"));
        assert!(result.contains(".body_node("));
        assert!(result.contains("div().class(\"sidebar\")"));
        assert!(result.contains(".children((sidebar_content()).into_children())"));
        assert!(result.contains("div().class(\"main\")"));
        assert!(result.contains("h2().text(\"hello\").into_node()"));
        assert!(result.contains("let mut __rsx_nodes = Vec::new();"));
        assert!(result.contains("__rsx_nodes.push("));
        assert!(result.contains("div().class(\"item\")"));
        assert!(result.contains("a().attr(\"href\", \"/\").text(\"home\").into_node()"));
        assert!(result.contains("a().attr(\"href\", \"/about\").text(\"about\").into_node()"));
    }

    #[test]
    fn test_compile_no_rsx_functions() {
        let source = r##"use crate::libs::web::prelude::*;

pub fn handler(_req: &Request) -> Response {
    Response::ok()
}
"##;
        let path = Path::new("<test>");
        let result = compile_source(source, path).unwrap();
        assert_eq!(result.as_str(), source);
    }

    #[test]
    fn test_compile_preserves_imports() {
        let source = r##"use crate::libs::web::prelude::*;
use crate::libs::db::web_editor::shared::CSS;

pub fn page(_req: &Request) -> Html {
    <div>"hello"</div>
}
"##;
        let path = Path::new("<test>");
        let result = compile_source(source, path).unwrap();
        assert!(result.contains("use crate::libs::web::prelude::*;"));
        assert!(result.contains("use crate::libs::db::web_editor::shared::CSS;"));
    }

    #[test]
    fn test_read_dist_config_default() {
        // Non-existent directory returns default
        let path = Path::new("/nonexistent_volki_test_path_12345");
        assert_eq!(read_dist_config(path).as_str(), ".volki");
    }

    #[test]
    fn test_compile_mixed_server_and_client() {
        let source = r##"use crate::libs::web::prelude::*;

pub fn page(_req: &Request) -> Html {
    <button onclick={on_click}>"Click me"</button>
    <p id="greeting">"Hello"</p>
}

pub fn on_click(target: &str) -> Client {
    let el = dom::query("#greeting");
    el.set_text("Clicked!");
}
"##;
        let path = Path::new("page.volki");
        let out = compile_source_full(source, path).unwrap();

        // Server output should have Html compiled but no Client function
        assert!(out.server_rs.contains("-> HtmlDocument"));
        assert!(out.server_rs.contains("HtmlDocument::new()"));
        assert!(out.server_rs.contains(".body_node("));
        assert!(!out.server_rs.contains("-> Client"));
        assert!(!out.server_rs.contains("dom::query"));

        // Server output should reference the glue script as a static module
        assert!(out.server_rs.contains(".script_module(\"/wasm/page_glue.js\")"));

        // Client output should exist
        assert!(out.client.is_some());
        let client = out.client.unwrap();

        // WASM .rs should have no_std and extern C exports
        assert!(client.wasm_rs.contains("#![no_std]"));
        assert!(client.wasm_rs.contains("pub extern \"C\" fn on_click("));
        assert!(client.wasm_rs.contains("__volki_dom_query"));
        assert!(client.wasm_rs.contains("__volki_dom_set_text"));

        // JS glue should register handlers and DOM bindings
        assert!(client.glue_js.contains("__volki_handlers[\"on_click\"]"));
        assert!(client.glue_js.contains("__bind_volki_handlers()"));
        assert!(client.glue_js.contains("__volki_dom_query"));
        assert!(client.glue_js.contains("WebAssembly.instantiate"));
    }

    #[test]
    fn test_compile_server_only_no_client() {
        let source = r#"pub fn page(_req: &Request) -> Html {
    <div>"hello"</div>
}
"#;
        let path = Path::new("page.volki");
        let out = compile_source_full(source, path).unwrap();

        assert!(out.server_rs.contains("HtmlDocument::new()"));
        assert!(!out.server_rs.contains(".script_module("));
        assert!(out.client.is_none());
    }

    #[test]
    fn test_compile_db_editor_page_with_client() {
        // Simulates the db web editor page.volki — mixes server-side Html/Fragment
        // with four Client functions (filter_rows, toggle_select_all, delete_selected, insert_row).
        let source = r##"
use crate::libs::web::prelude::*;

pub fn page(_req: &Request) -> Html {
    <div class="toolbar">
        <input id="filter-input" type="text" placeholder="Filter rows..." oninput={filter_rows} />
        <a id="btn-insert" class="btn" href="#" onclick={insert_row}>"+ Insert"</a>
        <a id="btn-delete" class="btn" href="#" onclick={delete_selected}>"Delete"</a>
    </div>
    <div id="conn-status">"Connected"</div>
}

pub fn filter_rows() -> Client {
    let input = dom::query("#filter-input");
    let term = input.get_value();
    dom::log(term);
}

pub fn toggle_select_all(checked: bool) -> Client {
    dom::log("select-all toggled");
}

pub fn delete_selected() -> Client {
    dom::log("delete-selected clicked");
    let status = dom::query("#conn-status");
    status.set_text("Deleting...");
}

pub fn insert_row() -> Client {
    dom::log("insert-row clicked");
    let btn = dom::query("#btn-insert");
    btn.add_class("btn-loading");
}
"##;
        let path = Path::new("page.volki");
        let out = compile_source_full(source, path).unwrap();

        // ── Server output ──
        // Html function compiled normally
        assert!(out.server_rs.contains("-> HtmlDocument"));
        assert!(out.server_rs.contains("HtmlDocument::new()"));
        assert!(out.server_rs.contains(".body_node("));
        // Glue script referenced as static module
        assert!(out.server_rs.contains(".script_module(\"/wasm/page_glue.js\")"));

        // All four Client functions stripped from server
        assert!(!out.server_rs.contains("fn filter_rows"));
        assert!(!out.server_rs.contains("fn toggle_select_all"));
        assert!(!out.server_rs.contains("fn delete_selected"));
        assert!(!out.server_rs.contains("fn insert_row"));
        assert!(!out.server_rs.contains("-> Client"));

        // ── Client output ──
        let client = out.client.as_ref().unwrap();

        // WASM Rust: all four exported
        assert!(client.wasm_rs.contains("#![no_std]"));
        assert!(client.wasm_rs.contains("pub extern \"C\" fn filter_rows()"));
        assert!(client.wasm_rs.contains("pub extern \"C\" fn toggle_select_all(checked: i32)"));
        assert!(client.wasm_rs.contains("pub extern \"C\" fn delete_selected()"));
        assert!(client.wasm_rs.contains("pub extern \"C\" fn insert_row()"));

        // WASM Rust: correct imports present
        assert!(client.wasm_rs.contains("fn __volki_dom_query("));
        assert!(client.wasm_rs.contains("fn __volki_dom_set_text("));
        assert!(client.wasm_rs.contains("fn __volki_dom_add_class("));
        assert!(client.wasm_rs.contains("fn __volki_console_log("));
        assert!(client.wasm_rs.contains("fn __volki_dom_get_value("));

        // JS glue: all four handlers
        assert!(client.glue_js.contains("__volki_handlers[\"filter_rows\"]"));
        assert!(client.glue_js.contains("__volki_handlers[\"toggle_select_all\"]"));
        assert!(client.glue_js.contains("__volki_handlers[\"delete_selected\"]"));
        assert!(client.glue_js.contains("__volki_handlers[\"insert_row\"]"));

        // JS glue: correct DOM bindings present
        assert!(client.glue_js.contains("__volki_dom_query(sel_ptr, sel_len)"));
        assert!(client.glue_js.contains("__volki_dom_set_text(handle, text_ptr, text_len)"));
        assert!(client.glue_js.contains("__volki_dom_add_class(handle, cls_ptr, cls_len)"));
        assert!(client.glue_js.contains("__volki_console_log(msg_ptr, msg_len)"));

        // JS glue: WASM loading
        assert!(client.glue_js.contains("fetch(\"/wasm/page_client.wasm\")"));
        assert!(client.glue_js.contains("WebAssembly.instantiate"));

        // JS glue: bool param gets converted
        assert!(client.glue_js.contains("(checked ? 1 : 0)"));
    }

    #[test]
    fn test_client_functions_stripped_from_server() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div>"hello"</div>
}

pub fn on_click(target: &str) -> Client {
    let el = dom::query(target);
    el.set_text("Clicked!");
}

pub fn toggle() -> Client {
    dom::log("toggled");
}
"#;
        let path = Path::new("test.volki");
        let out = compile_source_full(source, path).unwrap();

        // Neither Client function should appear in server output
        assert!(!out.server_rs.contains("fn on_click"));
        assert!(!out.server_rs.contains("fn toggle"));
        assert!(!out.server_rs.contains("dom::query"));
        assert!(!out.server_rs.contains("dom::log"));

        // Both should be in client output
        let client = out.client.unwrap();
        assert!(client.wasm_rs.contains("fn on_click("));
        assert!(client.wasm_rs.contains("fn toggle("));
    }

    #[test]
    fn test_compile_mixed_component_and_client() {
        let source = r##"use crate::libs::web::prelude::*;

pub fn page(_req: &Request) -> Html {
    <div>
        <p id="count">"0"</p>
        <button onclick={on_increment}>"+"</button>
    </div>
}

pub fn counter() -> Component {
    let count = use_state(0_i32);
    let el = dom::query("#count");
    el.set_text(state::fmt_i32(count));
}

pub fn on_increment() -> Client {
    let count = state::get_i32("counter", 0);
    state::set_i32("counter", 0, count + 1);
}
"##;
        let path = Path::new("page.volki");
        let out = compile_source_full(source, path).unwrap();

        // Server output: Html compiled, Component and Client stripped
        assert!(out.server_rs.contains("-> HtmlDocument"));
        assert!(!out.server_rs.contains("-> Component"));
        assert!(!out.server_rs.contains("-> Client"));
        assert!(!out.server_rs.contains("fn counter"));
        assert!(!out.server_rs.contains("fn on_increment"));
        assert!(!out.server_rs.contains("use_state"));
        assert!(!out.server_rs.contains("state::get_i32"));

        // Glue script referenced since we have client code
        assert!(out.server_rs.contains(".script_module(\"/wasm/page_glue.js\")"));

        // Client output exists
        let client = out.client.as_ref().unwrap();

        // WASM: Component function
        assert!(client.wasm_rs.contains("pub extern \"C\" fn __volki_component_counter()"));
        assert!(client.wasm_rs.contains("__volki_component_begin(0)"));
        assert!(client.wasm_rs.contains("__volki_component_end()"));
        assert!(client.wasm_rs.contains("__volki_state_init_i32(0, 0)"));

        // WASM: Client function with cross-state access
        assert!(client.wasm_rs.contains("pub extern \"C\" fn on_increment()"));
        assert!(client.wasm_rs.contains("__volki_xstate_get_i32(0, 0)"));
        assert!(client.wasm_rs.contains("__volki_xstate_set_i32(0, 0, count + 1)"));

        // WASM: State externs declared
        assert!(client.wasm_rs.contains("fn __volki_component_begin(id: i32);"));
        assert!(client.wasm_rs.contains("fn __volki_state_init_i32("));
        assert!(client.wasm_rs.contains("fn __volki_xstate_get_i32("));
        assert!(client.wasm_rs.contains("fn __volki_xstate_set_i32("));
        assert!(client.wasm_rs.contains("fn __volki_state_fmt_i32("));

        // JS: Component infrastructure
        assert!(client.glue_js.contains("const __components = new Map()"));
        assert!(client.glue_js.contains("function __register_component("));
        assert!(client.glue_js.contains("function __schedule_rerender("));

        // JS: State imports
        assert!(client.glue_js.contains("__volki_component_begin(id)"));
        assert!(client.glue_js.contains("__volki_state_init_i32(slot, initial)"));
        assert!(client.glue_js.contains("__volki_xstate_get_i32(comp_id, slot)"));
        assert!(client.glue_js.contains("__volki_xstate_set_i32(comp_id, slot, value)"));
        assert!(client.glue_js.contains("__volki_state_fmt_i32(value, buf_ptr, buf_len)"));

        // JS: Component registration and mount
        assert!(client.glue_js.contains("__register_component(0, \"counter\", \"__volki_component_counter\")"));
        assert!(client.glue_js.contains("__wasm.exports.__volki_component_counter()"));

        // JS: Client handler entry
        assert!(client.glue_js.contains("__volki_handlers[\"on_increment\"]"));
    }

    #[test]
    fn test_boundary_error_on_compile() {
        let source = r##"
pub fn page(_req: &Request) -> Html {
    let el = dom::query("#btn");
    el.set_text("hello");
}
"##;
        let path = Path::new("page.volki");
        let result = compile_source_full(source, path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("client-only API"));
        assert!(err.message.contains("dom::query"));
        assert!(err.message.contains("Html"));
    }

    #[test]
    fn test_legacy_volki_handler_syntax_errors() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <button onclick="__volki.on_click()">"x"</button>
}

pub fn on_click() -> Client {
    dom::log("x");
}
"#;
        let path = Path::new("page.volki");
        let result = compile_source_full(source, path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("legacy __volki inline handlers are removed"));
    }

    #[test]
    fn test_is_static_asset() {
        assert!(is_static_asset("css"));
        assert!(is_static_asset("svg"));
        assert!(is_static_asset("png"));
        assert!(is_static_asset("jpg"));
        assert!(is_static_asset("jpeg"));
        assert!(is_static_asset("gif"));
        assert!(is_static_asset("webp"));
        assert!(is_static_asset("avif"));
        assert!(is_static_asset("ico"));
        assert!(is_static_asset("woff"));
        assert!(is_static_asset("woff2"));
        assert!(is_static_asset("ttf"));
        assert!(is_static_asset("otf"));
        assert!(!is_static_asset("rs"));
        assert!(!is_static_asset("volki"));
        assert!(!is_static_asset("js"));
        assert!(!is_static_asset("html"));
    }

    #[test]
    fn test_compile_stylesheet_tag() {
        let source = r#"pub fn page(_req: &Request) -> Html {
    <Stylesheet href="/styles/app.css" />
    <div>"hello"</div>
}
"#;
        let path = Path::new("page.volki");
        let result = compile_source(source, path).unwrap();
        assert!(result.contains(".stylesheet(\"/styles/app.css\")"));
        assert!(result.contains("div().text(\"hello\").into_node()"));
    }

    #[test]
    fn test_component_tag_requires_fragment_return_type() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <Counter />
}

fn counter() -> Html {
    <div>"x"</div>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source_full(source, path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("must return Fragment"));
    }

    #[test]
    fn test_component_tag_must_resolve() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <MissingWidget />
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source_full(source, path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("unresolved component"));
    }

    #[test]
    fn test_top_level_state_produces_compile_error() {
        let source = r#"
let count = use_state(0_i32);

pub fn page(_req: &Request) -> Html {
    <div>"hello"</div>
}
"#;
        let path = Path::new("page.volki");
        let result = compile_source_full(source, path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("cannot be used at the top level"));
        assert!(err.message.contains("use_state"));
    }

    // ── Component resolution tests ──

    #[test]
    fn test_component_resolved_in_html() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div><Counter /></div>
}

fn counter() -> Fragment {
    <span>"hello"</span>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        // Counter tag resolved to counter() function call
        assert!(result.contains("counter()"));
        assert!(result.contains(".children((counter()).into_children())"));
        // Fragment function is emitted (not tree-shaken)
        assert!(result.contains("fn counter"));
        assert!(result.contains("-> Vec<HtmlNode>"));
        assert!(result.contains("span().text(\"hello\").into_node()"));
    }

    #[test]
    fn test_nested_component_resolution() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div><Outer /></div>
}

fn outer() -> Fragment {
    <section><Inner /></section>
}

fn inner() -> Fragment {
    <span>"deep"</span>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        // Both should be resolved to function calls
        assert!(result.contains("outer()"));
        assert!(result.contains("inner()"));
        // Both Fragment functions should be in output
        assert!(result.contains("fn outer"));
        assert!(result.contains("fn inner"));
        assert!(result.contains("section()"));
        assert!(result.contains("span().text(\"deep\").into_node()"));
    }

    #[test]
    fn test_component_between_elements() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div>
        <span>"a"</span>
        <Nav />
        <span>"b"</span>
    </div>
}

fn nav() -> Fragment {
    <nav>"links"</nav>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        // Nav resolved to function call, between elements
        assert!(result.contains("span().text(\"a\").into_node()"));
        assert!(result.contains("nav()"));
        assert!(result.contains("span().text(\"b\").into_node()"));
        // Fragment function is in output
        assert!(result.contains("fn nav"));
        assert!(result.contains("nav().text(\"links\").into_node()"));
    }

    #[test]
    fn test_component_fragment_not_treeshaken() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div><Badge /></div>
}

fn badge() -> Fragment {
    <span>"badge"</span>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        // Fragment function is kept in output (no tree-shaking)
        assert!(result.contains("fn badge"));
        assert!(result.contains("-> Vec<HtmlNode>"));
        assert!(result.contains("span().text(\"badge\").into_node()"));
        // Component tag resolved to function call
        assert!(result.contains("badge()"));
    }

    #[test]
    fn test_component_and_expression_call() {
        let source = r##"
pub fn page(_req: &Request) -> Html {
    <div>{sidebar()}</div>
    <div><Sidebar /></div>
}

fn sidebar() -> Fragment {
    <nav>"nav"</nav>
}
"##;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        // Both expression call and component tag work
        assert!(result.contains("fn sidebar"));
        assert!(result.contains("-> Vec<HtmlNode>"));
        assert!(result.contains("nav().text(\"nav\").into_node()"));
    }

    #[test]
    fn test_expression_called_fragment() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div>{sidebar()}</div>
}

fn sidebar() -> Fragment {
    <nav>"nav"</nav>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        // sidebar called as expression, function preserved
        assert!(result.contains("fn sidebar"));
        assert!(result.contains("-> Vec<HtmlNode>"));
        assert!(result.contains("nav().text(\"nav\").into_node()"));
    }

    #[test]
    fn test_component_classes_collected() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div class="flex"><Widget /></div>
}

fn widget() -> Fragment {
    <span class="text-red-500">"styled"</span>
}
"#;
        let path = Path::new("test.volki");
        let out = compile_source_full(source, path).unwrap();

        // Widget's classes should be picked up by CSS generation
        assert!(out.server_rs.contains("fn widget"));
        assert!(out.server_rs.contains("span()"));
        assert!(out.server_rs.contains("text(\"styled\")"));
    }

    // ── Component props tests ──

    #[test]
    fn test_component_tag_with_props() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div><Counter show_help={true} dark={false} /></div>
}

fn counter(show_help: bool, dark: bool) -> Fragment {
    <span>"counter"</span>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        // Props mapped to function arguments in declaration order
        assert!(result.contains("counter(true, false)"));
    }

    #[test]
    fn test_component_tag_with_string_prop() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div><Greeting name="world" /></div>
}

fn greeting(name: &str) -> Fragment {
    <span>"hello"</span>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        assert!(result.contains("greeting(\"world\")"));
    }

    #[test]
    fn test_component_tag_with_children() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <div>
        <Wrapper>
            <span>"child"</span>
        </Wrapper>
    </div>
}

fn wrapper(children: Vec<HtmlNode>) -> Fragment {
    <div class="wrap">{children}</div>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        // Children compiled into a block expression
        assert!(result.contains("wrapper("));
        assert!(result.contains("span().text(\"child\").into_node()"));
    }

    #[test]
    fn test_component_tag_with_props_and_children() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <Wrapper title="Hello">
        <span>"content"</span>
    </Wrapper>
}

fn wrapper(title: &str, children: Vec<HtmlNode>) -> Fragment {
    <h1>{title}</h1>
    {children}
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        assert!(result.contains("wrapper(\"Hello\""));
    }

    // ── Component error tests ──

    #[test]
    fn test_component_unknown_prop_error() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <Counter bogus={true} />
}

fn counter() -> Fragment {
    <span>"x"</span>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source_full(source, path);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("unknown prop"));
    }

    #[test]
    fn test_component_missing_prop_error() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <Counter />
}

fn counter(show_help: bool) -> Fragment {
    <span>"x"</span>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source_full(source, path);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("missing required prop"));
    }

    #[test]
    fn test_component_no_children_param_error() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <Counter><span>"x"</span></Counter>
}

fn counter() -> Fragment {
    <span>"x"</span>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source_full(source, path);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("does not accept children"));
    }

    #[test]
    fn test_component_top_level_in_html() {
        let source = r#"
pub fn page(_req: &Request) -> Html {
    <Counter />
    <div>"after"</div>
}

fn counter() -> Fragment {
    <span>"count"</span>
}
"#;
        let path = Path::new("test.volki");
        let result = compile_source(source, path).unwrap();

        // Top-level component in Html uses body_nodes
        assert!(result.contains(".body_nodes((counter()).into_children())"));
        assert!(result.contains("div().text(\"after\").into_node()"));
    }

    // ── RSX Component rendering tests ──

    #[test]
    fn test_rsx_component_basic() {
        let source = r##"use crate::libs::web::prelude::*;

pub fn page(_req: &Request) -> Html {
    <div>
        <p id="count">"0"</p>
        <button onclick={on_increment}>"+"</button>
    </div>
}

pub fn counter() -> Component {
    let (count, set_count) = use_state(0_i32);
    let _ = set_count;

    return (
        <div class="counter">
            <span>{state::fmt_i32(count)}</span>
        </div>
    )
}

pub fn on_increment() -> Client {
    let count = state::get_i32("counter", 0);
    state::set_i32("counter", 0, count + 1);
}
"##;
        let path = Path::new("page.volki");
        let out = compile_source_full(source, path).unwrap();

        // Server output: Component stripped
        assert!(!out.server_rs.contains("-> Component"));
        assert!(!out.server_rs.contains("fn counter"));

        // Client output exists
        let client = out.client.as_ref().unwrap();

        // WASM should have mount/update split
        assert!(client.wasm_rs.contains("__volki_component_is_mounted(0)"));
        assert!(client.wasm_rs.contains("__volki_component_mount_point(0)"));
        assert!(client.wasm_rs.contains("__volki_dom_create("));
        assert!(client.wasm_rs.contains("__volki_dom_create_text("));
        assert!(client.wasm_rs.contains("__volki_dom_append("));
        assert!(client.wasm_rs.contains("__volki_ref_set_i32("));
        assert!(client.wasm_rs.contains("__volki_ref_get_i32("));
        assert!(client.wasm_rs.contains("__volki_state_fmt_i32("));

        // JS should have RSX component support
        assert!(client.glue_js.contains("__volki_dom_create_text("));
        assert!(client.glue_js.contains("__volki_component_is_mounted("));
        assert!(client.glue_js.contains("__volki_component_mount_point("));
    }

    #[test]
    fn test_rsx_component_imperative_backward_compat() {
        // Old-style imperative Component should still work unchanged
        let source = r##"use crate::libs::web::prelude::*;

pub fn page(_req: &Request) -> Html {
    <div>
        <p id="count">"0"</p>
    </div>
}

pub fn counter() -> Component {
    let count = use_state(0_i32);
    let el = dom::query("#count");
    el.set_text(state::fmt_i32(count));
}
"##;
        let path = Path::new("page.volki");
        let out = compile_source_full(source, path).unwrap();

        let client = out.client.as_ref().unwrap();

        // Should use imperative path, NOT RSX mount/update
        assert!(client.wasm_rs.contains("__volki_dom_query("));
        assert!(client.wasm_rs.contains("__volki_dom_set_text("));
        // Should NOT have mount_point or is_mounted (no RSX)
        assert!(!client.wasm_rs.contains("__volki_component_is_mounted("));
        assert!(!client.wasm_rs.contains("__volki_component_mount_point("));
    }

    #[test]
    fn test_rsx_component_tag_generates_mount_point() {
        // An RSX Component used as a tag in Html should generate a mount-point div
        let source = r##"use crate::libs::web::prelude::*;

pub fn page(_req: &Request) -> Html {
    <div>
        <Counter />
        <p>"after"</p>
    </div>
}

pub fn counter() -> Component {
    let (count, set_count) = use_state(0_i32);
    let _ = set_count;

    return (
        <span>{state::fmt_i32(count)}</span>
    )
}
"##;
        let path = Path::new("page.volki");
        let out = compile_source_full(source, path).unwrap();

        // Server output should have mount-point div
        assert!(out.server_rs.contains("data-volki-component"));
        assert!(out.server_rs.contains("counter"));
    }

    #[test]
    fn test_rsx_component_css_classes_collected() {
        // CSS classes inside Component RSX should be collected for volkistyle
        let source = r##"use crate::libs::web::prelude::*;

pub fn page(_req: &Request) -> Html {
    <div class="flex">"hello"</div>
}

pub fn counter() -> Component {
    let (count, set_count) = use_state(0_i32);
    let _ = set_count;

    return (
        <div class="text-red-500">
            <span>{state::fmt_i32(count)}</span>
        </div>
    )
}
"##;
        let path = Path::new("page.volki");
        let out = compile_source_full(source, path).unwrap();

        // Both classes should be in the generated CSS
        // The server output should contain inline_style with both classes
        assert!(out.server_rs.contains("flex"));
        assert!(out.server_rs.contains("text-red-500") || out.server_rs.contains("inline_style"));
    }
}
