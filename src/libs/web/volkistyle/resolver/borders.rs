//! Border utilities — width, color, style, radius, divide, outline, ring.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32, resolve_color_with_opacity};
use crate::libs::web::volkistyle::palette;

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Border width (shorthand)
        "border" => "border-width:1px;",
        "border-0" => "border-width:0px;",
        "border-2" => "border-width:2px;",
        "border-4" => "border-width:4px;",
        "border-8" => "border-width:8px;",

        // Per-side border width (1px default)
        "border-t" => "border-top-width:1px;",
        "border-r" => "border-right-width:1px;",
        "border-b" => "border-bottom-width:1px;",
        "border-l" => "border-left-width:1px;",
        "border-x" => "border-left-width:1px;border-right-width:1px;",
        "border-y" => "border-top-width:1px;border-bottom-width:1px;",

        // Border style
        "border-solid" => "border-style:solid;",
        "border-dashed" => "border-style:dashed;",
        "border-dotted" => "border-style:dotted;",
        "border-double" => "border-style:double;",
        "border-hidden" => "border-style:hidden;",
        "border-none" => "border-style:none;",

        // Border radius (shorthand)
        "rounded" => "border-radius:0.25rem;",
        "rounded-none" => "border-radius:0px;",
        "rounded-sm" => "border-radius:0.125rem;",
        "rounded-md" => "border-radius:0.375rem;",
        "rounded-lg" => "border-radius:0.5rem;",
        "rounded-xl" => "border-radius:0.75rem;",
        "rounded-2xl" => "border-radius:1rem;",
        "rounded-3xl" => "border-radius:1.5rem;",
        "rounded-full" => "border-radius:9999px;",

        // Per-side radius
        "rounded-t" => "border-top-left-radius:0.25rem;border-top-right-radius:0.25rem;",
        "rounded-r" => "border-top-right-radius:0.25rem;border-bottom-right-radius:0.25rem;",
        "rounded-b" => "border-bottom-right-radius:0.25rem;border-bottom-left-radius:0.25rem;",
        "rounded-l" => "border-top-left-radius:0.25rem;border-bottom-left-radius:0.25rem;",

        // Outline
        "outline-none" => "outline:2px solid transparent;outline-offset:2px;",
        "outline" => "outline-style:solid;",
        "outline-dashed" => "outline-style:dashed;",
        "outline-dotted" => "outline-style:dotted;",
        "outline-double" => "outline-style:double;",

        // Ring
        "ring" => "box-shadow:0 0 0 3px rgba(59,130,246,0.5);",
        "ring-0" => "box-shadow:0 0 0 0px rgba(59,130,246,0.5);",
        "ring-1" => "box-shadow:0 0 0 1px rgba(59,130,246,0.5);",
        "ring-2" => "box-shadow:0 0 0 2px rgba(59,130,246,0.5);",
        "ring-4" => "box-shadow:0 0 0 4px rgba(59,130,246,0.5);",
        "ring-8" => "box-shadow:0 0 0 8px rgba(59,130,246,0.5);",
        "ring-inset" => "--tw-ring-inset:inset;",

        // Divide (child combinator)
        "divide-x" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-left-width:1px;"),
            });
        }
        "divide-y" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-top-width:1px;"),
            });
        }
        "divide-x-0" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-left-width:0px;"),
            });
        }
        "divide-y-0" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-top-width:0px;"),
            });
        }
        "divide-x-2" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-left-width:2px;"),
            });
        }
        "divide-y-2" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-top-width:2px;"),
            });
        }
        "divide-x-4" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-left-width:4px;"),
            });
        }
        "divide-y-4" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-top-width:4px;"),
            });
        }
        "divide-x-8" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-left-width:8px;"),
            });
        }
        "divide-y-8" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-top-width:8px;"),
            });
        }
        "divide-solid" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-style:solid;"),
            });
        }
        "divide-dashed" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-style:dashed;"),
            });
        }
        "divide-dotted" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-style:dotted;"),
            });
        }
        "divide-double" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-style:double;"),
            });
        }
        "divide-none" => {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: String::from("border-style:none;"),
            });
        }

        _ => {
            return resolve_prefix(class);
        }
    };
    Some(ResolvedUtility::Standard(String::from(decls)))
}

fn resolve_prefix(class: &str) -> Option<ResolvedUtility> {
    // Per-side border width with number: border-t-{n}
    if let Some(rest) = class.strip_prefix("border-t-") {
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("border-top-width:{}px;", n)));
        }
        if let Some(decls) = resolve_color_with_opacity(rest, "border-top-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        return None;
    }
    if let Some(rest) = class.strip_prefix("border-r-") {
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("border-right-width:{}px;", n)));
        }
        if let Some(decls) = resolve_color_with_opacity(rest, "border-right-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        return None;
    }
    if let Some(rest) = class.strip_prefix("border-b-") {
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("border-bottom-width:{}px;", n)));
        }
        if let Some(decls) = resolve_color_with_opacity(rest, "border-bottom-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        return None;
    }
    if let Some(rest) = class.strip_prefix("border-l-") {
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("border-left-width:{}px;", n)));
        }
        if let Some(decls) = resolve_color_with_opacity(rest, "border-left-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        return None;
    }
    if let Some(rest) = class.strip_prefix("border-x-") {
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("border-left-width:{}px;border-right-width:{}px;", n, n)));
        }
        return None;
    }
    if let Some(rest) = class.strip_prefix("border-y-") {
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("border-top-width:{}px;border-bottom-width:{}px;", n, n)));
        }
        return None;
    }

    // Border color
    if let Some(rest) = class.strip_prefix("border-") {
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("border-width:{}px;", n)));
        }
        if let Some(decls) = resolve_color_with_opacity(rest, "border-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        // Arbitrary value: border-[#30363d], border-[rgb(...)], etc.
        if let Some(val) = super::parse_arbitrary(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("border-color:{};", val)));
        }
        return None;
    }

    // Per-side / per-corner radius with size
    if let Some(rest) = class.strip_prefix("rounded-t-") {
        let val = radius_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!(
            "border-top-left-radius:{};border-top-right-radius:{};", val, val
        )));
    }
    if let Some(rest) = class.strip_prefix("rounded-r-") {
        let val = radius_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!(
            "border-top-right-radius:{};border-bottom-right-radius:{};", val, val
        )));
    }
    if let Some(rest) = class.strip_prefix("rounded-b-") {
        let val = radius_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!(
            "border-bottom-right-radius:{};border-bottom-left-radius:{};", val, val
        )));
    }
    if let Some(rest) = class.strip_prefix("rounded-l-") {
        let val = radius_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!(
            "border-top-left-radius:{};border-bottom-left-radius:{};", val, val
        )));
    }
    if let Some(rest) = class.strip_prefix("rounded-tl-") {
        let val = radius_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("border-top-left-radius:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("rounded-tr-") {
        let val = radius_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("border-top-right-radius:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("rounded-bl-") {
        let val = radius_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("border-bottom-left-radius:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("rounded-br-") {
        let val = radius_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("border-bottom-right-radius:{};", val)));
    }

    // Outline width / color / offset
    if let Some(rest) = class.strip_prefix("outline-") {
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("outline-width:{}px;", n)));
        }
        if let Some(decls) = resolve_color_with_opacity(rest, "outline-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        return None;
    }
    if let Some(rest) = class.strip_prefix("outline-offset-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("outline-offset:{}px;", n)));
    }

    // Ring color
    if let Some(rest) = class.strip_prefix("ring-offset-") {
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("--tw-ring-offset-width:{}px;box-shadow:0 0 0 var(--tw-ring-offset-width) var(--tw-ring-offset-color),var(--tw-ring-shadow);", n)));
        }
        if let Some(hex) = palette::color_hex(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("--tw-ring-offset-color:{};", hex)));
        }
        return None;
    }
    if let Some(rest) = class.strip_prefix("ring-") {
        if let Some(hex) = palette::color_hex(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("--tw-ring-color:{};", hex)));
        }
        return None;
    }

    // Divide color
    if let Some(rest) = class.strip_prefix("divide-") {
        if let Some(hex) = palette::color_hex(rest) {
            return Some(ResolvedUtility::Custom {
                selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
                declarations: crate::vformat!("border-color:{};", hex),
            });
        }
        return None;
    }

    None
}

fn radius_value(size: &str) -> Option<&'static str> {
    match size {
        "none" => Some("0px"),
        "sm" => Some("0.125rem"),
        "md" => Some("0.375rem"),
        "lg" => Some("0.5rem"),
        "xl" => Some("0.75rem"),
        "2xl" => Some("1rem"),
        "3xl" => Some("1.5rem"),
        "full" => Some("9999px"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::resolve;

    #[test]
    fn test_border_width() {
        assert_eq!(resolve("border").unwrap().as_str(), ".border{border-width:1px;}");
        assert_eq!(resolve("border-2").unwrap().as_str(), ".border-2{border-width:2px;}");
        assert_eq!(resolve("border-0").unwrap().as_str(), ".border-0{border-width:0px;}");
    }

    #[test]
    fn test_border_per_side() {
        assert_eq!(resolve("border-t").unwrap().as_str(), ".border-t{border-top-width:1px;}");
        assert_eq!(resolve("border-t-2").unwrap().as_str(), ".border-t-2{border-top-width:2px;}");
    }

    #[test]
    fn test_border_color() {
        assert_eq!(resolve("border-red-500").unwrap().as_str(), ".border-red-500{border-color:#ef4444;}");
    }

    #[test]
    fn test_border_style() {
        assert_eq!(resolve("border-dashed").unwrap().as_str(), ".border-dashed{border-style:dashed;}");
        assert_eq!(resolve("border-none").unwrap().as_str(), ".border-none{border-style:none;}");
    }

    #[test]
    fn test_rounded() {
        assert_eq!(resolve("rounded").unwrap().as_str(), ".rounded{border-radius:0.25rem;}");
        assert_eq!(resolve("rounded-lg").unwrap().as_str(), ".rounded-lg{border-radius:0.5rem;}");
        assert_eq!(resolve("rounded-full").unwrap().as_str(), ".rounded-full{border-radius:9999px;}");
    }

    #[test]
    fn test_rounded_per_side() {
        let r = resolve("rounded-t-lg").unwrap();
        assert!(r.as_str().contains("border-top-left-radius:0.5rem;"));
        assert!(r.as_str().contains("border-top-right-radius:0.5rem;"));
    }

    #[test]
    fn test_rounded_per_corner() {
        assert_eq!(resolve("rounded-tl-lg").unwrap().as_str(), ".rounded-tl-lg{border-top-left-radius:0.5rem;}");
    }

    #[test]
    fn test_divide() {
        let r = resolve("divide-x").unwrap();
        assert!(r.as_str().contains(">:not([hidden])~:not([hidden])"));
        assert!(r.as_str().contains("border-left-width:1px;"));
    }

    #[test]
    fn test_outline() {
        assert!(resolve("outline-none").unwrap().as_str().contains("outline:2px solid transparent;"));
        assert_eq!(resolve("outline-2").unwrap().as_str(), ".outline-2{outline-width:2px;}");
    }

    #[test]
    fn test_ring() {
        assert!(resolve("ring").unwrap().as_str().contains("box-shadow:0 0 0 3px"));
        assert!(resolve("ring-2").unwrap().as_str().contains("box-shadow:0 0 0 2px"));
    }

    #[test]
    fn test_border_arbitrary_hex() {
        assert_eq!(
            resolve("border-[#30363d]").unwrap().as_str(),
            ".border-\\[\\#30363d\\]{border-color:#30363d;}"
        );
    }
}
