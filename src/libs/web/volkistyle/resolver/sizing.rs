//! Sizing utilities — width, height, min-w/h, max-w/h, size.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_fraction, parse_spacing_value};

fn resolve_dimension(class: &str, prefix: &str, property: &str) -> Option<ResolvedUtility> {
    let rest = class.strip_prefix(prefix)?;

    // Keywords
    let decl = match rest {
        "auto" => crate::vformat!("{}:auto;", property),
        "full" => crate::vformat!("{}:100%;", property),
        "screen" => {
            let unit = if property == "width" || property.contains("width") { "vw" } else { "vh" };
            crate::vformat!("{}:100{};", property, unit)
        }
        "svw" => crate::vformat!("{}:100svw;", property),
        "svh" => crate::vformat!("{}:100svh;", property),
        "lvw" => crate::vformat!("{}:100lvw;", property),
        "lvh" => crate::vformat!("{}:100lvh;", property),
        "dvw" => crate::vformat!("{}:100dvw;", property),
        "dvh" => crate::vformat!("{}:100dvh;", property),
        "min" => crate::vformat!("{}:min-content;", property),
        "max" => crate::vformat!("{}:max-content;", property),
        "fit" => crate::vformat!("{}:fit-content;", property),
        "px" => crate::vformat!("{}:1px;", property),
        _ => {
            // Try fraction
            if let Some(pct) = parse_fraction(rest) {
                return Some(ResolvedUtility::Standard(crate::vformat!("{}:{};", property, pct)));
            }
            // Try numeric spacing
            if let Some(val) = parse_spacing_value(rest) {
                return Some(ResolvedUtility::Standard(crate::vformat!("{}:{};", property, val)));
            }
            return None;
        }
    };
    Some(ResolvedUtility::Standard(decl))
}

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    // flex-basis
    if let Some(rest) = class.strip_prefix("basis-") {
        let decl = match rest {
            "auto" => String::from("flex-basis:auto;"),
            "full" => String::from("flex-basis:100%;"),
            "px" => String::from("flex-basis:1px;"),
            _ => {
                if let Some(pct) = parse_fraction(rest) {
                    crate::vformat!("flex-basis:{};", pct)
                } else if let Some(v) = parse_spacing_value(rest) {
                    crate::vformat!("flex-basis:{};", v)
                } else {
                    return None;
                }
            }
        };
        return Some(ResolvedUtility::Standard(decl));
    }

    // size-{n} (sets both width + height)
    if let Some(rest) = class.strip_prefix("size-") {
        let val = match rest {
            "auto" => String::from("auto"),
            "full" => String::from("100%"),
            "min" => String::from("min-content"),
            "max" => String::from("max-content"),
            "fit" => String::from("fit-content"),
            "px" => String::from("1px"),
            _ => {
                if let Some(pct) = parse_fraction(rest) {
                    pct
                } else if let Some(v) = parse_spacing_value(rest) {
                    v
                } else {
                    return None;
                }
            }
        };
        return Some(ResolvedUtility::Standard(crate::vformat!("width:{};height:{};", val, val)));
    }

    // max-w
    if let Some(rest) = class.strip_prefix("max-w-") {
        let decl = match rest {
            "none" => "max-width:none;",
            "0" => "max-width:0rem;",
            "xs" => "max-width:20rem;",
            "sm" => "max-width:24rem;",
            "md" => "max-width:28rem;",
            "lg" => "max-width:32rem;",
            "xl" => "max-width:36rem;",
            "2xl" => "max-width:42rem;",
            "3xl" => "max-width:48rem;",
            "4xl" => "max-width:56rem;",
            "5xl" => "max-width:64rem;",
            "6xl" => "max-width:72rem;",
            "7xl" => "max-width:80rem;",
            "full" => "max-width:100%;",
            "min" => "max-width:min-content;",
            "max" => "max-width:max-content;",
            "fit" => "max-width:fit-content;",
            "prose" => "max-width:65ch;",
            "screen-sm" => "max-width:640px;",
            "screen-md" => "max-width:768px;",
            "screen-lg" => "max-width:1024px;",
            "screen-xl" => "max-width:1280px;",
            "screen-2xl" => "max-width:1536px;",
            "screen" => "max-width:100vw;",
            _ => {
                if let Some(val) = parse_spacing_value(rest) {
                    return Some(ResolvedUtility::Standard(crate::vformat!("max-width:{};", val)));
                }
                return None;
            }
        };
        return Some(ResolvedUtility::Standard(String::from(decl)));
    }

    // max-h
    if let Some(rest) = class.strip_prefix("max-h-") {
        let decl = match rest {
            "none" => "max-height:none;",
            "full" => "max-height:100%;",
            "screen" => "max-height:100vh;",
            "min" => "max-height:min-content;",
            "max" => "max-height:max-content;",
            "fit" => "max-height:fit-content;",
            _ => {
                if let Some(val) = parse_spacing_value(rest) {
                    return Some(ResolvedUtility::Standard(crate::vformat!("max-height:{};", val)));
                }
                return None;
            }
        };
        return Some(ResolvedUtility::Standard(String::from(decl)));
    }

    // min-w
    if let Some(rest) = class.strip_prefix("min-w-") {
        let decl = match rest {
            "0" => "min-width:0px;",
            "full" => "min-width:100%;",
            "min" => "min-width:min-content;",
            "max" => "min-width:max-content;",
            "fit" => "min-width:fit-content;",
            _ => {
                if let Some(val) = parse_spacing_value(rest) {
                    return Some(ResolvedUtility::Standard(crate::vformat!("min-width:{};", val)));
                }
                return None;
            }
        };
        return Some(ResolvedUtility::Standard(String::from(decl)));
    }

    // min-h
    if let Some(rest) = class.strip_prefix("min-h-") {
        let decl = match rest {
            "0" => "min-height:0px;",
            "full" => "min-height:100%;",
            "screen" => "min-height:100vh;",
            "svh" => "min-height:100svh;",
            "lvh" => "min-height:100lvh;",
            "dvh" => "min-height:100dvh;",
            "min" => "min-height:min-content;",
            "max" => "min-height:max-content;",
            "fit" => "min-height:fit-content;",
            _ => {
                if let Some(val) = parse_spacing_value(rest) {
                    return Some(ResolvedUtility::Standard(crate::vformat!("min-height:{};", val)));
                }
                return None;
            }
        };
        return Some(ResolvedUtility::Standard(String::from(decl)));
    }

    // w-{n}
    if class.starts_with("w-") {
        return resolve_dimension(class, "w-", "width");
    }

    // h-{n}
    if class.starts_with("h-") {
        return resolve_dimension(class, "h-", "height");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::resolve;

    #[test]
    fn test_width_numeric() {
        assert_eq!(resolve("w-4").unwrap().as_str(), ".w-4{width:1rem;}");
        assert_eq!(resolve("w-0").unwrap().as_str(), ".w-0{width:0px;}");
    }

    #[test]
    fn test_width_keywords() {
        assert_eq!(resolve("w-full").unwrap().as_str(), ".w-full{width:100%;}");
        assert_eq!(resolve("w-auto").unwrap().as_str(), ".w-auto{width:auto;}");
        assert_eq!(resolve("w-screen").unwrap().as_str(), ".w-screen{width:100vw;}");
        assert_eq!(resolve("w-min").unwrap().as_str(), ".w-min{width:min-content;}");
        assert_eq!(resolve("w-max").unwrap().as_str(), ".w-max{width:max-content;}");
        assert_eq!(resolve("w-fit").unwrap().as_str(), ".w-fit{width:fit-content;}");
    }

    #[test]
    fn test_width_fraction() {
        assert_eq!(resolve("w-1/2").unwrap().as_str(), ".w-1\\/2{width:50%;}");
        assert_eq!(resolve("w-1/3").unwrap().as_str(), ".w-1\\/3{width:33.333333%;}");
        assert_eq!(resolve("w-2/3").unwrap().as_str(), ".w-2\\/3{width:66.666667%;}");
    }

    #[test]
    fn test_height() {
        assert_eq!(resolve("h-8").unwrap().as_str(), ".h-8{height:2rem;}");
        assert_eq!(resolve("h-full").unwrap().as_str(), ".h-full{height:100%;}");
        assert_eq!(resolve("h-screen").unwrap().as_str(), ".h-screen{height:100vh;}");
    }

    #[test]
    fn test_size() {
        assert_eq!(resolve("size-4").unwrap().as_str(), ".size-4{width:1rem;height:1rem;}");
        assert_eq!(resolve("size-full").unwrap().as_str(), ".size-full{width:100%;height:100%;}");
    }

    #[test]
    fn test_max_width() {
        assert_eq!(resolve("max-w-lg").unwrap().as_str(), ".max-w-lg{max-width:32rem;}");
        assert_eq!(resolve("max-w-full").unwrap().as_str(), ".max-w-full{max-width:100%;}");
        assert_eq!(resolve("max-w-prose").unwrap().as_str(), ".max-w-prose{max-width:65ch;}");
        assert_eq!(resolve("max-w-screen-md").unwrap().as_str(), ".max-w-screen-md{max-width:768px;}");
    }

    #[test]
    fn test_max_height() {
        assert_eq!(resolve("max-h-full").unwrap().as_str(), ".max-h-full{max-height:100%;}");
        assert_eq!(resolve("max-h-screen").unwrap().as_str(), ".max-h-screen{max-height:100vh;}");
    }

    #[test]
    fn test_min_width() {
        assert_eq!(resolve("min-w-0").unwrap().as_str(), ".min-w-0{min-width:0px;}");
        assert_eq!(resolve("min-w-full").unwrap().as_str(), ".min-w-full{min-width:100%;}");
        assert_eq!(resolve("min-w-min").unwrap().as_str(), ".min-w-min{min-width:min-content;}");
    }

    #[test]
    fn test_min_height() {
        assert_eq!(resolve("min-h-0").unwrap().as_str(), ".min-h-0{min-height:0px;}");
        assert_eq!(resolve("min-h-full").unwrap().as_str(), ".min-h-full{min-height:100%;}");
        assert_eq!(resolve("min-h-screen").unwrap().as_str(), ".min-h-screen{min-height:100vh;}");
    }

    #[test]
    fn test_arbitrary_width() {
        assert_eq!(resolve("w-[200px]").unwrap().as_str(), ".w-\\[200px\\]{width:200px;}");
    }
}
