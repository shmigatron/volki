//! Typography utilities — text size/color, font, leading, tracking, decoration, etc.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32, spacing, resolve_color_with_opacity};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Text alignment
        "text-left" => "text-align:left;",
        "text-center" => "text-align:center;",
        "text-right" => "text-align:right;",
        "text-justify" => "text-align:justify;",
        "text-start" => "text-align:start;",
        "text-end" => "text-align:end;",

        // Text transform
        "uppercase" => "text-transform:uppercase;",
        "lowercase" => "text-transform:lowercase;",
        "capitalize" => "text-transform:capitalize;",
        "normal-case" => "text-transform:none;",

        // Font style
        "italic" => "font-style:italic;",
        "not-italic" => "font-style:normal;",

        // Text decoration line
        "underline" => "text-decoration-line:underline;",
        "no-underline" => "text-decoration-line:none;",
        "line-through" => "text-decoration-line:line-through;",
        "overline" => "text-decoration-line:overline;",

        // Text decoration style
        "decoration-solid" => "text-decoration-style:solid;",
        "decoration-dashed" => "text-decoration-style:dashed;",
        "decoration-dotted" => "text-decoration-style:dotted;",
        "decoration-double" => "text-decoration-style:double;",
        "decoration-wavy" => "text-decoration-style:wavy;",
        "decoration-auto" => "text-decoration-thickness:auto;",
        "decoration-from-font" => "text-decoration-thickness:from-font;",

        // Truncate / overflow
        "truncate" => "overflow:hidden;text-overflow:ellipsis;white-space:nowrap;",
        "text-ellipsis" => "text-overflow:ellipsis;",
        "text-clip" => "text-overflow:clip;",

        // Whitespace
        "whitespace-normal" => "white-space:normal;",
        "whitespace-nowrap" => "white-space:nowrap;",
        "whitespace-pre" => "white-space:pre;",
        "whitespace-pre-line" => "white-space:pre-line;",
        "whitespace-pre-wrap" => "white-space:pre-wrap;",
        "whitespace-break-spaces" => "white-space:break-spaces;",

        // Word break
        "break-normal" => "overflow-wrap:normal;word-break:normal;",
        "break-all" => "word-break:break-all;",
        "break-keep" => "word-break:keep-all;",
        "break-words" => "overflow-wrap:break-word;",

        // Text wrap
        "text-wrap" => "text-wrap:wrap;",
        "text-nowrap" => "text-wrap:nowrap;",
        "text-balance" => "text-wrap:balance;",
        "text-pretty" => "text-wrap:pretty;",

        // Font family
        "font-sans" => "font-family:ui-sans-serif,system-ui,sans-serif,\"Apple Color Emoji\",\"Segoe UI Emoji\",\"Segoe UI Symbol\",\"Noto Color Emoji\";",
        "font-serif" => "font-family:ui-serif,Georgia,Cambria,\"Times New Roman\",Times,serif;",
        "font-mono" => "font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,\"Liberation Mono\",\"Courier New\",monospace;",

        // List style type
        "list-none" => "list-style-type:none;",
        "list-disc" => "list-style-type:disc;",
        "list-decimal" => "list-style-type:decimal;",

        // List style position
        "list-inside" => "list-style-position:inside;",
        "list-outside" => "list-style-position:outside;",

        // Vertical align
        "align-baseline" => "vertical-align:baseline;",
        "align-top" => "vertical-align:top;",
        "align-middle" => "vertical-align:middle;",
        "align-bottom" => "vertical-align:bottom;",
        "align-text-top" => "vertical-align:text-top;",
        "align-text-bottom" => "vertical-align:text-bottom;",
        "align-sub" => "vertical-align:sub;",
        "align-super" => "vertical-align:super;",

        // Hyphens
        "hyphens-none" => "hyphens:none;",
        "hyphens-manual" => "hyphens:manual;",
        "hyphens-auto" => "hyphens:auto;",

        // Content
        "content-none" => "content:none;",

        _ => {
            return resolve_prefix(class);
        }
    };
    Some(ResolvedUtility::Standard(String::from(decls)))
}

fn resolve_prefix(class: &str) -> Option<ResolvedUtility> {
    // Text size / color
    if let Some(rest) = class.strip_prefix("text-") {
        // Text sizes
        let size_decl = match rest {
            "xs" => Some("font-size:0.75rem;line-height:1rem;"),
            "sm" => Some("font-size:0.875rem;line-height:1.25rem;"),
            "base" => Some("font-size:1rem;line-height:1.5rem;"),
            "lg" => Some("font-size:1.125rem;line-height:1.75rem;"),
            "xl" => Some("font-size:1.25rem;line-height:1.75rem;"),
            "2xl" => Some("font-size:1.5rem;line-height:2rem;"),
            "3xl" => Some("font-size:1.875rem;line-height:2.25rem;"),
            "4xl" => Some("font-size:2.25rem;line-height:2.5rem;"),
            "5xl" => Some("font-size:3rem;line-height:1;"),
            "6xl" => Some("font-size:3.75rem;line-height:1;"),
            "7xl" => Some("font-size:4.5rem;line-height:1;"),
            "8xl" => Some("font-size:6rem;line-height:1;"),
            "9xl" => Some("font-size:8rem;line-height:1;"),
            _ => None,
        };
        if let Some(d) = size_decl {
            return Some(ResolvedUtility::Standard(String::from(d)));
        }
        // Text color (with opacity support)
        if let Some(decls) = resolve_color_with_opacity(rest, "color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        // Arbitrary value: text-[#e6edf3], text-[rgb(...)], etc.
        if let Some(val) = super::parse_arbitrary(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("color:{};", val)));
        }
        return None;
    }

    // Font weight
    if let Some(rest) = class.strip_prefix("font-") {
        let decl = match rest {
            "thin" => "font-weight:100;",
            "extralight" => "font-weight:200;",
            "light" => "font-weight:300;",
            "normal" => "font-weight:400;",
            "medium" => "font-weight:500;",
            "semibold" => "font-weight:600;",
            "bold" => "font-weight:700;",
            "extrabold" => "font-weight:800;",
            "black" => "font-weight:900;",
            _ => return None,
        };
        return Some(ResolvedUtility::Standard(String::from(decl)));
    }

    // Line height
    if let Some(rest) = class.strip_prefix("leading-") {
        let decl = match rest {
            "none" => String::from("line-height:1;"),
            "tight" => String::from("line-height:1.25;"),
            "snug" => String::from("line-height:1.375;"),
            "normal" => String::from("line-height:1.5;"),
            "relaxed" => String::from("line-height:1.625;"),
            "loose" => String::from("line-height:2;"),
            _ => {
                let n = parse_u32(rest)?;
                crate::vformat!("line-height:{};", spacing(n))
            }
        };
        return Some(ResolvedUtility::Standard(decl));
    }

    // Letter spacing
    if let Some(rest) = class.strip_prefix("tracking-") {
        let decl = match rest {
            "tighter" => "letter-spacing:-0.05em;",
            "tight" => "letter-spacing:-0.025em;",
            "normal" => "letter-spacing:0em;",
            "wide" => "letter-spacing:0.025em;",
            "wider" => "letter-spacing:0.05em;",
            "widest" => "letter-spacing:0.1em;",
            _ => return None,
        };
        return Some(ResolvedUtility::Standard(String::from(decl)));
    }

    // Line clamp
    if let Some(rest) = class.strip_prefix("line-clamp-") {
        if rest == "none" {
            return Some(ResolvedUtility::Standard(String::from(
                "overflow:visible;display:block;-webkit-box-orient:horizontal;-webkit-line-clamp:none;"
            )));
        }
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!(
            "overflow:hidden;display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:{};",
            n
        )));
    }

    // Text decoration thickness
    if let Some(rest) = class.strip_prefix("decoration-") {
        // Check if it's a thickness number
        if let Some(n) = parse_u32(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("text-decoration-thickness:{}px;", n)));
        }
        // Check if it's a color
        if let Some(decls) = resolve_color_with_opacity(rest, "text-decoration-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        return None;
    }

    // Underline offset
    if let Some(rest) = class.strip_prefix("underline-offset-") {
        if rest == "auto" {
            return Some(ResolvedUtility::Standard(String::from("text-underline-offset:auto;")));
        }
        let n = parse_u32(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("text-underline-offset:{}px;", n)));
    }

    // Text indent
    if let Some(rest) = class.strip_prefix("indent-") {
        if let Some(val) = super::parse_spacing_value(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("text-indent:{};", val)));
        }
        return None;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::resolve;

    #[test]
    fn test_text_alignment() {
        assert_eq!(resolve("text-left").unwrap().as_str(), ".text-left{text-align:left;}");
        assert_eq!(resolve("text-center").unwrap().as_str(), ".text-center{text-align:center;}");
    }

    #[test]
    fn test_text_size() {
        let r = resolve("text-sm").unwrap();
        assert!(r.as_str().contains("font-size:0.875rem;"));
        assert!(r.as_str().contains("line-height:1.25rem;"));

        assert!(resolve("text-7xl").unwrap().as_str().contains("font-size:4.5rem;"));
        assert!(resolve("text-9xl").unwrap().as_str().contains("font-size:8rem;"));
    }

    #[test]
    fn test_text_color() {
        assert_eq!(resolve("text-red-500").unwrap().as_str(), ".text-red-500{color:#ef4444;}");
        assert_eq!(resolve("text-white").unwrap().as_str(), ".text-white{color:#ffffff;}");
    }

    #[test]
    fn test_text_color_opacity() {
        assert_eq!(
            resolve("text-red-500/50").unwrap().as_str(),
            ".text-red-500\\/50{color:rgb(239 68 68 / 0.5);}"
        );
    }

    #[test]
    fn test_font_weight() {
        assert_eq!(resolve("font-bold").unwrap().as_str(), ".font-bold{font-weight:700;}");
        assert_eq!(resolve("font-normal").unwrap().as_str(), ".font-normal{font-weight:400;}");
    }

    #[test]
    fn test_font_family() {
        assert!(resolve("font-sans").unwrap().as_str().contains("font-family:ui-sans-serif"));
        assert!(resolve("font-mono").unwrap().as_str().contains("font-family:ui-monospace"));
    }

    #[test]
    fn test_leading() {
        assert_eq!(resolve("leading-none").unwrap().as_str(), ".leading-none{line-height:1;}");
        assert_eq!(resolve("leading-tight").unwrap().as_str(), ".leading-tight{line-height:1.25;}");
    }

    #[test]
    fn test_tracking() {
        assert_eq!(resolve("tracking-tight").unwrap().as_str(), ".tracking-tight{letter-spacing:-0.025em;}");
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(resolve("whitespace-nowrap").unwrap().as_str(), ".whitespace-nowrap{white-space:nowrap;}");
    }

    #[test]
    fn test_word_break() {
        assert_eq!(resolve("break-all").unwrap().as_str(), ".break-all{word-break:break-all;}");
    }

    #[test]
    fn test_text_decoration() {
        assert_eq!(resolve("underline").unwrap().as_str(), ".underline{text-decoration-line:underline;}");
        assert_eq!(resolve("line-through").unwrap().as_str(), ".line-through{text-decoration-line:line-through;}");
        assert_eq!(resolve("decoration-wavy").unwrap().as_str(), ".decoration-wavy{text-decoration-style:wavy;}");
    }

    #[test]
    fn test_decoration_thickness() {
        assert_eq!(resolve("decoration-2").unwrap().as_str(), ".decoration-2{text-decoration-thickness:2px;}");
    }

    #[test]
    fn test_list_style() {
        assert_eq!(resolve("list-disc").unwrap().as_str(), ".list-disc{list-style-type:disc;}");
        assert_eq!(resolve("list-inside").unwrap().as_str(), ".list-inside{list-style-position:inside;}");
    }

    #[test]
    fn test_vertical_align() {
        assert_eq!(resolve("align-middle").unwrap().as_str(), ".align-middle{vertical-align:middle;}");
    }

    #[test]
    fn test_text_wrap() {
        assert_eq!(resolve("text-balance").unwrap().as_str(), ".text-balance{text-wrap:balance;}");
    }

    #[test]
    fn test_line_clamp() {
        let r = resolve("line-clamp-3").unwrap();
        assert!(r.as_str().contains("-webkit-line-clamp:3;"));
    }

    #[test]
    fn test_indent() {
        assert_eq!(resolve("indent-4").unwrap().as_str(), ".indent-4{text-indent:1rem;}");
    }

    #[test]
    fn test_text_arbitrary_hex() {
        assert_eq!(
            resolve("text-[#e6edf3]").unwrap().as_str(),
            ".text-\\[\\#e6edf3\\]{color:#e6edf3;}"
        );
    }
}
