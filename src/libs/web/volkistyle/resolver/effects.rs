//! Effects utilities — shadow, opacity, mix-blend-mode, bg-blend-mode.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Box shadow
        "shadow" => "box-shadow:0 1px 3px 0 rgba(0,0,0,0.1),0 1px 2px -1px rgba(0,0,0,0.1);",
        "shadow-sm" => "box-shadow:0 1px 2px 0 rgba(0,0,0,0.05);",
        "shadow-md" => "box-shadow:0 4px 6px -1px rgba(0,0,0,0.1),0 2px 4px -2px rgba(0,0,0,0.1);",
        "shadow-lg" => "box-shadow:0 10px 15px -3px rgba(0,0,0,0.1),0 4px 6px -4px rgba(0,0,0,0.1);",
        "shadow-xl" => "box-shadow:0 20px 25px -5px rgba(0,0,0,0.1),0 8px 10px -6px rgba(0,0,0,0.1);",
        "shadow-2xl" => "box-shadow:0 25px 50px -12px rgba(0,0,0,0.25);",
        "shadow-inner" => "box-shadow:inset 0 2px 4px 0 rgba(0,0,0,0.05);",
        "shadow-none" => "box-shadow:0 0 #0000;",

        // Mix blend mode
        "mix-blend-normal" => "mix-blend-mode:normal;",
        "mix-blend-multiply" => "mix-blend-mode:multiply;",
        "mix-blend-screen" => "mix-blend-mode:screen;",
        "mix-blend-overlay" => "mix-blend-mode:overlay;",
        "mix-blend-darken" => "mix-blend-mode:darken;",
        "mix-blend-lighten" => "mix-blend-mode:lighten;",
        "mix-blend-color-dodge" => "mix-blend-mode:color-dodge;",
        "mix-blend-color-burn" => "mix-blend-mode:color-burn;",
        "mix-blend-hard-light" => "mix-blend-mode:hard-light;",
        "mix-blend-soft-light" => "mix-blend-mode:soft-light;",
        "mix-blend-difference" => "mix-blend-mode:difference;",
        "mix-blend-exclusion" => "mix-blend-mode:exclusion;",
        "mix-blend-hue" => "mix-blend-mode:hue;",
        "mix-blend-saturation" => "mix-blend-mode:saturation;",
        "mix-blend-color" => "mix-blend-mode:color;",
        "mix-blend-luminosity" => "mix-blend-mode:luminosity;",
        "mix-blend-plus-lighter" => "mix-blend-mode:plus-lighter;",

        // Background blend mode
        "bg-blend-normal" => "background-blend-mode:normal;",
        "bg-blend-multiply" => "background-blend-mode:multiply;",
        "bg-blend-screen" => "background-blend-mode:screen;",
        "bg-blend-overlay" => "background-blend-mode:overlay;",
        "bg-blend-darken" => "background-blend-mode:darken;",
        "bg-blend-lighten" => "background-blend-mode:lighten;",
        "bg-blend-color-dodge" => "background-blend-mode:color-dodge;",
        "bg-blend-color-burn" => "background-blend-mode:color-burn;",
        "bg-blend-hard-light" => "background-blend-mode:hard-light;",
        "bg-blend-soft-light" => "background-blend-mode:soft-light;",
        "bg-blend-difference" => "background-blend-mode:difference;",
        "bg-blend-exclusion" => "background-blend-mode:exclusion;",
        "bg-blend-hue" => "background-blend-mode:hue;",
        "bg-blend-saturation" => "background-blend-mode:saturation;",
        "bg-blend-color" => "background-blend-mode:color;",
        "bg-blend-luminosity" => "background-blend-mode:luminosity;",

        _ => {
            // Opacity
            if let Some(rest) = class.strip_prefix("opacity-") {
                let n = parse_u32(rest)?;
                if n > 100 { return None; }
                let decl = if n == 0 {
                    String::from("opacity:0;")
                } else if n == 100 {
                    String::from("opacity:1;")
                } else if n % 10 == 0 {
                    crate::vformat!("opacity:0.{};", n / 10)
                } else {
                    crate::vformat!("opacity:0.{};", n)
                };
                return Some(ResolvedUtility::Standard(decl));
            }

            // Shadow color
            if let Some(rest) = class.strip_prefix("shadow-") {
                if let Some(hex) = crate::libs::web::volkistyle::palette::color_hex(rest) {
                    return Some(ResolvedUtility::Standard(crate::vformat!("--tw-shadow-color:{};", hex)));
                }
                return None;
            }

            return None;
        }
    };
    Some(ResolvedUtility::Standard(String::from(decls)))
}

#[cfg(test)]
mod tests {
    use super::super::resolve;

    #[test]
    fn test_shadow() {
        assert!(resolve("shadow").unwrap().as_str().contains("box-shadow:"));
        assert!(resolve("shadow-lg").unwrap().as_str().contains("box-shadow:"));
        assert_eq!(resolve("shadow-none").unwrap().as_str(), ".shadow-none{box-shadow:0 0 #0000;}");
        assert!(resolve("shadow-inner").unwrap().as_str().contains("inset"));
    }

    #[test]
    fn test_opacity() {
        assert_eq!(resolve("opacity-0").unwrap().as_str(), ".opacity-0{opacity:0;}");
        assert_eq!(resolve("opacity-50").unwrap().as_str(), ".opacity-50{opacity:0.5;}");
        assert_eq!(resolve("opacity-100").unwrap().as_str(), ".opacity-100{opacity:1;}");
        assert_eq!(resolve("opacity-75").unwrap().as_str(), ".opacity-75{opacity:0.75;}");
    }

    #[test]
    fn test_mix_blend() {
        assert_eq!(resolve("mix-blend-multiply").unwrap().as_str(), ".mix-blend-multiply{mix-blend-mode:multiply;}");
    }

    #[test]
    fn test_bg_blend() {
        assert_eq!(resolve("bg-blend-overlay").unwrap().as_str(), ".bg-blend-overlay{background-blend-mode:overlay;}");
    }
}
