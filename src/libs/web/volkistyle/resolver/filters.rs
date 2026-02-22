//! Filter utilities — blur, brightness, contrast, saturate, grayscale, invert, sepia,
//! hue-rotate, drop-shadow, and all backdrop-* equivalents.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Blur
        "blur-none" => "filter:blur(0);",
        "blur-sm" => "filter:blur(4px);",
        "blur" => "filter:blur(8px);",
        "blur-md" => "filter:blur(12px);",
        "blur-lg" => "filter:blur(16px);",
        "blur-xl" => "filter:blur(24px);",
        "blur-2xl" => "filter:blur(40px);",
        "blur-3xl" => "filter:blur(64px);",

        // Grayscale
        "grayscale" => "filter:grayscale(100%);",
        "grayscale-0" => "filter:grayscale(0);",

        // Invert
        "invert" => "filter:invert(100%);",
        "invert-0" => "filter:invert(0);",

        // Sepia
        "sepia" => "filter:sepia(100%);",
        "sepia-0" => "filter:sepia(0);",

        // Drop shadow
        "drop-shadow-sm" => "filter:drop-shadow(0 1px 1px rgba(0,0,0,0.05));",
        "drop-shadow" => "filter:drop-shadow(0 1px 2px rgba(0,0,0,0.1)) drop-shadow(0 1px 1px rgba(0,0,0,0.06));",
        "drop-shadow-md" => "filter:drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));",
        "drop-shadow-lg" => "filter:drop-shadow(0 10px 8px rgba(0,0,0,0.04)) drop-shadow(0 4px 3px rgba(0,0,0,0.1));",
        "drop-shadow-xl" => "filter:drop-shadow(0 20px 13px rgba(0,0,0,0.03)) drop-shadow(0 8px 5px rgba(0,0,0,0.08));",
        "drop-shadow-2xl" => "filter:drop-shadow(0 25px 25px rgba(0,0,0,0.15));",
        "drop-shadow-none" => "filter:drop-shadow(0 0 #0000);",

        // Backdrop blur
        "backdrop-blur-none" => "backdrop-filter:blur(0);",
        "backdrop-blur-sm" => "backdrop-filter:blur(4px);",
        "backdrop-blur" => "backdrop-filter:blur(8px);",
        "backdrop-blur-md" => "backdrop-filter:blur(12px);",
        "backdrop-blur-lg" => "backdrop-filter:blur(16px);",
        "backdrop-blur-xl" => "backdrop-filter:blur(24px);",
        "backdrop-blur-2xl" => "backdrop-filter:blur(40px);",
        "backdrop-blur-3xl" => "backdrop-filter:blur(64px);",

        // Backdrop grayscale
        "backdrop-grayscale" => "backdrop-filter:grayscale(100%);",
        "backdrop-grayscale-0" => "backdrop-filter:grayscale(0);",

        // Backdrop invert
        "backdrop-invert" => "backdrop-filter:invert(100%);",
        "backdrop-invert-0" => "backdrop-filter:invert(0);",

        // Backdrop sepia
        "backdrop-sepia" => "backdrop-filter:sepia(100%);",
        "backdrop-sepia-0" => "backdrop-filter:sepia(0);",

        // Backdrop opacity
        "backdrop-opacity-0" => "backdrop-filter:opacity(0);",
        "backdrop-opacity-100" => "backdrop-filter:opacity(1);",

        _ => {
            return resolve_prefix(class);
        }
    };
    Some(ResolvedUtility::Standard(String::from(decls)))
}

fn resolve_prefix(class: &str) -> Option<ResolvedUtility> {
    // Brightness
    if let Some(rest) = class.strip_prefix("brightness-") {
        let n = parse_u32(rest)?;
        let val = filter_percent(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("filter:brightness({});", val)));
    }

    // Contrast
    if let Some(rest) = class.strip_prefix("contrast-") {
        let n = parse_u32(rest)?;
        let val = filter_percent(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("filter:contrast({});", val)));
    }

    // Saturate
    if let Some(rest) = class.strip_prefix("saturate-") {
        let n = parse_u32(rest)?;
        let val = filter_percent(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("filter:saturate({});", val)));
    }

    // Hue rotate
    if let Some(rest) = class.strip_prefix("-hue-rotate-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("filter:hue-rotate(-{}deg);", n)));
    }
    if let Some(rest) = class.strip_prefix("hue-rotate-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("filter:hue-rotate({}deg);", n)));
    }

    // Backdrop brightness
    if let Some(rest) = class.strip_prefix("backdrop-brightness-") {
        let n = parse_u32(rest)?;
        let val = filter_percent(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("backdrop-filter:brightness({});", val)));
    }

    // Backdrop contrast
    if let Some(rest) = class.strip_prefix("backdrop-contrast-") {
        let n = parse_u32(rest)?;
        let val = filter_percent(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("backdrop-filter:contrast({});", val)));
    }

    // Backdrop saturate
    if let Some(rest) = class.strip_prefix("backdrop-saturate-") {
        let n = parse_u32(rest)?;
        let val = filter_percent(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("backdrop-filter:saturate({});", val)));
    }

    // Backdrop hue-rotate
    if let Some(rest) = class.strip_prefix("backdrop-hue-rotate-") {
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("backdrop-filter:hue-rotate({}deg);", n)));
    }

    // Backdrop opacity (numeric)
    if let Some(rest) = class.strip_prefix("backdrop-opacity-") {
        let n = parse_u32(rest)?;
        if n > 100 { return None; }
        let val = filter_percent(n);
        return Some(ResolvedUtility::Standard(crate::vformat!("backdrop-filter:opacity({});", val)));
    }

    None
}

fn filter_percent(n: u32) -> String {
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
    fn test_blur() {
        assert_eq!(resolve("blur").unwrap().as_str(), ".blur{filter:blur(8px);}");
        assert_eq!(resolve("blur-lg").unwrap().as_str(), ".blur-lg{filter:blur(16px);}");
        assert_eq!(resolve("blur-none").unwrap().as_str(), ".blur-none{filter:blur(0);}");
    }

    #[test]
    fn test_brightness() {
        assert_eq!(resolve("brightness-50").unwrap().as_str(), ".brightness-50{filter:brightness(0.5);}");
        assert_eq!(resolve("brightness-100").unwrap().as_str(), ".brightness-100{filter:brightness(1);}");
        assert_eq!(resolve("brightness-150").unwrap().as_str(), ".brightness-150{filter:brightness(1.5);}");
    }

    #[test]
    fn test_contrast() {
        assert_eq!(resolve("contrast-0").unwrap().as_str(), ".contrast-0{filter:contrast(0);}");
        assert_eq!(resolve("contrast-100").unwrap().as_str(), ".contrast-100{filter:contrast(1);}");
    }

    #[test]
    fn test_grayscale() {
        assert_eq!(resolve("grayscale").unwrap().as_str(), ".grayscale{filter:grayscale(100%);}");
        assert_eq!(resolve("grayscale-0").unwrap().as_str(), ".grayscale-0{filter:grayscale(0);}");
    }

    #[test]
    fn test_invert() {
        assert_eq!(resolve("invert").unwrap().as_str(), ".invert{filter:invert(100%);}");
    }

    #[test]
    fn test_sepia() {
        assert_eq!(resolve("sepia").unwrap().as_str(), ".sepia{filter:sepia(100%);}");
    }

    #[test]
    fn test_hue_rotate() {
        assert_eq!(resolve("hue-rotate-90").unwrap().as_str(), ".hue-rotate-90{filter:hue-rotate(90deg);}");
        assert_eq!(resolve("-hue-rotate-15").unwrap().as_str(), ".-hue-rotate-15{filter:hue-rotate(-15deg);}");
    }

    #[test]
    fn test_drop_shadow() {
        assert!(resolve("drop-shadow").unwrap().as_str().contains("filter:drop-shadow("));
        assert_eq!(resolve("drop-shadow-none").unwrap().as_str(), ".drop-shadow-none{filter:drop-shadow(0 0 #0000);}");
    }

    #[test]
    fn test_backdrop_blur() {
        assert_eq!(resolve("backdrop-blur").unwrap().as_str(), ".backdrop-blur{backdrop-filter:blur(8px);}");
        assert_eq!(resolve("backdrop-blur-lg").unwrap().as_str(), ".backdrop-blur-lg{backdrop-filter:blur(16px);}");
    }

    #[test]
    fn test_backdrop_brightness() {
        assert_eq!(resolve("backdrop-brightness-75").unwrap().as_str(), ".backdrop-brightness-75{backdrop-filter:brightness(0.75);}");
    }

    #[test]
    fn test_saturate() {
        assert_eq!(resolve("saturate-50").unwrap().as_str(), ".saturate-50{filter:saturate(0.5);}");
    }
}
