//! FS-based route discovery.

use super::matcher::file_path_to_route;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::fs::FileType;
use crate::core::volkiwithstds::path::PathBuf;

pub struct DiscoveredRoute {
    pub pattern: String,
    pub file_path: PathBuf,
    pub is_api: bool,
}

pub fn discover_routes(app_dir: &str) -> Vec<DiscoveredRoute> {
    let mut routes = Vec::new();

    // Scan pages/ directory
    let mut pages_dir = PathBuf::from(app_dir);
    pages_dir.push("pages");
    if fs::is_dir(pages_dir.as_path()) {
        scan_dir(&pages_dir, &pages_dir, false, &mut routes);
    }

    // Scan api/ directory
    let mut api_dir = PathBuf::from(app_dir);
    api_dir.push("api");
    if fs::is_dir(api_dir.as_path()) {
        scan_dir(&api_dir, &api_dir, true, &mut routes);
    }

    routes
}

fn scan_dir(
    dir: &PathBuf,
    base: &PathBuf,
    is_api: bool,
    routes: &mut Vec<DiscoveredRoute>,
) {
    let entries = match fs::read_dir(dir.as_path()) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry_result in entries {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = PathBuf::from(entry.path().as_str());
        if entry.file_type() == FileType::Directory {
            scan_dir(&path, base, is_api, routes);
        } else if path.as_str().ends_with(".rs") {
            let relative = strip_prefix(path.as_str(), base.as_str());
            let pattern = if is_api {
                let route = file_path_to_route(&relative);
                let mut full = String::from("/api");
                if !route.starts_with("/") {
                    full.push('/');
                }
                if route.as_str() != "/" {
                    full.push_str(route.as_str());
                }
                full
            } else {
                file_path_to_route(&relative)
            };

            routes.push(DiscoveredRoute {
                pattern,
                file_path: path,
                is_api,
            });
        }
    }
}

fn strip_prefix(path: &str, prefix: &str) -> String {
    let p = if path.starts_with(prefix) {
        &path[prefix.len()..]
    } else {
        path
    };
    let trimmed = p.trim_start_matches('/');
    String::from(trimmed)
}
