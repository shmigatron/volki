//! HTTP request type.

use super::headers::Headers;
use super::method::Method;
use crate::core::volkiwithstds::collections::{HashMap, String, Vec};

pub struct Request {
    pub method: Method,
    pub path: String,
    pub route_path: String,
    pub query_string: String,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
}

impl Request {
    pub fn new(method: Method, path: String, headers: Headers, body: Vec<u8>) -> Self {
        let (route_path, query_string) = split_path_query(&path);
        Self {
            method,
            path,
            route_path,
            query_string,
            headers,
            body,
            params: HashMap::new(),
        }
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    pub fn query_params(&self) -> Vec<(&str, &str)> {
        let mut result = Vec::new();
        if self.query_string.is_empty() {
            return result;
        }
        for pair in self.query_string.as_str().split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = &pair[..eq_pos];
                let val = &pair[eq_pos + 1..];
                result.push((key, val));
            } else if !pair.is_empty() {
                result.push((pair, ""));
            }
        }
        result
    }

    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type")
    }
}

fn split_path_query(path: &String) -> (String, String) {
    if let Some(pos) = path.find("?") {
        let route = String::from(&path.as_str()[..pos]);
        let query = String::from(&path.as_str()[pos + 1..]);
        (route, query)
    } else {
        (path.clone(), String::new())
    }
}
