//! Transform utilities — scale, rotate, translate, skew, transform-origin.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32, parse_fraction, parse_spacing_value};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Transform origin
        "origin-center" => "transform-origin:center;",
        "origin-top" => "transform-origin:top;",
        "origin-top-right" => "transform-origin:top right;",
        "origin-right" => "transform-origin:right;",
        "origin-bottom-right" => "transform-origin:bottom right;",
        "origin-bottom" => "transform-origin:bottom;",
        "origin-bottom-left" => "transform-origin:bottom left;",
        "origin-left" => "transform-origin:left;",
        "origin-top-left" => "transform-origin:top left;",

        _ => {
            return resolve_prefix(class);
        }
    };
    Some(ResolvedUtility::Standard(String::from(decls)))
}

fn resolve_prefix(class: &str) -> Option<ResolvedUtility> {
    // Scale
    if let Some(rest) = class.strip_prefix("scale-x-") {
        let n = parse_u32(rest)?;
        let val = scale_value(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:scaleX({});", val)));
    }
    if let Some(rest) = class.strip_prefix("scale-y-") {
        let n = parse_u32(rest)?;
        let val = scale_value(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:scaleY({});", val)));
    }
    if let Some(rest) = class.strip_prefix("scale-") {
        let n = parse_u32(rest)?;
        let val = scale_value(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:scale({});", val)));
    }

    // Negative rotate
    if let Some(rest) = class.strip_prefix("-rotate-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:rotate(-{}deg);", n)));
    }
    // Rotate
    if let Some(rest) = class.strip_prefix("rotate-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:rotate({}deg);", n)));
    }

    // Negative translate
    if let Some(rest) = class.strip_prefix("-translate-x-") {
        if let Some(pct) = parse_fraction(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("transform:translateX(-{});", pct)));
        }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:translateX(-{});", val)));
    }
    if let Some(rest) = class.strip_prefix("-translate-y-") {
        if let Some(pct) = parse_fraction(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("transform:translateY(-{});", pct)));
        }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:translateY(-{});", val)));
    }
    // Translate
    if let Some(rest) = class.strip_prefix("translate-x-") {
        if rest == "full" {
            return Some(ResolvedUtility::Standard(String::from("transform:translateX(100%);")));
        }
        if let Some(pct) = parse_fraction(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("transform:translateX({});", pct)));
        }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:translateX({});", val)));
    }
    if let Some(rest) = class.strip_prefix("translate-y-") {
        if rest == "full" {
            return Some(ResolvedUtility::Standard(String::from("transform:translateY(100%);")));
        }
        if let Some(pct) = parse_fraction(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("transform:translateY({});", pct)));
        }
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:translateY({});", val)));
    }

    // Negative skew
    if let Some(rest) = class.strip_prefix("-skew-x-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:skewX(-{}deg);", n)));
    }
    if let Some(rest) = class.strip_prefix("-skew-y-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:skewY(-{}deg);", n)));
    }
    // Skew
    if let Some(rest) = class.strip_prefix("skew-x-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:skewX({}deg);", n)));
    }
    if let Some(rest) = class.strip_prefix("skew-y-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("transform:skewY({}deg);", n)));
    }

    None
}

fn scale_value(n: u32) -> String {
    if n == 0 {
        String::from("0")
    } else if n == 100 {
        String::from("1")
    } else if n % 100 == 0 {
        crate::vformat!("{}", n / 100)
    } else if n % 10 == 0 {
        crate::vformat!("{}.{}", n / 100, (n % 100) / 10)
    } else {
        crate::vformat!("{}.{}", n / 100, n % 100)
    }
}

#[cfg(test)]
mod tests {
    use super::super::resolve;

    #[test]
    fn test_scale() {
        assert_eq!(resolve("scale-100").unwrap().as_str(), ".scale-100{transform:scale(1);}");
        assert_eq!(resolve("scale-50").unwrap().as_str(), ".scale-50{transform:scale(0.5);}");
        assert_eq!(resolve("scale-150").unwrap().as_str(), ".scale-150{transform:scale(1.5);}");
        assert_eq!(resolve("scale-0").unwrap().as_str(), ".scale-0{transform:scale(0);}");
    }

    #[test]
    fn test_scale_axis() {
        assert_eq!(resolve("scale-x-75").unwrap().as_str(), ".scale-x-75{transform:scaleX(0.75);}");
        assert_eq!(resolve("scale-y-110").unwrap().as_str(), ".scale-y-110{transform:scaleY(1.1);}");
    }

    #[test]
    fn test_rotate() {
        assert_eq!(resolve("rotate-45").unwrap().as_str(), ".rotate-45{transform:rotate(45deg);}");
        assert_eq!(resolve("rotate-180").unwrap().as_str(), ".rotate-180{transform:rotate(180deg);}");
        assert_eq!(resolve("-rotate-45").unwrap().as_str(), ".-rotate-45{transform:rotate(-45deg);}");
    }

    #[test]
    fn test_translate() {
        assert_eq!(resolve("translate-x-4").unwrap().as_str(), ".translate-x-4{transform:translateX(1rem);}");
        assert_eq!(resolve("-translate-x-4").unwrap().as_str(), ".-translate-x-4{transform:translateX(-1rem);}");
        assert_eq!(resolve("translate-x-full").unwrap().as_str(), ".translate-x-full{transform:translateX(100%);}");
    }

    #[test]
    fn test_translate_fraction() {
        assert_eq!(resolve("translate-x-1/2").unwrap().as_str(), ".translate-x-1\\/2{transform:translateX(50%);}");
    }

    #[test]
    fn test_skew() {
        assert_eq!(resolve("skew-x-6").unwrap().as_str(), ".skew-x-6{transform:skewX(6deg);}");
        assert_eq!(resolve("-skew-y-3").unwrap().as_str(), ".-skew-y-3{transform:skewY(-3deg);}");
    }

    #[test]
    fn test_origin() {
        assert_eq!(resolve("origin-center").unwrap().as_str(), ".origin-center{transform-origin:center;}");
        assert_eq!(resolve("origin-top-left").unwrap().as_str(), ".origin-top-left{transform-origin:top left;}");
    }
}
