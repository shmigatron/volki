//! Route discovery — scans a web app directory for file-based routes.
//!
//! Conventions:
//! - `app/page.volki` → page handler at `/`
//! - `app/about/page.volki` → page handler at `/about`
//! - `app/not_found.volki` → 404 handler
//! - `app/api/tables/route.volki` or `route.rs` → API route at `/api/tables` (scans for pub fn get/post/etc.)
//! - Other `.volki`/`.rs` files → utility modules (e.g., `shared.volki`)

use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::Path;

use super::CompileError;

/// The kind of discovered route.
pub enum RouteKind {
    /// A page route (`page.volki` or `page.rs`) — rendered with `HtmlDocument`.
    Page,
    /// A 404 handler (`not_found.volki` or `not_found.rs`).
    NotFound,
    /// An API route (`route.volki` or `route.rs`) with per-method handlers.
    Api,
}

/// A route discovered from the file system.
pub struct DiscoveredRoute {
    pub kind: RouteKind,
    pub url_path: String,
    pub module_path: String,
    pub methods: Vec<String>,
    pub has_metadata: bool,
}

const HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch", "head"];

/// Discover all routes under a web app root directory.
///
/// Looks for an `app/` subdirectory and scans it recursively for
/// `page.volki`/`page.rs`, `not_found.volki`/`not_found.rs`, and `route.volki`/`route.rs` files.
pub fn discover_routes(root: &Path) -> Result<Vec<DiscoveredRoute>, CompileError> {
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
    routes: &mut Vec<DiscoveredRoute>,
) -> Result<(), CompileError> {
    // Page route: page.volki or page.rs
    let page_volki = dir.join("page.volki");
    let page_rs = dir.join("page.rs");
    if page_volki.as_path().exists() || page_rs.as_path().exists() {
        let url = dir_to_url(dir, root);
        let mut module = dir_to_module(dir, root);
        module.push_str("::page");

        let source_path = if page_volki.as_path().exists() {
            page_volki
        } else {
            page_rs
        };
        let source = fs::read_to_string(source_path.as_path()).unwrap_or_else(|_| String::new());
        let has_metadata = source.as_str().contains("pub fn metadata");

        routes.push(DiscoveredRoute {
            kind: RouteKind::Page,
            url_path: url,
            module_path: module,
            methods: Vec::new(),
            has_metadata,
        });
    }

    // Not-found handler: not_found.volki or not_found.rs
    let nf_volki = dir.join("not_found.volki");
    let nf_rs = dir.join("not_found.rs");
    if nf_volki.as_path().exists() || nf_rs.as_path().exists() {
        let mut module = dir_to_module(dir, root);
        module.push_str("::not_found");
        routes.push(DiscoveredRoute {
            kind: RouteKind::NotFound,
            url_path: String::new(),
            module_path: module,
            methods: Vec::new(),
            has_metadata: false,
        });
    }

    // API route: route.volki or route.rs with pub fn get/post/etc.
    let route_volki = dir.join("route.volki");
    let route_rs = dir.join("route.rs");
    if route_volki.as_path().exists() || route_rs.as_path().exists() {
        let url = dir_to_url(dir, root);
        let mut module = dir_to_module(dir, root);
        module.push_str("::route");

        let source_path = if route_volki.as_path().exists() {
            route_volki
        } else {
            route_rs
        };
        let source = fs::read_to_string(source_path.as_path()).unwrap_or_else(|_| String::new());
        let mut methods = Vec::new();
        for method in HTTP_METHODS {
            let pattern = crate::vformat!("pub fn {}(", method);
            if source.as_str().contains(pattern.as_str()) {
                methods.push(String::from(*method));
            }
        }

        if !methods.is_empty() {
            routes.push(DiscoveredRoute {
                kind: RouteKind::Api,
                url_path: url,
                module_path: module,
                methods,
                has_metadata: false,
            });
        }
    }

    // Recurse into subdirectories
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
            scan_dir(entry.path(), root, routes)?;
        }
    }

    Ok(())
}

/// Generate `mod.rs` content for a directory — declares all sub-modules.
pub fn generate_mod_file(dir: &Path) -> Result<String, CompileError> {
    let mut module_names: Vec<String> = Vec::new();

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

        let name = String::from(entry.file_name());

        if entry.file_type() == fs::FileType::Directory {
            if !name.as_str().starts_with('.') && name.as_str() != "public" {
                add_unique(&mut module_names, name);
            }
        } else if entry.file_type() == fs::FileType::File {
            if name.as_str() == "mod.rs" {
                continue;
            }
            let path = entry.path();
            let ext = path.extension();
            if ext == Some("rs") || ext == Some("volki") {
                if let Some(stem) = path.file_stem() {
                    add_unique(&mut module_names, String::from(stem));
                }
            }
        }
    }

    module_names.sort();

    let mut out = String::from("//! @generated by volki compiler \u{2014} do not edit.\n\n");
    for m in module_names.iter() {
        out.push_str("pub mod ");
        out.push_str(m.as_str());
        out.push_str(";\n");
    }

    Ok(out)
}

/// A client asset discovered in `public/wasm/`.
struct WasmAsset {
    filename: String,
    const_name: String,
    fn_name: String,
    is_wasm: bool,
}

/// Scan `root/public/wasm/` for `.js` and `.wasm` files.
fn discover_wasm_assets(root: &Path) -> Vec<WasmAsset> {
    let wasm_dir = root.join("public").join("wasm");
    let mut assets = Vec::new();
    if !wasm_dir.as_path().exists() {
        return assets;
    }
    let entries = match fs::read_dir(wasm_dir.as_path()) {
        Ok(e) => e,
        Err(_) => return assets,
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = String::from(entry.file_name());
        let path = entry.path();
        let is_js = path.extension() == Some("js");
        let is_wasm = path.extension() == Some("wasm");
        if !is_js && !is_wasm {
            continue;
        }
        let const_name = filename_to_const(name.as_str());
        let fn_name = filename_to_fn(name.as_str());
        assets.push(WasmAsset { filename: name, const_name, fn_name, is_wasm });
    }
    assets
}

/// Convert filename to SCREAMING_SNAKE_CASE const name (e.g. `page_glue.js` → `PAGE_GLUE_JS`).
fn filename_to_const(name: &str) -> String {
    let mut out = String::new();
    for &b in name.as_bytes() {
        if b == b'.' || b == b'-' {
            out.push('_');
        } else if b >= b'a' && b <= b'z' {
            out.push((b - b'a' + b'A') as char);
        } else {
            out.push(b as char);
        }
    }
    out
}

/// Convert filename to snake_case fn name (e.g. `page_glue.js` → `__serve_page_glue_js`).
fn filename_to_fn(name: &str) -> String {
    let mut out = String::from("__serve_");
    for &b in name.as_bytes() {
        if b == b'.' || b == b'-' {
            out.push('_');
        } else if b >= b'A' && b <= b'Z' {
            out.push((b - b'A' + b'a') as char);
        } else {
            out.push(b as char);
        }
    }
    out
}

/// Generate the root `mod.rs` with module declarations and a `start()` function.
///
/// If `public/wasm/` contains `.js` or `.wasm` files, generates `include_str!`/`include_bytes!`
/// constants and API handler functions to serve them, so assets are embedded in the binary.
pub fn generate_root_mod(
    root: &Path,
    routes: &[DiscoveredRoute],
    public_dir: Option<&str>,
) -> Result<String, CompileError> {
    let mod_content = generate_mod_file(root)?;

    let mut out = mod_content;
    out.push_str("\nuse crate::libs::web::server::Server;\n");

    let has_api = routes.iter().any(|r| matches!(r.kind, RouteKind::Api));
    if has_api {
        out.push_str("use crate::libs::web::router::file_route::FileRoute;\n");
    }

    // Discover embedded client assets
    let assets = discover_wasm_assets(root);
    if !assets.is_empty() {
        out.push_str("use crate::libs::web::http::request::Request;\n");
        out.push_str("use crate::libs::web::http::response::Response;\n");
        out.push('\n');

        // Const declarations
        for asset in &assets {
            if asset.is_wasm {
                out.push_str("const ");
                out.push_str(asset.const_name.as_str());
                out.push_str(": &[u8] = include_bytes!(\"public/wasm/");
                out.push_str(asset.filename.as_str());
                out.push_str("\");\n");
            } else {
                out.push_str("const ");
                out.push_str(asset.const_name.as_str());
                out.push_str(": &str = include_str!(\"public/wasm/");
                out.push_str(asset.filename.as_str());
                out.push_str("\");\n");
            }
        }
        out.push('\n');

        // Handler functions
        for asset in &assets {
            out.push_str("fn ");
            out.push_str(asset.fn_name.as_str());
            out.push_str("(_req: &Request) -> Response {\n");
            out.push_str("    Response::new(crate::libs::web::http::status::StatusCode::OK)\n");
            if asset.is_wasm {
                out.push_str("        .body_bytes(");
                out.push_str(asset.const_name.as_str());
                out.push_str(")\n");
                out.push_str("        .header(\"Content-Type\", \"application/wasm\")\n");
            } else {
                out.push_str("        .body_bytes(");
                out.push_str(asset.const_name.as_str());
                out.push_str(".as_bytes())\n");
                out.push_str("        .header(\"Content-Type\", \"application/javascript; charset=utf-8\")\n");
            }
            out.push_str("        .header(\"Cache-Control\", \"public, max-age=3600\")\n");
            out.push_str("}\n\n");
        }
    }

    out.push_str("pub fn start(host: &str, port: u16) -> ! {\n");
    out.push_str("    Server::new()\n");
    out.push_str("        .host(host)\n");
    out.push_str("        .port(port)\n");

    // Static asset serving from public directory
    if let Some(dir) = public_dir {
        out.push_str("        .public_dir(\"");
        out.push_str(dir);
        out.push_str("\")\n");
    }

    // Embedded asset routes
    for asset in &assets {
        out.push_str("        .api(\"/wasm/");
        out.push_str(asset.filename.as_str());
        out.push_str("\", ");
        out.push_str(asset.fn_name.as_str());
        out.push_str(")\n");
    }

    // Page routes
    for route in routes {
        if let RouteKind::Page = route.kind {
            if route.has_metadata {
                out.push_str("        .page_with_metadata(\"");
                out.push_str(route.url_path.as_str());
                out.push_str("\", ");
                out.push_str(route.module_path.as_str());
                out.push_str("::page, ");
                out.push_str(route.module_path.as_str());
                out.push_str("::metadata)\n");
            } else {
                out.push_str("        .page(\"");
                out.push_str(route.url_path.as_str());
                out.push_str("\", ");
                out.push_str(route.module_path.as_str());
                out.push_str("::page)\n");
            }
        }
    }

    // API routes
    for route in routes {
        if let RouteKind::Api = route.kind {
            out.push_str("        .file_route(\n");
            out.push_str("            \"");
            out.push_str(route.url_path.as_str());
            out.push_str("\",\n");
            out.push_str("            FileRoute::new()");
            for method in route.methods.iter() {
                out.push('.');
                out.push_str(method.as_str());
                out.push('(');
                out.push_str(route.module_path.as_str());
                out.push_str("::");
                out.push_str(method.as_str());
                out.push(')');
            }
            out.push_str(",\n");
            out.push_str("        )\n");
        }
    }

    // Not-found handler (last)
    for route in routes {
        if let RouteKind::NotFound = route.kind {
            out.push_str("        .not_found_page(");
            out.push_str(route.module_path.as_str());
            out.push_str("::page)\n");
        }
    }

    out.push_str("        .listen()\n");
    out.push_str("}\n");

    Ok(out)
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

/// Convert directory path to Rust module path relative to root.
fn dir_to_module(dir: &Path, root: &Path) -> String {
    match dir.strip_prefix(root.as_str()) {
        Some(rel) if rel.is_empty() => String::new(),
        Some(rel) => {
            let mut result = String::new();
            for part in rel.split('/') {
                if part.is_empty() {
                    continue;
                }
                if !result.is_empty() {
                    result.push_str("::");
                }
                result.push_str(part);
            }
            result
        }
        None => String::new(),
    }
}

fn add_unique(vec: &mut Vec<String>, s: String) {
    for existing in vec.iter() {
        if existing.as_str() == s.as_str() {
            return;
        }
    }
    vec.push(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_to_url_root() {
        let root = Path::new("/project");
        let dir = Path::new("/project/app");
        assert_eq!(dir_to_url(dir, root).as_str(), "/");
    }

    #[test]
    fn test_dir_to_url_nested() {
        let root = Path::new("/project");
        let dir = Path::new("/project/app/api/tables");
        assert_eq!(dir_to_url(dir, root).as_str(), "/api/tables");
    }

    #[test]
    fn test_dir_to_module_root() {
        let root = Path::new("/project");
        let dir = Path::new("/project/app");
        assert_eq!(dir_to_module(dir, root).as_str(), "app");
    }

    #[test]
    fn test_dir_to_module_nested() {
        let root = Path::new("/project");
        let dir = Path::new("/project/app/api/tables");
        assert_eq!(dir_to_module(dir, root).as_str(), "app::api::tables");
    }

    #[test]
    fn test_add_unique_deduplicates() {
        let mut v = Vec::new();
        add_unique(&mut v, String::from("page"));
        add_unique(&mut v, String::from("shared"));
        add_unique(&mut v, String::from("page"));
        assert_eq!(v.len(), 2);
    }
}
