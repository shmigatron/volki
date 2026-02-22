//! Interactivity utilities — cursor, resize, scroll, touch, appearance, will-change, etc.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_spacing_value, resolve_color_with_opacity};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Cursor
        "cursor-auto" => "cursor:auto;",
        "cursor-default" => "cursor:default;",
        "cursor-pointer" => "cursor:pointer;",
        "cursor-wait" => "cursor:wait;",
        "cursor-text" => "cursor:text;",
        "cursor-move" => "cursor:move;",
        "cursor-help" => "cursor:help;",
        "cursor-not-allowed" => "cursor:not-allowed;",
        "cursor-none" => "cursor:none;",
        "cursor-context-menu" => "cursor:context-menu;",
        "cursor-progress" => "cursor:progress;",
        "cursor-cell" => "cursor:cell;",
        "cursor-crosshair" => "cursor:crosshair;",
        "cursor-vertical-text" => "cursor:vertical-text;",
        "cursor-alias" => "cursor:alias;",
        "cursor-copy" => "cursor:copy;",
        "cursor-no-drop" => "cursor:no-drop;",
        "cursor-grab" => "cursor:grab;",
        "cursor-grabbing" => "cursor:grabbing;",
        "cursor-all-scroll" => "cursor:all-scroll;",
        "cursor-col-resize" => "cursor:col-resize;",
        "cursor-row-resize" => "cursor:row-resize;",
        "cursor-n-resize" => "cursor:n-resize;",
        "cursor-e-resize" => "cursor:e-resize;",
        "cursor-s-resize" => "cursor:s-resize;",
        "cursor-w-resize" => "cursor:w-resize;",
        "cursor-ne-resize" => "cursor:ne-resize;",
        "cursor-nw-resize" => "cursor:nw-resize;",
        "cursor-se-resize" => "cursor:se-resize;",
        "cursor-sw-resize" => "cursor:sw-resize;",
        "cursor-ew-resize" => "cursor:ew-resize;",
        "cursor-ns-resize" => "cursor:ns-resize;",
        "cursor-nesw-resize" => "cursor:nesw-resize;",
        "cursor-nwse-resize" => "cursor:nwse-resize;",
        "cursor-zoom-in" => "cursor:zoom-in;",
        "cursor-zoom-out" => "cursor:zoom-out;",

        // Resize
        "resize-none" => "resize:none;",
        "resize-y" => "resize:vertical;",
        "resize-x" => "resize:horizontal;",
        "resize" => "resize:both;",

        // User select
        "select-none" => "user-select:none;",
        "select-text" => "user-select:text;",
        "select-all" => "user-select:all;",
        "select-auto" => "user-select:auto;",

        // Pointer events
        "pointer-events-none" => "pointer-events:none;",
        "pointer-events-auto" => "pointer-events:auto;",

        // Scroll behavior
        "scroll-auto" => "scroll-behavior:auto;",
        "scroll-smooth" => "scroll-behavior:smooth;",

        // Scroll snap type
        "snap-none" => "scroll-snap-type:none;",
        "snap-x" => "scroll-snap-type:x var(--tw-scroll-snap-strictness);",
        "snap-y" => "scroll-snap-type:y var(--tw-scroll-snap-strictness);",
        "snap-both" => "scroll-snap-type:both var(--tw-scroll-snap-strictness);",

        // Scroll snap strictness
        "snap-mandatory" => "--tw-scroll-snap-strictness:mandatory;",
        "snap-proximity" => "--tw-scroll-snap-strictness:proximity;",

        // Scroll snap align
        "snap-start" => "scroll-snap-align:start;",
        "snap-end" => "scroll-snap-align:end;",
        "snap-center" => "scroll-snap-align:center;",
        "snap-align-none" => "scroll-snap-align:none;",

        // Scroll snap stop
        "snap-normal" => "scroll-snap-stop:normal;",
        "snap-always" => "scroll-snap-stop:always;",

        // Touch action
        "touch-auto" => "touch-action:auto;",
        "touch-none" => "touch-action:none;",
        "touch-pan-x" => "touch-action:pan-x;",
        "touch-pan-y" => "touch-action:pan-y;",
        "touch-pan-left" => "touch-action:pan-left;",
        "touch-pan-right" => "touch-action:pan-right;",
        "touch-pan-up" => "touch-action:pan-up;",
        "touch-pan-down" => "touch-action:pan-down;",
        "touch-pinch-zoom" => "touch-action:pinch-zoom;",
        "touch-manipulation" => "touch-action:manipulation;",

        // Appearance
        "appearance-none" => "appearance:none;",
        "appearance-auto" => "appearance:auto;",

        // Will change
        "will-change-auto" => "will-change:auto;",
        "will-change-scroll" => "will-change:scroll-position;",
        "will-change-contents" => "will-change:contents;",
        "will-change-transform" => "will-change:transform;",

        _ => {
            return resolve_prefix(class);
        }
    };
    Some(ResolvedUtility::Standard(String::from(decls)))
}

fn resolve_prefix(class: &str) -> Option<ResolvedUtility> {
    // Accent color
    if let Some(rest) = class.strip_prefix("accent-") {
        if rest == "auto" {
            return Some(ResolvedUtility::Standard(String::from("accent-color:auto;")));
        }
        if let Some(decls) = resolve_color_with_opacity(rest, "accent-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        return None;
    }

    // Caret color
    if let Some(rest) = class.strip_prefix("caret-") {
        if let Some(decls) = resolve_color_with_opacity(rest, "caret-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        return None;
    }

    // Scroll margin
    if let Some(rest) = class.strip_prefix("scroll-mx-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-margin-left:{};scroll-margin-right:{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-my-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-margin-top:{};scroll-margin-bottom:{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-mt-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-margin-top:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-mr-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-margin-right:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-mb-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-margin-bottom:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-ml-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-margin-left:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-m-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-margin:{};", val)));
    }

    // Scroll padding
    if let Some(rest) = class.strip_prefix("scroll-px-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-padding-left:{};scroll-padding-right:{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-py-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-padding-top:{};scroll-padding-bottom:{};", val, val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-pt-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-padding-top:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-pr-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-padding-right:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-pb-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-padding-bottom:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-pl-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-padding-left:{};", val)));
    }
    if let Some(rest) = class.strip_prefix("scroll-p-") {
        let val = parse_spacing_value(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!("scroll-padding:{};", val)));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::resolve;

    #[test]
    fn test_cursor() {
        assert_eq!(resolve("cursor-pointer").unwrap().as_str(), ".cursor-pointer{cursor:pointer;}");
        assert_eq!(resolve("cursor-wait").unwrap().as_str(), ".cursor-wait{cursor:wait;}");
        assert_eq!(resolve("cursor-grab").unwrap().as_str(), ".cursor-grab{cursor:grab;}");
        assert_eq!(resolve("cursor-zoom-in").unwrap().as_str(), ".cursor-zoom-in{cursor:zoom-in;}");
    }

    #[test]
    fn test_resize() {
        assert_eq!(resolve("resize").unwrap().as_str(), ".resize{resize:both;}");
        assert_eq!(resolve("resize-none").unwrap().as_str(), ".resize-none{resize:none;}");
    }

    #[test]
    fn test_select() {
        assert_eq!(resolve("select-none").unwrap().as_str(), ".select-none{user-select:none;}");
    }

    #[test]
    fn test_scroll_behavior() {
        assert_eq!(resolve("scroll-smooth").unwrap().as_str(), ".scroll-smooth{scroll-behavior:smooth;}");
    }

    #[test]
    fn test_scroll_snap() {
        assert!(resolve("snap-x").unwrap().as_str().contains("scroll-snap-type:x"));
        assert_eq!(resolve("snap-start").unwrap().as_str(), ".snap-start{scroll-snap-align:start;}");
    }

    #[test]
    fn test_touch() {
        assert_eq!(resolve("touch-none").unwrap().as_str(), ".touch-none{touch-action:none;}");
        assert_eq!(resolve("touch-manipulation").unwrap().as_str(), ".touch-manipulation{touch-action:manipulation;}");
    }

    #[test]
    fn test_will_change() {
        assert_eq!(resolve("will-change-transform").unwrap().as_str(), ".will-change-transform{will-change:transform;}");
    }

    #[test]
    fn test_accent_color() {
        assert_eq!(resolve("accent-red-500").unwrap().as_str(), ".accent-red-500{accent-color:#ef4444;}");
        assert_eq!(resolve("accent-auto").unwrap().as_str(), ".accent-auto{accent-color:auto;}");
    }

    #[test]
    fn test_caret_color() {
        assert_eq!(resolve("caret-blue-500").unwrap().as_str(), ".caret-blue-500{caret-color:#3b82f6;}");
    }

    #[test]
    fn test_scroll_margin() {
        assert_eq!(resolve("scroll-m-4").unwrap().as_str(), ".scroll-m-4{scroll-margin:1rem;}");
        assert_eq!(resolve("scroll-mt-2").unwrap().as_str(), ".scroll-mt-2{scroll-margin-top:0.5rem;}");
    }

    #[test]
    fn test_scroll_padding() {
        assert_eq!(resolve("scroll-p-4").unwrap().as_str(), ".scroll-p-4{scroll-padding:1rem;}");
        assert_eq!(resolve("scroll-pl-2").unwrap().as_str(), ".scroll-pl-2{scroll-padding-left:0.5rem;}");
    }
}
