//! HTML entity escaping.

use crate::core::volkiwithstds::collections::String;

pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn escape_attr(input: &str) -> String {
    escape_html(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<script>alert('xss')</script>").as_str(),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_escape_attr() {
        assert_eq!(
            escape_attr("value=\"hello\"").as_str(),
            "value=&quot;hello&quot;"
        );
    }

    #[test]
    fn test_no_escape_needed() {
        assert_eq!(escape_html("hello world").as_str(), "hello world");
    }

    #[test]
    fn test_ampersand() {
        assert_eq!(escape_html("a & b").as_str(), "a &amp; b");
    }
}
