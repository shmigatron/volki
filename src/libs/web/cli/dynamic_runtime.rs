//! Shared dynamic runtime bootstrap for dev-style web servers.

use crate::core::cli::error::CliError;
use crate::core::cli::style;
use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::{Path, PathBuf};
use crate::libs::web::interpreter::scanner::{DynamicRouteKind, discover_dynamic_routes};
use crate::libs::web::server::Server;
use crate::veprintln;

pub enum EmptyRoutesPolicy<'a> {
    WarnAndReturn,
    Error(&'a str),
}

pub struct DynamicRuntimeOptions<'a> {
    pub host: &'a str,
    pub port: u16,
    pub source_dir: &'a Path,
    pub title: &'a str,
    pub scan_prefix: Option<&'a str>,
    pub show_routes: bool,
    pub show_summary: bool,
    pub show_source_dir: bool,
    pub empty_routes: EmptyRoutesPolicy<'a>,
}

pub fn run_dynamic_runtime(opts: DynamicRuntimeOptions<'_>) -> Result<(), CliError> {
    if let Some(prefix) = opts.scan_prefix {
        veprintln!();
        veprintln!("  {} scanning .volki files...", style::dim(prefix));
    }

    let routes = discover_dynamic_routes(opts.source_dir).map_err(|e| {
        CliError::InvalidUsage(crate::vformat!("route discovery failed: {e}"))
    })?;

    if routes.is_empty() {
        match opts.empty_routes {
            EmptyRoutesPolicy::WarnAndReturn => {
                veprintln!("  {} no page.volki files found in app/", style::yellow("warn"));
                veprintln!();
                return Ok(());
            }
            EmptyRoutesPolicy::Error(message) => {
                return Err(CliError::InvalidUsage(
                    crate::core::volkiwithstds::collections::String::from(message),
                ));
            }
        }
    }

    let source_public_dir = opts.source_dir.join("public");
    let runtime_public_dir = opts.source_dir.join(".volki").join("public");

    if !runtime_public_dir.as_path().exists() {
        fs::create_dir_all(runtime_public_dir.as_path()).map_err(|e| {
            CliError::InvalidUsage(crate::vformat!(
                "failed to create public directory {}: {}",
                runtime_public_dir,
                e
            ))
        })?;
    }

    if source_public_dir.as_path().exists() {
        copy_tree(source_public_dir.as_path(), runtime_public_dir.as_path()).map_err(|e| {
            CliError::InvalidUsage(crate::vformat!(
                "failed to mirror public assets into {}: {}",
                runtime_public_dir,
                e
            ))
        })?;
    }

    let mut server = Server::new()
        .host(opts.host)
        .port(opts.port)
        .public_dir(runtime_public_dir.as_path().as_str());

    let mut page_count: usize = 0;
    let mut has_not_found = false;

    for route in routes {
        match route.kind {
            DynamicRouteKind::Page => {
                if opts.show_routes {
                    veprintln!("  {} {}", style::green("page"), route.url_path);
                }
                server = server.dynamic_page(route.url_path.as_str(), route.data);
                page_count += 1;
            }
            DynamicRouteKind::NotFound => {
                if opts.show_routes {
                    veprintln!("  {} 404 handler", style::cyan("page"));
                }
                server = server.not_found_dynamic_page(route.data);
                has_not_found = true;
            }
        }
    }

    veprintln!();
    veprintln!("  {}", opts.title);
    veprintln!("  http://{}:{}", opts.host, opts.port);
    veprintln!();
    if opts.show_source_dir {
        veprintln!("  serving from {}", opts.source_dir);
        veprintln!();
    }

    if opts.show_summary {
        let summary = crate::vformat!(
            "  {} page(s){}",
            page_count,
            if has_not_found { " + 404 handler" } else { "" }
        );
        veprintln!("  {}", style::dim(summary.as_str()));
        veprintln!("  {}", style::dim("note: complex expressions may show placeholders"));
        veprintln!("  {}", style::dim("      restart to pick up file changes"));
        veprintln!();
    }

    server.listen();
}

fn copy_tree(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| crate::vformat!("create_dir_all {}: {}", dst, e))?;

    let entries = fs::read_dir(src).map_err(|e| crate::vformat!("read_dir {}: {}", src, e))?;
    for entry in entries {
        let entry = entry.map_err(|e| crate::vformat!("read_dir entry {}: {}", src, e))?;
        let name = entry.file_name();
        let src_path = entry.path().to_path_buf();
        let dst_path: PathBuf = dst.join(name);

        if entry.file_type() == fs::FileType::Directory {
            copy_tree(src_path.as_path(), dst_path.as_path())?;
            continue;
        }

        if entry.file_type() == fs::FileType::File {
            let content = fs::read(src_path.as_path())
                .map_err(|e| crate::vformat!("read {}: {}", src_path, e))?;
            fs::write(dst_path.as_path(), content.as_slice())
                .map_err(|e| crate::vformat!("write {}: {}", dst_path, e))?;
        }
    }

    Ok(())
}
