//! Inset utilities — top, right, bottom, left, inset (+ axis, fractions, negative, auto).

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32, parse_fraction, parse_spacing_value};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    // Negative inset
    if let Some(rest) = class.strip_prefix("-inset-x-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("left:-{};right:-{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("-inset-y-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("top:-{};bottom:-{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("-inset-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("inset:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-top-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("top:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-right-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("right:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-bottom-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("bottom:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-left-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("left:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-start-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("inset-inline-start:-{};", val)));
    }
    if let Some(rest) = class.strip_prefix("-end-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("inset-inline-end:-{};", val)));
    }

    // Inset axis
    if let Some(rest) = class.strip_prefix("inset-x-") {
        let val = resolve_inset_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("left:{};right:{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("inset-y-") {
        let val = resolve_inset_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("top:{};bottom:{};", val, val)));
    }

    // Inset all
    if let Some(rest) = class.strip_prefix("inset-") {
        let val = resolve_inset_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("inset:{};", val)));
    }

    // Individual sides
    if let Some(rest) = class.strip_prefix("top-") {
        let val = resolve_inset_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("top:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("right-") {
        let val = resolve_inset_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("right:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("bottom-") {
        let val = resolve_inset_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("bottom:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("left-") {
        let val = resolve_inset_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("left:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("start-") {
        let val = resolve_inset_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("inset-inline-start:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("end-") {
        let val = resolve_inset_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("inset-inline-end:{};", val)));
    }

    // Z-index
    if let Some(rest) = class.strip_prefix("z-") {
        if rest == "auto" {
            return Some(ResolvedUtility::Standard(String::from("z-index:auto;")));
        }
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("z-index:{};", n)));
    }

    None
}

fn resolve_inset_value(s: &str) -> Option<String> {
    if s == "auto" {
        return Some(String::from("auto"));
    }
    if s == "full" {
        return Some(String::from("100%"));
    }
    if s == "px" {
        return Some(String::from("1px"));
    }
    // Fraction
    if let Some(pct) = parse_fraction(s) {
        return Some(pct);
    }
    // Spacing value
    if let Some(val) = parse_spacing_value(s) {
        return Some(val);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::super::resolve;

    #[test]
    fn test_inset() {
        assert_eq!(resolve("inset-0").unwrap().as_str(), ".inset-0{inset:0px;}");
        assert_eq!(resolve("inset-4").unwrap().as_str(), ".inset-4{inset:1rem;}");
        assert_eq!(resolve("inset-auto").unwrap().as_str(), ".inset-auto{inset:auto;}");
    }

    #[test]
    fn test_inset_axis() {
        let r = resolve("inset-x-0").unwrap();
        assert!(r.as_str().contains("left:0px;"));
        assert!(r.as_str().contains("right:0px;"));

        let r = resolve("inset-y-4").unwrap();
        assert!(r.as_str().contains("top:1rem;"));
        assert!(r.as_str().contains("bottom:1rem;"));
    }

    #[test]
    fn test_individual_sides() {
        assert_eq!(resolve("top-0").unwrap().as_str(), ".top-0{top:0px;}");
        assert_eq!(resolve("right-4").unwrap().as_str(), ".right-4{right:1rem;}");
        assert_eq!(resolve("bottom-0").unwrap().as_str(), ".bottom-0{bottom:0px;}");
        assert_eq!(resolve("left-0").unwrap().as_str(), ".left-0{left:0px;}");
    }

    #[test]
    fn test_inset_auto() {
        assert_eq!(resolve("top-auto").unwrap().as_str(), ".top-auto{top:auto;}");
        assert_eq!(resolve("left-auto").unwrap().as_str(), ".left-auto{left:auto;}");
    }

    #[test]
    fn test_inset_fraction() {
        assert_eq!(resolve("top-1/2").unwrap().as_str(), ".top-1\\/2{top:50%;}");
        assert_eq!(resolve("left-full").unwrap().as_str(), ".left-full{left:100%;}");
    }

    #[test]
    fn test_negative_inset() {
        assert_eq!(resolve("-top-4").unwrap().as_str(), ".-top-4{top:-1rem;}");
        assert_eq!(resolve("-inset-x-2").unwrap().as_str(), ".-inset-x-2{left:-0.5rem;right:-0.5rem;}");
    }

    #[test]
    fn test_z_index() {
        assert_eq!(resolve("z-10").unwrap().as_str(), ".z-10{z-index:10;}");
        assert_eq!(resolve("z-0").unwrap().as_str(), ".z-0{z-index:0;}");
        assert_eq!(resolve("z-auto").unwrap().as_str(), ".z-auto{z-index:auto;}");
    }
}
