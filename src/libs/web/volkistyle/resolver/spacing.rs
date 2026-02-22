//! Spacing utilities — padding, margin, gap, space-x/y.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_spacing_value};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    // Space between (uses child combinator)
    if class == "space-x-reverse" {
        return Some(ResolvedUtility::Custom {
            selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
            declarations: String::from("--tw-space-x-reverse:1;"),
        });
    }
    if class == "space-y-reverse" {
        return Some(ResolvedUtility::Custom {
            selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
            declarations: String::from("--tw-space-y-reverse:1;"),
        });
    }
    if let Some(rest) = class.strip_prefix("space-x-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Custom {
            selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
            declarations: crate::vformat!("margin-left:{};", val),
        });
    }
    if let Some(rest) = class.strip_prefix("space-y-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Custom {
            selector_suffix: String::from(">:not([hidden])~:not([hidden])"),
            declarations: crate::vformat!("margin-top:{};", val),
        });
    }

    // Negative margins
    if let Some(rest) = class.strip_prefix("-mx-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-left:-{};margin-right:-{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("-my-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-top:-{};margin-bottom:-{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("-mt-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-top:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-mr-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-right:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-mb-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-bottom:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-ml-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-left:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-ms-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-inline-start:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-me-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-inline-end:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-m-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin:-{};", val)));
    }

    // Padding — axis
    if let Some(rest) = class.strip_prefix("px-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("padding-left:{};padding-right:{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("py-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("padding-top:{};padding-bottom:{};", val, val)));
    }
    // Padding — sides
    if let Some(rest) = class.strip_prefix("pt-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("padding-top:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("pr-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("padding-right:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("pb-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("padding-bottom:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("pl-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("padding-left:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("ps-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("padding-inline-start:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("pe-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("padding-inline-end:{};", val)));
    }
    // Padding — all
    if let Some(rest) = class.strip_prefix("p-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("padding:{};", val)));
    }

    // Margin — axis
    if let Some(rest) = class.strip_prefix("mx-") {
        if rest == "auto" {
            return Some(ResolvedUtility::Standard(String::from("margin-left:auto;margin-right:auto;")));
        }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-left:{};margin-right:{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("my-") {
        if rest == "auto" {
            return Some(ResolvedUtility::Standard(String::from("margin-top:auto;margin-bottom:auto;")));
        }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-top:{};margin-bottom:{};", val, val)));
    }
    // Margin — sides
    if let Some(rest) = class.strip_prefix("mt-") {
        if rest == "auto" { return Some(ResolvedUtility::Standard(String::from("margin-top:auto;"))); }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-top:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("mr-") {
        if rest == "auto" { return Some(ResolvedUtility::Standard(String::from("margin-right:auto;"))); }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-right:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("mb-") {
        if rest == "auto" { return Some(ResolvedUtility::Standard(String::from("margin-bottom:auto;"))); }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-bottom:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("ml-") {
        if rest == "auto" { return Some(ResolvedUtility::Standard(String::from("margin-left:auto;"))); }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-left:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("ms-") {
        if rest == "auto" { return Some(ResolvedUtility::Standard(String::from("margin-inline-start:auto;"))); }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-inline-start:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("me-") {
        if rest == "auto" { return Some(ResolvedUtility::Standard(String::from("margin-inline-end:auto;"))); }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin-inline-end:{};", val)));
    }
    // Margin — all
    if let Some(rest) = class.strip_prefix("m-") {
        if rest == "auto" {
            return Some(ResolvedUtility::Standard(String::from("margin:auto;")));
        }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("margin:{};", val)));
    }

    // Gap
    if let Some(rest) = class.strip_prefix("gap-x-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("column-gap:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("gap-y-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("row-gap:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("gap-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("gap:{};", val)));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::resolve;

    #[test]
    fn test_padding() {
        assert_eq!(resolve("p-0").unwrap().as_str(), ".p-0{padding:0px;}");
        assert_eq!(resolve("p-1").unwrap().as_str(), ".p-1{padding:0.25rem;}");
        assert_eq!(resolve("p-4").unwrap().as_str(), ".p-4{padding:1rem;}");
        assert_eq!(resolve("p-8").unwrap().as_str(), ".p-8{padding:2rem;}");
    }

    #[test]
    fn test_padding_axis() {
        let r = resolve("px-4").unwrap();
        assert!(r.as_str().contains("padding-left:1rem;"));
        assert!(r.as_str().contains("padding-right:1rem;"));

        let r = resolve("py-2").unwrap();
        assert!(r.as_str().contains("padding-top:0.5rem;"));
        assert!(r.as_str().contains("padding-bottom:0.5rem;"));
    }

    #[test]
    fn test_padding_sides() {
        assert_eq!(resolve("pt-4").unwrap().as_str(), ".pt-4{padding-top:1rem;}");
        assert_eq!(resolve("pr-4").unwrap().as_str(), ".pr-4{padding-right:1rem;}");
        assert_eq!(resolve("pb-4").unwrap().as_str(), ".pb-4{padding-bottom:1rem;}");
        assert_eq!(resolve("pl-4").unwrap().as_str(), ".pl-4{padding-left:1rem;}");
    }

    #[test]
    fn test_margin() {
        assert_eq!(resolve("m-4").unwrap().as_str(), ".m-4{margin:1rem;}");
        assert_eq!(resolve("m-auto").unwrap().as_str(), ".m-auto{margin:auto;}");
        assert_eq!(resolve("mx-auto").unwrap().as_str(), ".mx-auto{margin-left:auto;margin-right:auto;}");
    }

    #[test]
    fn test_negative_margin() {
        assert_eq!(resolve("-mt-4").unwrap().as_str(), ".-mt-4{margin-top:-1rem;}");
        assert_eq!(resolve("-m-2").unwrap().as_str(), ".-m-2{margin:-0.5rem;}");
    }

    #[test]
    fn test_fractional_padding() {
        assert_eq!(resolve("p-0.5").unwrap().as_str(), ".p-0\\.5{padding:0.125rem;}");
    }

    #[test]
    fn test_gap() {
        assert_eq!(resolve("gap-4").unwrap().as_str(), ".gap-4{gap:1rem;}");
        assert_eq!(resolve("gap-0").unwrap().as_str(), ".gap-0{gap:0px;}");
        assert_eq!(resolve("gap-x-2").unwrap().as_str(), ".gap-x-2{column-gap:0.5rem;}");
        assert_eq!(resolve("gap-y-4").unwrap().as_str(), ".gap-y-4{row-gap:1rem;}");
    }

    #[test]
    fn test_space_between() {
        let r = resolve("space-x-4").unwrap();
        assert!(r.as_str().contains("margin-left:1rem;"));
        assert!(r.as_str().contains(">:not([hidden])~:not([hidden])"));
    }

    #[test]
    fn test_arbitrary_padding() {
        assert_eq!(resolve("p-[20px]").unwrap().as_str(), ".p-\\[20px\\]{padding:20px;}");
    }
}
