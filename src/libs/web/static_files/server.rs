//! Static file serving with path sanitization.

use super::mime::mime_from_extension;
use crate::core::volkiwithstds::collections::String;
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::PathBuf;
use crate::libs::web::http::response::Response;
use crate::libs::web::http::status::StatusCode;

pub fn try_serve_static(public_dir: &str, url_path: &str) -> Option<Response> {
    // Sanitize path — reject traversal and hidden files
    let clean = sanitize_path(url_path)?;

    let mut file_path = PathBuf::from(public_dir);
    file_path.push(&clean);

    // If it's a directory, try index.html
    if fs::is_dir(file_path.as_path()) {
        file_path.push("index.html");
    }

    if !fs::is_file(file_path.as_path()) {
        return None;
    }

    let data = match fs::read(file_path.as_path()) {
        Ok(d) => d,
        Err(_) => return None,
    };

    let ext = extract_extension(file_path.as_str());
    let mime = mime_from_extension(ext);

    let resp = Response::new(StatusCode::OK)
        .header("Content-Type", mime)
        .header("Cache-Control", "public, max-age=3600")
        .body_bytes(data.as_slice());

    Some(resp)
}

fn sanitize_path(url_path: &str) -> Option<String> {
    let trimmed = url_path.trim_start_matches('/');

    // Check each segment
    for segment in trimmed.split('/') {
        if segment.is_empty() {
            continue;
        }
        if segment == ".." || segment.starts_with('.') {
            return None;
        }
    }

    Some(String::from(trimmed))
}

fn extract_extension(path: &str) -> &str {
    if let Some(dot_pos) = path.rfind('.') {
        &path[dot_pos + 1..]
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_rejects_traversal() {
        assert!(sanitize_path("/../etc/passwd").is_none());
        assert!(sanitize_path("/..").is_none());
    }

    #[test]
    fn test_sanitize_rejects_hidden() {
        assert!(sanitize_path("/.hidden").is_none());
        assert!(sanitize_path("/.env").is_none());
    }

    #[test]
    fn test_sanitize_valid() {
        assert_eq!(sanitize_path("/css/style.css").unwrap().as_str(), "css/style.css");
        assert_eq!(sanitize_path("/index.html").unwrap().as_str(), "index.html");
    }

    #[test]
    fn test_extract_extension() {
        assert_eq!(extract_extension("style.css"), "css");
        assert_eq!(extract_extension("file.tar.gz"), "gz");
        assert_eq!(extract_extension("noext"), "");
    }
}
