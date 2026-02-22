//! Resolver — maps CSS utility class names to declarations.
//!
//! Dispatches to category sub-modules. Shared helpers live here.

pub mod backgrounds;
pub mod borders;
pub mod effects;
pub mod filters;
pub mod flexbox;
pub mod grid;
pub mod inset;
pub mod interactivity;
pub mod layout;
pub mod sizing;
pub mod spacing;
pub mod svg;
pub mod tables;
pub mod transforms;
pub mod transitions;
pub mod typography;

use crate::core::volkiwithstds::collections::String;

use super::escape::escape_selector;
use super::palette;
pub use super::variants::ResolvedUtility;

// ── Shared helpers ──────────────────────────────────────────────────────────

/// Build a complete CSS rule: `.escaped-class{declarations}`.
pub fn rule(class: &str, decls: &str) -> String {
    let escaped = escape_selector(class);
    let mut s = String::with_capacity(1 + escaped.len() + 1 + decls.len() + 1);
    s.push_str(".");
    s.push_str(escaped.as_str());
    s.push_str("{");
    s.push_str(decls);
    s.push_str("}");
    s
}

/// Build a rule with escaped selector (pre-escaped).
pub fn rule_escaped(selector: &str, decls: &str) -> String {
    let mut s = String::with_capacity(selector.len() + 1 + decls.len() + 1);
    s.push_str(selector);
    s.push_str("{");
    s.push_str(decls);
    s.push_str("}");
    s
}

/// Convert a spacing scale value to a CSS length.
/// `0` → `"0px"`, otherwise `n * 0.25rem`.
pub fn spacing(n: u32) -> String {
    if n == 0 {
        return String::from("0px");
    }
    let whole = n / 4;
    let frac = n % 4;
    match frac {
        0 => crate::vformat!("{}rem", whole),
        1 => crate::vformat!("{}.25rem", whole),
        2 => crate::vformat!("{}.5rem", whole),
        3 => crate::vformat!("{}.75rem", whole),
        _ => unreachable!(),
    }
}

/// Try to parse a string as a u32.
pub fn parse_u32(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let mut n: u32 = 0;
    for b in s.as_bytes() {
        if *b < b'0' || *b > b'9' {
            return None;
        }
        n = n.checked_mul(10)?.checked_add((*b - b'0') as u32)?;
    }
    Some(n)
}

/// Parse fractional spacing like "0.5" → 0.125rem, "1.5" → 0.375rem, etc.
/// Returns the spacing value string or None.
pub fn parse_fractional_spacing(s: &str) -> Option<String> {
    // Handle patterns like "0.5", "1.5", "2.5", "3.5"
    let dot_pos = s.find('.')?;
    let whole_part = &s[..dot_pos];
    let frac_part = &s[dot_pos + 1..];
    if frac_part != "5" {
        return None;
    }
    let whole = parse_u32(whole_part)?;
    // n.5 in Tailwind spacing scale = (n * 4 + 2) * 0.25rem = (n + 0.5) * 0.25rem
    // Actually: p-0.5 = 0.125rem, p-1.5 = 0.375rem, p-2.5 = 0.625rem, p-3.5 = 0.875rem
    // Formula: (whole * 2 + 1) * 0.125rem → simpler: whole * 0.25 + 0.125
    let val = whole * 2 + 1; // units of 0.125rem
    // val=1 → 0.125rem, val=3 → 0.375rem, val=5 → 0.625rem, val=7 → 0.875rem
    Some(crate::vformat!("0.{}rem", val * 125))
}

/// Parse a fraction like "1/2" → "50%", "1/3" → "33.333333%", etc.
pub fn parse_fraction(s: &str) -> Option<String> {
    let slash = s.find('/')?;
    let num = parse_u32(&s[..slash])?;
    let den = parse_u32(&s[slash + 1..])?;
    if den == 0 || num > den {
        return None;
    }
    if num == den {
        return Some(String::from("100%"));
    }
    // Common fractions
    match (num, den) {
        (1, 2) => Some(String::from("50%")),
        (1, 3) => Some(String::from("33.333333%")),
        (2, 3) => Some(String::from("66.666667%")),
        (1, 4) => Some(String::from("25%")),
        (2, 4) => Some(String::from("50%")),
        (3, 4) => Some(String::from("75%")),
        (1, 5) => Some(String::from("20%")),
        (2, 5) => Some(String::from("40%")),
        (3, 5) => Some(String::from("60%")),
        (4, 5) => Some(String::from("80%")),
        (1, 6) => Some(String::from("16.666667%")),
        (2, 6) => Some(String::from("33.333333%")),
        (3, 6) => Some(String::from("50%")),
        (4, 6) => Some(String::from("66.666667%")),
        (5, 6) => Some(String::from("83.333333%")),
        (1, 12) => Some(String::from("8.333333%")),
        (2, 12) => Some(String::from("16.666667%")),
        (3, 12) => Some(String::from("25%")),
        (4, 12) => Some(String::from("33.333333%")),
        (5, 12) => Some(String::from("41.666667%")),
        (6, 12) => Some(String::from("50%")),
        (7, 12) => Some(String::from("58.333333%")),
        (8, 12) => Some(String::from("66.666667%")),
        (9, 12) => Some(String::from("75%")),
        (10, 12) => Some(String::from("83.333333%")),
        (11, 12) => Some(String::from("91.666667%")),
        _ => None,
    }
}

/// Parse an arbitrary value like "[200px]" → "200px".
pub fn parse_arbitrary(s: &str) -> Option<&str> {
    if s.starts_with('[') && s.ends_with(']') && s.len() > 2 {
        Some(&s[1..s.len() - 1])
    } else {
        None
    }
}

/// Convert a hex color string to (r, g, b) tuple.
/// Supports "#rrggbb" format.
pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    if !hex.starts_with('#') || hex.len() != 7 {
        return None;
    }
    let r = u8_from_hex(&hex[1..3])?;
    let g = u8_from_hex(&hex[3..5])?;
    let b = u8_from_hex(&hex[5..7])?;
    Some((r, g, b))
}

fn u8_from_hex(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let hi = hex_digit(bytes[0])?;
    let lo = hex_digit(bytes[1])?;
    Some(hi * 16 + lo)
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Try to resolve a color name, possibly with opacity modifier (e.g. "red-500/50").
/// Returns (declarations_value, true) if opacity modifier present.
pub fn resolve_color_with_opacity(color_part: &str, property: &str) -> Option<String> {
    if let Some(slash_pos) = color_part.find('/') {
        let color_name = &color_part[..slash_pos];
        let opacity_str = &color_part[slash_pos + 1..];
        let opacity_val = parse_u32(opacity_str)?;
        if opacity_val > 100 {
            return None;
        }
        let hex = palette::color_hex(color_name)?;
        if hex == "transparent" {
            return Some(crate::vformat!("{}:transparent;", property));
        }
        let (r, g, b) = hex_to_rgb(hex)?;
        let alpha = if opacity_val == 100 {
            String::from("1")
        } else if opacity_val == 0 {
            String::from("0")
        } else if opacity_val % 10 == 0 {
            crate::vformat!("0.{}", opacity_val / 10)
        } else {
            crate::vformat!("0.{}", opacity_val)
        };
        Some(crate::vformat!("{}:rgb({} {} {} / {});", property, r, g, b, alpha))
    } else {
        let hex = palette::color_hex(color_part)?;
        Some(crate::vformat!("{}:{};", property, hex))
    }
}

/// Parse a spacing value that could be a number, fractional (0.5), or arbitrary ([200px]).
/// Returns the CSS value string.
pub fn parse_spacing_value(s: &str) -> Option<String> {
    // Try integer
    if let Some(n) = parse_u32(s) {
        return Some(spacing(n));
    }
    // Try fractional spacing (0.5, 1.5, etc.)
    if let Some(v) = parse_fractional_spacing(s) {
        return Some(v);
    }
    // Try arbitrary value
    if let Some(v) = parse_arbitrary(s) {
        return Some(String::from(v));
    }
    None
}

// ── Main dispatch ───────────────────────────────────────────────────────────

/// Resolve a utility class name to its declarations (no selector wrapping).
/// Returns a `ResolvedUtility` for special cases like space-x/y.
pub fn resolve_declarations(class: &str) -> Option<ResolvedUtility> {
    // Try each category in order
    if let Some(r) = layout::resolve(class) { return Some(r); }
    if let Some(r) = flexbox::resolve(class) { return Some(r); }
    if let Some(r) = grid::resolve(class) { return Some(r); }
    if let Some(r) = spacing::resolve(class) { return Some(r); }
    if let Some(r) = sizing::resolve(class) { return Some(r); }
    if let Some(r) = typography::resolve(class) { return Some(r); }
    if let Some(r) = backgrounds::resolve(class) { return Some(r); }
    if let Some(r) = borders::resolve(class) { return Some(r); }
    if let Some(r) = effects::resolve(class) { return Some(r); }
    if let Some(r) = transforms::resolve(class) { return Some(r); }
    if let Some(r) = filters::resolve(class) { return Some(r); }
    if let Some(r) = transitions::resolve(class) { return Some(r); }
    if let Some(r) = interactivity::resolve(class) { return Some(r); }
    if let Some(r) = tables::resolve(class) { return Some(r); }
    if let Some(r) = svg::resolve(class) { return Some(r); }
    if let Some(r) = inset::resolve(class) { return Some(r); }
    None
}

/// Resolve a utility class name to a complete CSS rule (backward compat).
pub fn resolve(class: &str) -> Option<String> {
    match resolve_declarations(class)? {
        ResolvedUtility::Standard(decls) => Some(rule(class, decls.as_str())),
        ResolvedUtility::Custom { selector_suffix, declarations } => {
            let escaped = escape_selector(class);
            let mut s = String::with_capacity(1 + escaped.len() + selector_suffix.len() + 1 + declarations.len() + 1);
            s.push_str(".");
            s.push_str(escaped.as_str());
            s.push_str(selector_suffix.as_str());
            s.push_str("{");
            s.push_str(declarations.as_str());
            s.push_str("}");
            Some(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_zero() {
        assert_eq!(spacing(0).as_str(), "0px");
    }

    #[test]
    fn test_spacing_values() {
        assert_eq!(spacing(1).as_str(), "0.25rem");
        assert_eq!(spacing(2).as_str(), "0.5rem");
        assert_eq!(spacing(4).as_str(), "1rem");
        assert_eq!(spacing(8).as_str(), "2rem");
    }

    #[test]
    fn test_parse_u32_valid() {
        assert_eq!(parse_u32("0"), Some(0));
        assert_eq!(parse_u32("42"), Some(42));
        assert_eq!(parse_u32("100"), Some(100));
    }

    #[test]
    fn test_parse_u32_invalid() {
        assert_eq!(parse_u32(""), None);
        assert_eq!(parse_u32("abc"), None);
        assert_eq!(parse_u32("12x"), None);
    }

    #[test]
    fn test_fractional_spacing() {
        assert_eq!(parse_fractional_spacing("0.5").unwrap().as_str(), "0.125rem");
        assert_eq!(parse_fractional_spacing("1.5").unwrap().as_str(), "0.375rem");
        assert_eq!(parse_fractional_spacing("2.5").unwrap().as_str(), "0.625rem");
        assert_eq!(parse_fractional_spacing("3.5").unwrap().as_str(), "0.875rem");
    }

    #[test]
    fn test_parse_fraction() {
        assert_eq!(parse_fraction("1/2").unwrap().as_str(), "50%");
        assert_eq!(parse_fraction("1/3").unwrap().as_str(), "33.333333%");
        assert_eq!(parse_fraction("2/3").unwrap().as_str(), "66.666667%");
        assert_eq!(parse_fraction("3/4").unwrap().as_str(), "75%");
    }

    #[test]
    fn test_parse_arbitrary() {
        assert_eq!(parse_arbitrary("[200px]"), Some("200px"));
        assert_eq!(parse_arbitrary("[#ff0000]"), Some("#ff0000"));
        assert_eq!(parse_arbitrary("200px"), None);
        assert_eq!(parse_arbitrary("[]"), None);
    }

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(hex_to_rgb("#ef4444"), Some((239, 68, 68)));
        assert_eq!(hex_to_rgb("#3b82f6"), Some((59, 130, 246)));
        assert_eq!(hex_to_rgb("#000000"), Some((0, 0, 0)));
        assert_eq!(hex_to_rgb("#ffffff"), Some((255, 255, 255)));
    }

    #[test]
    fn test_color_with_opacity() {
        let r = resolve_color_with_opacity("red-500/50", "background-color").unwrap();
        assert_eq!(r.as_str(), "background-color:rgb(239 68 68 / 0.5);");
    }

    #[test]
    fn test_color_without_opacity() {
        let r = resolve_color_with_opacity("blue-500", "color").unwrap();
        assert_eq!(r.as_str(), "color:#3b82f6;");
    }

    #[test]
    fn test_resolve_basic() {
        let r = resolve("flex").unwrap();
        assert_eq!(r.as_str(), ".flex{display:flex;}");
    }

    #[test]
    fn test_resolve_padding() {
        assert_eq!(resolve("p-4").unwrap().as_str(), ".p-4{padding:1rem;}");
    }

    #[test]
    fn test_resolve_unknown() {
        assert!(resolve("my-custom-class").is_none());
    }

    #[test]
    fn test_resolve_escaped_fraction() {
        let r = resolve("w-1/2").unwrap();
        assert_eq!(r.as_str(), ".w-1\\/2{width:50%;}");
    }

    #[test]
    fn test_resolve_arbitrary() {
        let r = resolve("w-[200px]").unwrap();
        assert_eq!(r.as_str(), ".w-\\[200px\\]{width:200px;}");
    }

    #[test]
    fn test_resolve_negative_margin() {
        let r = resolve("-mt-4").unwrap();
        assert_eq!(r.as_str(), ".-mt-4{margin-top:-1rem;}");
    }

    #[test]
    fn test_resolve_color_opacity() {
        let r = resolve("bg-blue-500/75").unwrap();
        assert_eq!(r.as_str(), ".bg-blue-500\\/75{background-color:rgb(59 130 246 / 0.75);}");
    }
}
