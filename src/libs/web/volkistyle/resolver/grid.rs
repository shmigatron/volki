//! Grid utilities — columns, rows, spans, flow, auto-cols/rows.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Grid flow
        "grid-flow-row" => "grid-auto-flow:row;",
        "grid-flow-col" => "grid-auto-flow:column;",
        "grid-flow-dense" => "grid-auto-flow:dense;",
        "grid-flow-row-dense" => "grid-auto-flow:row dense;",
        "grid-flow-col-dense" => "grid-auto-flow:column dense;",

        // Auto cols
        "auto-cols-auto" => "grid-auto-columns:auto;",
        "auto-cols-min" => "grid-auto-columns:min-content;",
        "auto-cols-max" => "grid-auto-columns:max-content;",
        "auto-cols-fr" => "grid-auto-columns:minmax(0,1fr);",

        // Auto rows
        "auto-rows-auto" => "grid-auto-rows:auto;",
        "auto-rows-min" => "grid-auto-rows:min-content;",
        "auto-rows-max" => "grid-auto-rows:max-content;",
        "auto-rows-fr" => "grid-auto-rows:minmax(0,1fr);",

        // Col span helpers
        "col-auto" => "grid-column:auto;",
        "col-span-full" => "grid-column:1 / -1;",

        // Row span helpers
        "row-auto" => "grid-row:auto;",
        "row-span-full" => "grid-row:1 / -1;",

        _ => {
            // grid-cols-{n}
            if let Some(rest) = class.strip_prefix("grid-cols-") {
                let decl = match rest {
                    "none" => String::from("grid-template-columns:none;"),
                    "subgrid" => String::from("grid-template-columns:subgrid;"),
                    _ => {
                        let n = parse_u32(rest)?;
                        if n < 1 || n > 12 { return None; }
                        crate::vformat!("grid-template-columns:repeat({},minmax(0,1fr));", n)
                    }
                };
                return Some(ResolvedUtility::Standard(decl));
            }

            // grid-rows-{n}
            if let Some(rest) = class.strip_prefix("grid-rows-") {
                let decl = match rest {
                    "none" => String::from("grid-template-rows:none;"),
                    "subgrid" => String::from("grid-template-rows:subgrid;"),
                    _ => {
                        let n = parse_u32(rest)?;
                        if n < 1 || n > 12 { return None; }
                        crate::vformat!("grid-template-rows:repeat({},minmax(0,1fr));", n)
                    }
                };
                return Some(ResolvedUtility::Standard(decl));
            }

            // col-span-{n}, col-start-{n}, col-end-{n}
            if let Some(rest) = class.strip_prefix("col-span-") {
                let n = parse_u32(rest)?;
                if n < 1 || n > 12 { return None; }
                return Some(ResolvedUtility::Standard(crate::vformat!("grid-column:span {} / span {};", n, n)));
            }
            if let Some(rest) = class.strip_prefix("col-start-") {
                let n = parse_u32(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("grid-column-start:{};", n)));
            }
            if let Some(rest) = class.strip_prefix("col-end-") {
                let n = parse_u32(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("grid-column-end:{};", n)));
            }

            // row-span-{n}, row-start-{n}, row-end-{n}
            if let Some(rest) = class.strip_prefix("row-span-") {
                let n = parse_u32(rest)?;
                if n < 1 || n > 12 { return None; }
                return Some(ResolvedUtility::Standard(crate::vformat!("grid-row:span {} / span {};", n, n)));
            }
            if let Some(rest) = class.strip_prefix("row-start-") {
                let n = parse_u32(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("grid-row-start:{};", n)));
            }
            if let Some(rest) = class.strip_prefix("row-end-") {
                let n = parse_u32(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("grid-row-end:{};", n)));
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
    fn test_grid_cols() {
        assert!(resolve("grid-cols-3").unwrap().as_str().contains("repeat(3,minmax(0,1fr))"));
        assert!(resolve("grid-cols-12").unwrap().as_str().contains("repeat(12,"));
        assert_eq!(resolve("grid-cols-none").unwrap().as_str(), ".grid-cols-none{grid-template-columns:none;}");
    }

    #[test]
    fn test_grid_rows() {
        assert!(resolve("grid-rows-6").unwrap().as_str().contains("repeat(6,minmax(0,1fr))"));
    }

    #[test]
    fn test_col_span() {
        assert_eq!(resolve("col-span-2").unwrap().as_str(), ".col-span-2{grid-column:span 2 / span 2;}");
        assert_eq!(resolve("col-span-full").unwrap().as_str(), ".col-span-full{grid-column:1 / -1;}");
    }

    #[test]
    fn test_col_start_end() {
        assert_eq!(resolve("col-start-1").unwrap().as_str(), ".col-start-1{grid-column-start:1;}");
        assert_eq!(resolve("col-end-3").unwrap().as_str(), ".col-end-3{grid-column-end:3;}");
    }

    #[test]
    fn test_row_span() {
        assert_eq!(resolve("row-span-2").unwrap().as_str(), ".row-span-2{grid-row:span 2 / span 2;}");
    }

    #[test]
    fn test_grid_flow() {
        assert_eq!(resolve("grid-flow-col").unwrap().as_str(), ".grid-flow-col{grid-auto-flow:column;}");
        assert_eq!(resolve("grid-flow-row-dense").unwrap().as_str(), ".grid-flow-row-dense{grid-auto-flow:row dense;}");
    }

    #[test]
    fn test_auto_cols_rows() {
        assert_eq!(resolve("auto-cols-fr").unwrap().as_str(), ".auto-cols-fr{grid-auto-columns:minmax(0,1fr);}");
        assert_eq!(resolve("auto-rows-min").unwrap().as_str(), ".auto-rows-min{grid-auto-rows:min-content;}");
    }
}
