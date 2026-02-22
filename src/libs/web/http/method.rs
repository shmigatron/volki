//! HTTP method enum.

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl Method {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"GET" => Some(Method::Get),
            b"POST" => Some(Method::Post),
            b"PUT" => Some(Method::Put),
            b"DELETE" => Some(Method::Delete),
            b"PATCH" => Some(Method::Patch),
            b"HEAD" => Some(Method::Head),
            b"OPTIONS" => Some(Method::Options),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        assert_eq!(Method::from_bytes(b"GET"), Some(Method::Get));
        assert_eq!(Method::from_bytes(b"POST"), Some(Method::Post));
        assert_eq!(Method::from_bytes(b"INVALID"), None);
    }

    #[test]
    fn test_as_str() {
        assert_eq!(Method::Get.as_str(), "GET");
        assert_eq!(Method::Delete.as_str(), "DELETE");
    }
}
