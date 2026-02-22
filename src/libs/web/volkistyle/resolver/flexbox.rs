//! Flexbox utilities — direction, wrap, grow, shrink, align, justify, order.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Direction
        "flex-row" => "flex-direction:row;",
        "flex-col" => "flex-direction:column;",
        "flex-row-reverse" => "flex-direction:row-reverse;",
        "flex-col-reverse" => "flex-direction:column-reverse;",

        // Wrap
        "flex-wrap" => "flex-wrap:wrap;",
        "flex-nowrap" => "flex-wrap:nowrap;",
        "flex-wrap-reverse" => "flex-wrap:wrap-reverse;",

        // Flex sizing
        "flex-1" => "flex:1 1 0%;",
        "flex-auto" => "flex:1 1 auto;",
        "flex-initial" => "flex:0 1 auto;",
        "flex-none" => "flex:none;",
        "flex-grow" => "flex-grow:1;",
        "flex-grow-0" => "flex-grow:0;",
        "flex-shrink" => "flex-shrink:1;",
        "flex-shrink-0" => "flex-shrink:0;",

        // Align items
        "items-center" => "align-items:center;",
        "items-start" => "align-items:flex-start;",
        "items-end" => "align-items:flex-end;",
        "items-stretch" => "align-items:stretch;",
        "items-baseline" => "align-items:baseline;",

        // Justify content
        "justify-center" => "justify-content:center;",
        "justify-between" => "justify-content:space-between;",
        "justify-around" => "justify-content:space-around;",
        "justify-evenly" => "justify-content:space-evenly;",
        "justify-start" => "justify-content:flex-start;",
        "justify-end" => "justify-content:flex-end;",
        "justify-normal" => "justify-content:normal;",
        "justify-stretch" => "justify-content:stretch;",

        // Justify items
        "justify-items-start" => "justify-items:start;",
        "justify-items-end" => "justify-items:end;",
        "justify-items-center" => "justify-items:center;",
        "justify-items-stretch" => "justify-items:stretch;",

        // Justify self
        "justify-self-auto" => "justify-self:auto;",
        "justify-self-start" => "justify-self:start;",
        "justify-self-end" => "justify-self:end;",
        "justify-self-center" => "justify-self:center;",
        "justify-self-stretch" => "justify-self:stretch;",

        // Align content
        "content-normal" => "align-content:normal;",
        "content-center" => "align-content:center;",
        "content-start" => "align-content:flex-start;",
        "content-end" => "align-content:flex-end;",
        "content-between" => "align-content:space-between;",
        "content-around" => "align-content:space-around;",
        "content-evenly" => "align-content:space-evenly;",
        "content-baseline" => "align-content:baseline;",
        "content-stretch" => "align-content:stretch;",

        // Align self
        "self-auto" => "align-self:auto;",
        "self-start" => "align-self:flex-start;",
        "self-end" => "align-self:flex-end;",
        "self-center" => "align-self:center;",
        "self-stretch" => "align-self:stretch;",
        "self-baseline" => "align-self:baseline;",

        // Place content
        "place-content-center" => "place-content:center;",
        "place-content-start" => "place-content:start;",
        "place-content-end" => "place-content:end;",
        "place-content-between" => "place-content:space-between;",
        "place-content-around" => "place-content:space-around;",
        "place-content-evenly" => "place-content:space-evenly;",
        "place-content-baseline" => "place-content:baseline;",
        "place-content-stretch" => "place-content:stretch;",

        // Place items
        "place-items-start" => "place-items:start;",
        "place-items-end" => "place-items:end;",
        "place-items-center" => "place-items:center;",
        "place-items-baseline" => "place-items:baseline;",
        "place-items-stretch" => "place-items:stretch;",

        // Place self
        "place-self-auto" => "place-self:auto;",
        "place-self-start" => "place-self:start;",
        "place-self-end" => "place-self:end;",
        "place-self-center" => "place-self:center;",
        "place-self-stretch" => "place-self:stretch;",

        _ => {
            if let Some(rest) = class.strip_prefix("grow-") {
                let n = parse_u32(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("flex-grow:{};", n)));
            }
            if let Some(rest) = class.strip_prefix("shrink-") {
                let n = parse_u32(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("flex-shrink:{};", n)));
            }
            // Order
            if let Some(rest) = class.strip_prefix("order-") {
                let decl = match rest {
                    "first" => String::from("order:-9999;"),
                    "last" => String::from("order:9999;"),
                    "none" => String::from("order:0;"),
                    _ => {
                        let n = parse_u32(rest)?;
                        crate::vformat!("order:{};", n)
                    }
                };
                return Some(ResolvedUtility::Standard(decl));
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
    fn test_flex_direction() {
        assert_eq!(resolve("flex-row").unwrap().as_str(), ".flex-row{flex-direction:row;}");
        assert_eq!(resolve("flex-col").unwrap().as_str(), ".flex-col{flex-direction:column;}");
    }

    #[test]
    fn test_flex_wrap() {
        assert_eq!(resolve("flex-wrap").unwrap().as_str(), ".flex-wrap{flex-wrap:wrap;}");
        assert_eq!(resolve("flex-wrap-reverse").unwrap().as_str(), ".flex-wrap-reverse{flex-wrap:wrap-reverse;}");
    }

    #[test]
    fn test_items() {
        assert_eq!(resolve("items-center").unwrap().as_str(), ".items-center{align-items:center;}");
    }

    #[test]
    fn test_justify() {
        assert_eq!(resolve("justify-between").unwrap().as_str(), ".justify-between{justify-content:space-between;}");
    }

    #[test]
    fn test_self_align() {
        assert_eq!(resolve("self-center").unwrap().as_str(), ".self-center{align-self:center;}");
    }

    #[test]
    fn test_place_content() {
        assert_eq!(resolve("place-content-center").unwrap().as_str(), ".place-content-center{place-content:center;}");
    }

    #[test]
    fn test_order() {
        assert_eq!(resolve("order-1").unwrap().as_str(), ".order-1{order:1;}");
        assert_eq!(resolve("order-first").unwrap().as_str(), ".order-first{order:-9999;}");
        assert_eq!(resolve("order-last").unwrap().as_str(), ".order-last{order:9999;}");
    }
}
