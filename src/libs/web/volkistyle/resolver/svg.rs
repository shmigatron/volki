//! SVG utilities — fill, stroke.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32};
use crate::libs::web::volkistyle::palette;

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        "fill-none" => "fill:none;",
        "fill-current" => "fill:currentColor;",
        "fill-inherit" => "fill:inherit;",
        "stroke-none" => "stroke:none;",
        "stroke-current" => "stroke:currentColor;",
        "stroke-inherit" => "stroke:inherit;",

        _ => {
            // Fill color
            if let Some(rest) = class.strip_prefix("fill-") {
                let hex = palette::color_hex(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("fill:{};", hex)));
            }

            // Stroke width or color
            if let Some(rest) = class.strip_prefix("stroke-") {
                // Try as width
                if let Some(n) = parse_u32(rest) {
                    return Some(ResolvedUtility::Standard(crate::vformat!("stroke-width:{};", n)));
                }
                // Try as color
                if let Some(hex) = palette::color_hex(rest) {
                    return Some(ResolvedUtility::Standard(crate::vformat!("stroke:{};", hex)));
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
    fn test_fill() {
        assert_eq!(resolve("fill-none").unwrap().as_str(), ".fill-none{fill:none;}");
        assert_eq!(resolve("fill-current").unwrap().as_str(), ".fill-current{fill:currentColor;}");
        assert_eq!(resolve("fill-red-500").unwrap().as_str(), ".fill-red-500{fill:#ef4444;}");
    }

    #[test]
    fn test_stroke() {
        assert_eq!(resolve("stroke-none").unwrap().as_str(), ".stroke-none{stroke:none;}");
        assert_eq!(resolve("stroke-current").unwrap().as_str(), ".stroke-current{stroke:currentColor;}");
        assert_eq!(resolve("stroke-2").unwrap().as_str(), ".stroke-2{stroke-width:2;}");
        assert_eq!(resolve("stroke-blue-500").unwrap().as_str(), ".stroke-blue-500{stroke:#3b82f6;}");
    }
}
