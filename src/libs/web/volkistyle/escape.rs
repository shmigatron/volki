//! CSS selector escaping — backslash-escapes special characters in class names.

use crate::core::volkiwithstds::collections::String;

/// Escape special characters in a CSS class name for use in a selector.
///
/// Characters escaped: `:`, `/`, `.`, `[`, `]`, `#`, `%`, `!`, `,`, `(`, `)`, `'`, `@`
pub fn escape_selector(class: &str) -> String {
    let mut out = String::with_capacity(class.len() + 8);
    for c in class.chars() {
        match c {
            ':' | '/' | '.' | '[' | ']' | '#' | '%' | '!' | ',' | '(' | ')' | '\'' | '@' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_escaping_needed() {
        assert_eq!(escape_selector("flex").as_str(), "flex");
        assert_eq!(escape_selector("p-4").as_str(), "p-4");
        assert_eq!(escape_selector("text-red-500").as_str(), "text-red-500");
    }

    #[test]
    fn test_escape_colon() {
        assert_eq!(escape_selector("hover:text-red-500").as_str(), "hover\\:text-red-500");
        assert_eq!(escape_selector("md:flex").as_str(), "md\\:flex");
    }

    #[test]
    fn test_escape_slash() {
        assert_eq!(escape_selector("w-1/2").as_str(), "w-1\\/2");
        assert_eq!(escape_selector("bg-red-500/50").as_str(), "bg-red-500\\/50");
    }

    #[test]
    fn test_escape_brackets() {
        assert_eq!(escape_selector("w-[200px]").as_str(), "w-\\[200px\\]");
    }

    #[test]
    fn test_escape_dot() {
        assert_eq!(escape_selector("p-0.5").as_str(), "p-0\\.5");
    }

    #[test]
    fn test_escape_bang() {
        assert_eq!(escape_selector("!font-bold").as_str(), "\\!font-bold");
    }

    #[test]
    fn test_multiple_specials() {
        assert_eq!(
            escape_selector("hover:md:bg-red-500/50").as_str(),
            "hover\\:md\\:bg-red-500\\/50"
        );
    }
}
