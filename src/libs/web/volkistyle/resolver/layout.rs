//! Layout utilities — display, position, float, clear, visibility, overflow, etc.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Display
        "block" => "display:block;",
        "inline" => "display:inline;",
        "inline-block" => "display:inline-block;",
        "flex" => "display:flex;",
        "inline-flex" => "display:inline-flex;",
        "grid" => "display:grid;",
        "inline-grid" => "display:inline-grid;",
        "hidden" => "display:none;",
        "table" => "display:table;",
        "table-row" => "display:table-row;",
        "table-cell" => "display:table-cell;",
        "table-caption" => "display:table-caption;",
        "table-column" => "display:table-column;",
        "table-column-group" => "display:table-column-group;",
        "table-footer-group" => "display:table-footer-group;",
        "table-header-group" => "display:table-header-group;",
        "table-row-group" => "display:table-row-group;",
        "contents" => "display:contents;",
        "list-item" => "display:list-item;",
        "flow-root" => "display:flow-root;",
        "container" => "width:100%;",

        // Position
        "relative" => "position:relative;",
        "absolute" => "position:absolute;",
        "fixed" => "position:fixed;",
        "sticky" => "position:sticky;",
        "static" => "position:static;",

        // Float / Clear
        "float-right" => "float:right;",
        "float-left" => "float:left;",
        "float-none" => "float:none;",
        "clear-left" => "clear:left;",
        "clear-right" => "clear:right;",
        "clear-both" => "clear:both;",
        "clear-none" => "clear:none;",

        // Visibility
        "visible" => "visibility:visible;",
        "invisible" => "visibility:hidden;",
        "collapse" => "visibility:collapse;",

        // Box sizing
        "box-border" => "box-sizing:border-box;",
        "box-content" => "box-sizing:content-box;",

        // Isolation
        "isolate" => "isolation:isolate;",
        "isolation-auto" => "isolation:auto;",

        // Aspect ratio
        "aspect-auto" => "aspect-ratio:auto;",
        "aspect-square" => "aspect-ratio:1 / 1;",
        "aspect-video" => "aspect-ratio:16 / 9;",

        // Object fit
        "object-contain" => "object-fit:contain;",
        "object-cover" => "object-fit:cover;",
        "object-fill" => "object-fit:fill;",
        "object-none" => "object-fit:none;",
        "object-scale-down" => "object-fit:scale-down;",

        // Object position
        "object-bottom" => "object-position:bottom;",
        "object-center" => "object-position:center;",
        "object-left" => "object-position:left;",
        "object-left-bottom" => "object-position:left bottom;",
        "object-left-top" => "object-position:left top;",
        "object-right" => "object-position:right;",
        "object-right-bottom" => "object-position:right bottom;",
        "object-right-top" => "object-position:right top;",
        "object-top" => "object-position:top;",

        // Overflow
        "overflow-hidden" => "overflow:hidden;",
        "overflow-auto" => "overflow:auto;",
        "overflow-scroll" => "overflow:scroll;",
        "overflow-visible" => "overflow:visible;",
        "overflow-clip" => "overflow:clip;",
        "overflow-x-auto" => "overflow-x:auto;",
        "overflow-y-auto" => "overflow-y:auto;",
        "overflow-x-hidden" => "overflow-x:hidden;",
        "overflow-y-hidden" => "overflow-y:hidden;",
        "overflow-x-clip" => "overflow-x:clip;",
        "overflow-y-clip" => "overflow-y:clip;",
        "overflow-x-visible" => "overflow-x:visible;",
        "overflow-y-visible" => "overflow-y:visible;",
        "overflow-x-scroll" => "overflow-x:scroll;",
        "overflow-y-scroll" => "overflow-y:scroll;",

        // Overscroll
        "overscroll-auto" => "overscroll-behavior:auto;",
        "overscroll-contain" => "overscroll-behavior:contain;",
        "overscroll-none" => "overscroll-behavior:none;",
        "overscroll-x-auto" => "overscroll-behavior-x:auto;",
        "overscroll-x-contain" => "overscroll-behavior-x:contain;",
        "overscroll-x-none" => "overscroll-behavior-x:none;",
        "overscroll-y-auto" => "overscroll-behavior-y:auto;",
        "overscroll-y-contain" => "overscroll-behavior-y:contain;",
        "overscroll-y-none" => "overscroll-behavior-y:none;",

        // Break after
        "break-after-auto" => "break-after:auto;",
        "break-after-avoid" => "break-after:avoid;",
        "break-after-all" => "break-after:all;",
        "break-after-avoid-page" => "break-after:avoid-page;",
        "break-after-page" => "break-after:page;",
        "break-after-left" => "break-after:left;",
        "break-after-right" => "break-after:right;",
        "break-after-column" => "break-after:column;",

        // Break before
        "break-before-auto" => "break-before:auto;",
        "break-before-avoid" => "break-before:avoid;",
        "break-before-all" => "break-before:all;",
        "break-before-avoid-page" => "break-before:avoid-page;",
        "break-before-page" => "break-before:page;",
        "break-before-left" => "break-before:left;",
        "break-before-right" => "break-before:right;",
        "break-before-column" => "break-before:column;",

        // Break inside
        "break-inside-auto" => "break-inside:auto;",
        "break-inside-avoid" => "break-inside:avoid;",
        "break-inside-avoid-page" => "break-inside:avoid-page;",
        "break-inside-avoid-column" => "break-inside:avoid-column;",

        // Screen reader
        "sr-only" => "position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border-width:0;",
        "not-sr-only" => "position:static;width:auto;height:auto;padding:0;margin:0;overflow:visible;clip:auto;white-space:normal;",

        _ => {
            // Columns prefix
            if let Some(rest) = class.strip_prefix("columns-") {
                let decl = match rest {
                    "auto" => "columns:auto;",
                    "3xs" => "columns:16rem;",
                    "2xs" => "columns:18rem;",
                    "xs" => "columns:20rem;",
                    "sm" => "columns:24rem;",
                    "md" => "columns:28rem;",
                    "lg" => "columns:32rem;",
                    "xl" => "columns:36rem;",
                    "2xl" => "columns:42rem;",
                    "3xl" => "columns:48rem;",
                    "4xl" => "columns:56rem;",
                    "5xl" => "columns:64rem;",
                    "6xl" => "columns:72rem;",
                    "7xl" => "columns:80rem;",
                    _ => {
                        let n = parse_u32(rest)?;
                        if n < 1 || n > 12 { return None; }
                        return Some(ResolvedUtility::Standard(crate::vformat!("columns:{};", n)));
                    }
                };
                return Some(ResolvedUtility::Standard(String::from(decl)));
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
    fn test_display() {
        assert_eq!(resolve("flex").unwrap().as_str(), ".flex{display:flex;}");
        assert_eq!(resolve("hidden").unwrap().as_str(), ".hidden{display:none;}");
        assert_eq!(resolve("inline-grid").unwrap().as_str(), ".inline-grid{display:inline-grid;}");
        assert_eq!(resolve("contents").unwrap().as_str(), ".contents{display:contents;}");
        assert_eq!(resolve("flow-root").unwrap().as_str(), ".flow-root{display:flow-root;}");
    }

    #[test]
    fn test_position() {
        assert_eq!(resolve("absolute").unwrap().as_str(), ".absolute{position:absolute;}");
        assert_eq!(resolve("sticky").unwrap().as_str(), ".sticky{position:sticky;}");
    }

    #[test]
    fn test_float_clear() {
        assert_eq!(resolve("float-right").unwrap().as_str(), ".float-right{float:right;}");
        assert_eq!(resolve("clear-both").unwrap().as_str(), ".clear-both{clear:both;}");
    }

    #[test]
    fn test_visibility() {
        assert_eq!(resolve("visible").unwrap().as_str(), ".visible{visibility:visible;}");
        assert_eq!(resolve("invisible").unwrap().as_str(), ".invisible{visibility:hidden;}");
        assert_eq!(resolve("collapse").unwrap().as_str(), ".collapse{visibility:collapse;}");
    }

    #[test]
    fn test_box_sizing() {
        assert_eq!(resolve("box-border").unwrap().as_str(), ".box-border{box-sizing:border-box;}");
    }

    #[test]
    fn test_aspect_ratio() {
        assert_eq!(resolve("aspect-video").unwrap().as_str(), ".aspect-video{aspect-ratio:16 / 9;}");
    }

    #[test]
    fn test_object_fit() {
        assert_eq!(resolve("object-cover").unwrap().as_str(), ".object-cover{object-fit:cover;}");
    }

    #[test]
    fn test_columns() {
        assert_eq!(resolve("columns-3").unwrap().as_str(), ".columns-3{columns:3;}");
        assert_eq!(resolve("columns-auto").unwrap().as_str(), ".columns-auto{columns:auto;}");
        assert_eq!(resolve("columns-sm").unwrap().as_str(), ".columns-sm{columns:24rem;}");
    }

    #[test]
    fn test_overflow() {
        assert_eq!(resolve("overflow-hidden").unwrap().as_str(), ".overflow-hidden{overflow:hidden;}");
        assert_eq!(resolve("overflow-x-auto").unwrap().as_str(), ".overflow-x-auto{overflow-x:auto;}");
    }

    #[test]
    fn test_overscroll() {
        assert_eq!(resolve("overscroll-contain").unwrap().as_str(), ".overscroll-contain{overscroll-behavior:contain;}");
    }
}
