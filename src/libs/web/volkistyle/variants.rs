//! Variant prefix parsing — responsive/state/media/attribute variants.

use crate::core::volkiwithstds::collections::{String, Vec};

use super::config::{DarkModeStrategy, VolkiStyleConfig};

/// A parsed class with variant information extracted.
pub struct ParsedClass {
    /// The bare utility name (without variant prefixes).
    pub utility: String,
    /// Pseudo-class/element selectors to append (e.g. ":hover", "::before").
    pub pseudo_classes: Vec<String>,
    /// Ancestor prefixes that wrap the selector (e.g. ".dark ", ".group:hover ").
    pub selector_prefixes: Vec<String>,
    /// Selector suffixes to append after pseudo classes.
    pub selector_suffixes: Vec<String>,
    /// Media query chain (combined with `and`).
    pub media_queries: Vec<String>,
    /// Whether `!important` should be appended to declarations.
    pub important: bool,
    /// The original full class name (for selector generation).
    pub original: String,
    /// Whether this is a custom (pass-through) class that skips resolution.
    pub is_custom: bool,
}

/// CSS rule with structured data for grouping and output.
pub struct CssRule {
    pub selector: String,
    pub declarations: String,
    pub media: Option<String>,
    pub layer: u8,
}

impl PartialEq for CssRule {
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer && self.selector.as_str() == other.selector.as_str()
    }
}

impl Eq for CssRule {}

impl PartialOrd for CssRule {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CssRule {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.layer.cmp(&other.layer) {
            core::cmp::Ordering::Equal => self.selector.cmp(&other.selector),
            ord => ord,
        }
    }
}

/// Result from the resolver — either a standard rule or one with a child selector suffix.
pub enum ResolvedUtility {
    Standard(String),
    Custom {
        selector_suffix: String,
        declarations: String,
    },
}

/// Backward-compatible variant parsing with default config.
pub fn parse_variants(class: &str) -> ParsedClass {
    parse_variants_with_config(class, &VolkiStyleConfig::default())
}

pub fn parse_variants_with_config(class: &str, config: &VolkiStyleConfig) -> ParsedClass {
    let original = String::from(class);

    let (important, rest) = if class.starts_with('!') {
        (true, &class[1..])
    } else {
        (false, class)
    };

    let parts = split_variant_chain(rest);
    if parts.len() <= 1 {
        return ParsedClass {
            utility: String::from(rest),
            pseudo_classes: Vec::new(),
            selector_prefixes: Vec::new(),
            selector_suffixes: Vec::new(),
            media_queries: Vec::new(),
            important,
            original,
            is_custom: false,
        };
    }

    let mut pseudo_classes = Vec::new();
    let mut selector_prefixes = Vec::new();
    let selector_suffixes = Vec::new();
    let mut media_queries = Vec::new();
    let mut is_custom = false;

    for prefix in &parts[..parts.len() - 1] {
        if *prefix == "custom" {
            is_custom = true;
            continue;
        }

        if let Some(mq) = responsive_media(prefix, config) {
            media_queries.push(mq);
            continue;
        }

        if let Some(mq) = max_responsive_media(prefix, config) {
            media_queries.push(mq);
            continue;
        }

        if *prefix == "dark" {
            match config.dark_mode {
                DarkModeStrategy::Media => media_queries.push(String::from("(prefers-color-scheme:dark)")),
                DarkModeStrategy::Class => selector_prefixes.push(String::from(".dark ")),
            }
            continue;
        }

        if let Some(pc) = pseudo_class(prefix) {
            pseudo_classes.push(String::from(pc));
            continue;
        }

        if let Some(pe) = pseudo_element(prefix) {
            pseudo_classes.push(String::from(pe));
            continue;
        }

        if *prefix == "group-hover" {
            selector_prefixes.push(String::from(".group:hover "));
            continue;
        }
        if *prefix == "group-focus" {
            selector_prefixes.push(String::from(".group:focus "));
            continue;
        }
        if *prefix == "peer-hover" {
            selector_prefixes.push(String::from(".peer:hover ~ "));
            continue;
        }
        if *prefix == "peer-focus" {
            selector_prefixes.push(String::from(".peer:focus ~ "));
            continue;
        }

        if config.variants.enable_group_peer_named {
            if let Some(named) = prefix.strip_prefix("group-hover/") {
                selector_prefixes.push(crate::vformat!(".group\\/{}:hover ", named));
                continue;
            }
            if let Some(named) = prefix.strip_prefix("group-focus/") {
                selector_prefixes.push(crate::vformat!(".group\\/{}:focus ", named));
                continue;
            }
            if let Some(named) = prefix.strip_prefix("peer-hover/") {
                selector_prefixes.push(crate::vformat!(".peer\\/{}:hover ~ ", named));
                continue;
            }
            if let Some(named) = prefix.strip_prefix("peer-focus/") {
                selector_prefixes.push(crate::vformat!(".peer\\/{}:focus ~ ", named));
                continue;
            }
        }

        if let Some(mq) = media_variant(prefix) {
            media_queries.push(String::from(mq));
            continue;
        }

        if let Some(v) = prefix.strip_prefix("min-") {
            if let Some(raw) = parse_bracket(v) {
                media_queries.push(crate::vformat!("(min-width:{})", raw));
                continue;
            }
        }

        if let Some(v) = prefix.strip_prefix("max-") {
            if let Some(raw) = parse_bracket(v) {
                media_queries.push(crate::vformat!("(max-width:{})", raw));
                continue;
            }
        }

        if config.variants.enable_supports {
            if let Some(v) = prefix.strip_prefix("supports-") {
                if let Some(raw) = parse_bracket(v) {
                    media_queries.push(crate::vformat!("({})", raw));
                    continue;
                }
            }
        }

        if config.variants.enable_data_aria {
            if let Some(v) = prefix.strip_prefix("data-") {
                if let Some(raw) = parse_bracket(v) {
                    pseudo_classes.push(crate::vformat!("[data-{}]", raw));
                    continue;
                }
            }
            if let Some(v) = prefix.strip_prefix("aria-") {
                if let Some(raw) = parse_bracket(v) {
                    pseudo_classes.push(crate::vformat!("[aria-{}]", raw));
                    continue;
                }
            }
        }

        break;
    }

    ParsedClass {
        utility: String::from(parts[parts.len() - 1]),
        pseudo_classes,
        selector_prefixes,
        selector_suffixes,
        media_queries,
        important,
        original,
        is_custom,
    }
}

fn split_variant_chain(input: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    for (i, ch) in input.char_indices() {
        if ch == '[' {
            depth += 1;
        } else if ch == ']' {
            depth = depth.saturating_sub(1);
        } else if ch == ':' && depth == 0 {
            out.push(&input[start..i]);
            start = i + 1;
        }
    }
    out.push(&input[start..]);
    out
}

fn responsive_media(prefix: &str, config: &VolkiStyleConfig) -> Option<String> {
    let width = config.theme.screens.get(prefix)?;
    Some(crate::vformat!("(min-width:{})", width))
}

fn max_responsive_media(prefix: &str, config: &VolkiStyleConfig) -> Option<String> {
    let key = prefix.strip_prefix("max-")?;
    let width = config.theme.screens.get(key)?;
    Some(crate::vformat!("(max-width:{})", width))
}

fn pseudo_class(prefix: &str) -> Option<&'static str> {
    match prefix {
        "hover" => Some(":hover"),
        "focus" => Some(":focus"),
        "active" => Some(":active"),
        "visited" => Some(":visited"),
        "disabled" => Some(":disabled"),
        "first" => Some(":first-child"),
        "last" => Some(":last-child"),
        "odd" => Some(":nth-child(odd)"),
        "even" => Some(":nth-child(even)"),
        "focus-within" => Some(":focus-within"),
        "focus-visible" => Some(":focus-visible"),
        "checked" => Some(":checked"),
        "required" => Some(":required"),
        "empty" => Some(":empty"),
        "open" => Some(":open"),
        _ => None,
    }
}

fn pseudo_element(prefix: &str) -> Option<&'static str> {
    match prefix {
        "placeholder" => Some("::placeholder"),
        "before" => Some("::before"),
        "after" => Some("::after"),
        "selection" => Some("::selection"),
        "marker" => Some("::marker"),
        "file" => Some("::file-selector-button"),
        _ => None,
    }
}

fn media_variant(prefix: &str) -> Option<&'static str> {
    match prefix {
        "motion-safe" => Some("(prefers-reduced-motion:no-preference)"),
        "motion-reduce" => Some("(prefers-reduced-motion:reduce)"),
        "print" => Some("print"),
        _ => None,
    }
}

fn parse_bracket(s: &str) -> Option<&str> {
    if s.starts_with('[') && s.ends_with(']') && s.len() > 2 {
        Some(&s[1..s.len() - 1])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_variants() {
        let p = parse_variants("text-red-500");
        assert_eq!(p.utility.as_str(), "text-red-500");
        assert!(p.pseudo_classes.is_empty());
        assert!(p.media_queries.is_empty());
        assert!(!p.important);
    }

    #[test]
    fn test_combined_chain() {
        let p = parse_variants("hover:md:text-red-500");
        assert_eq!(p.utility.as_str(), "text-red-500");
        assert_eq!(p.pseudo_classes[0].as_str(), ":hover");
        assert_eq!(p.media_queries[0].as_str(), "(min-width:768px)");
    }

    #[test]
    fn test_dark_class_mode() {
        let mut cfg = VolkiStyleConfig::default();
        cfg.dark_mode = DarkModeStrategy::Class;
        let p = parse_variants_with_config("dark:bg-black", &cfg);
        assert_eq!(p.selector_prefixes[0].as_str(), ".dark ");
    }

    #[test]
    fn test_max_breakpoint() {
        let p = parse_variants("max-md:hidden");
        assert_eq!(p.media_queries[0].as_str(), "(max-width:768px)");
    }

    #[test]
    fn test_attribute_variants() {
        let p = parse_variants("data-[state=open]:bg-red-500");
        assert_eq!(p.pseudo_classes[0].as_str(), "[data-state=open]");
    }

    #[test]
    fn test_custom_prefix() {
        let p = parse_variants("custom:sidebar-header");
        assert!(p.is_custom);
        assert_eq!(p.utility.as_str(), "sidebar-header");
    }

    #[test]
    fn test_no_custom_prefix() {
        let p = parse_variants("hover:bg-red-500");
        assert!(!p.is_custom);
    }
}
