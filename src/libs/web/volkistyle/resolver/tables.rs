//! Table utilities — layout, border-collapse, border-spacing, caption.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_spacing_value};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        "table-auto" => "table-layout:auto;",
        "table-fixed" => "table-layout:fixed;",
        "border-collapse" => "border-collapse:collapse;",
        "border-separate" => "border-collapse:separate;",
        "caption-top" => "caption-side:top;",
        "caption-bottom" => "caption-side:bottom;",

        _ => {
            // Border spacing
            if let Some(rest) = class.strip_prefix("border-spacing-x-") {
                let val = parse_spacing_value(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("border-spacing:{} 0;", val)));
            }
            if let Some(rest) = class.strip_prefix("border-spacing-y-") {
                let val = parse_spacing_value(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("border-spacing:0 {};", val)));
            }
            if let Some(rest) = class.strip_prefix("border-spacing-") {
                let val = parse_spacing_value(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("border-spacing:{};", val)));
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
    fn test_table_layout() {
        assert_eq!(resolve("table-auto").unwrap().as_str(), ".table-auto{table-layout:auto;}");
        assert_eq!(resolve("table-fixed").unwrap().as_str(), ".table-fixed{table-layout:fixed;}");
    }

    #[test]
    fn test_border_collapse() {
        assert_eq!(resolve("border-collapse").unwrap().as_str(), ".border-collapse{border-collapse:collapse;}");
        assert_eq!(resolve("border-separate").unwrap().as_str(), ".border-separate{border-collapse:separate;}");
    }

    #[test]
    fn test_border_spacing() {
        assert_eq!(resolve("border-spacing-2").unwrap().as_str(), ".border-spacing-2{border-spacing:0.5rem;}");
        assert_eq!(resolve("border-spacing-x-4").unwrap().as_str(), ".border-spacing-x-4{border-spacing:1rem 0;}");
        assert_eq!(resolve("border-spacing-y-4").unwrap().as_str(), ".border-spacing-y-4{border-spacing:0 1rem;}");
    }

    #[test]
    fn test_caption() {
        assert_eq!(resolve("caption-top").unwrap().as_str(), ".caption-top{caption-side:top;}");
        assert_eq!(resolve("caption-bottom").unwrap().as_str(), ".caption-bottom{caption-side:bottom;}");
    }
}
