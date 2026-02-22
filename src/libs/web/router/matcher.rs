//! Route segment parsing.

use crate::core::volkiwithstds::collections::{String, Vec};

#[derive(Debug, Clone)]
pub enum RouteSegment {
    Static(String),
    Dynamic(String),
    CatchAll(String),
}

pub fn parse_route_path(pattern: &str) -> Vec<RouteSegment> {
    let mut segments = Vec::new();
    let trimmed = pattern.trim_matches('/');
    if trimmed.is_empty() {
        return segments;
    }
    for part in trimmed.split('/') {
        if part.starts_with("[...") && part.ends_with(']') {
            let name = &part[4..part.len() - 1];
            segments.push(RouteSegment::CatchAll(String::from(name)));
        } else if part.starts_with('[') && part.ends_with(']') {
            let name = &part[1..part.len() - 1];
            segments.push(RouteSegment::Dynamic(String::from(name)));
        } else {
            segments.push(RouteSegment::Static(String::from(part)));
        }
    }
    segments
}

pub fn file_path_to_route(path: &str) -> String {
    let mut result = String::from("/");
    let stripped = path
        .trim_end_matches(".rs")
        .trim_matches('/');

    if stripped.is_empty() || stripped == "index" {
        return result;
    }

    let parts: Vec<&str> = stripped.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "index" {
            continue;
        }
        if i > 0 || !result.ends_with("/") {
            result.push('/');
        }
        if part.starts_with("[...") && part.ends_with(']') {
            result.push_str(part);
        } else if part.starts_with('[') && part.ends_with(']') {
            result.push_str(part);
        } else {
            result.push_str(part);
        }
    }

    if result.is_empty() {
        result.push('/');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_static() {
        let segs = parse_route_path("/users/list");
        assert_eq!(segs.len(), 2);
        match &segs[0] {
            RouteSegment::Static(s) => assert_eq!(s.as_str(), "users"),
            _ => panic!("expected static"),
        }
    }

    #[test]
    fn test_parse_dynamic() {
        let segs = parse_route_path("/users/[id]");
        assert_eq!(segs.len(), 2);
        match &segs[1] {
            RouteSegment::Dynamic(s) => assert_eq!(s.as_str(), "id"),
            _ => panic!("expected dynamic"),
        }
    }

    #[test]
    fn test_parse_catch_all() {
        let segs = parse_route_path("/docs/[...slug]");
        assert_eq!(segs.len(), 2);
        match &segs[1] {
            RouteSegment::CatchAll(s) => assert_eq!(s.as_str(), "slug"),
            _ => panic!("expected catch-all"),
        }
    }

    #[test]
    fn test_file_path_to_route_index() {
        assert_eq!(file_path_to_route("index.rs").as_str(), "/");
    }

    #[test]
    fn test_file_path_to_route_nested() {
        assert_eq!(file_path_to_route("users/[id].rs").as_str(), "/users/[id]");
    }
}
