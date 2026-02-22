//! Background utilities — color, gradients, size, position, repeat, attachment, clip, origin.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, resolve_color_with_opacity};
use crate::libs::web::volkistyle::palette;

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Gradient directions
        "bg-gradient-to-t" => "background-image:linear-gradient(to top,var(--tw-gradient-stops));",
        "bg-gradient-to-tr" => "background-image:linear-gradient(to top right,var(--tw-gradient-stops));",
        "bg-gradient-to-r" => "background-image:linear-gradient(to right,var(--tw-gradient-stops));",
        "bg-gradient-to-br" => "background-image:linear-gradient(to bottom right,var(--tw-gradient-stops));",
        "bg-gradient-to-b" => "background-image:linear-gradient(to bottom,var(--tw-gradient-stops));",
        "bg-gradient-to-bl" => "background-image:linear-gradient(to bottom left,var(--tw-gradient-stops));",
        "bg-gradient-to-l" => "background-image:linear-gradient(to left,var(--tw-gradient-stops));",
        "bg-gradient-to-tl" => "background-image:linear-gradient(to top left,var(--tw-gradient-stops));",
        "bg-none" => "background-image:none;",

        // Background size
        "bg-auto" => "background-size:auto;",
        "bg-cover" => "background-size:cover;",
        "bg-contain" => "background-size:contain;",

        // Background position
        "bg-center" => "background-position:center;",
        "bg-top" => "background-position:top;",
        "bg-right" => "background-position:right;",
        "bg-bottom" => "background-position:bottom;",
        "bg-left" => "background-position:left;",
        "bg-left-bottom" => "background-position:left bottom;",
        "bg-left-top" => "background-position:left top;",
        "bg-right-bottom" => "background-position:right bottom;",
        "bg-right-top" => "background-position:right top;",

        // Background repeat
        "bg-repeat" => "background-repeat:repeat;",
        "bg-no-repeat" => "background-repeat:no-repeat;",
        "bg-repeat-x" => "background-repeat:repeat-x;",
        "bg-repeat-y" => "background-repeat:repeat-y;",
        "bg-repeat-round" => "background-repeat:round;",
        "bg-repeat-space" => "background-repeat:space;",

        // Background attachment
        "bg-fixed" => "background-attachment:fixed;",
        "bg-local" => "background-attachment:local;",
        "bg-scroll" => "background-attachment:scroll;",

        // Background clip
        "bg-clip-border" => "background-clip:border-box;",
        "bg-clip-padding" => "background-clip:padding-box;",
        "bg-clip-content" => "background-clip:content-box;",
        "bg-clip-text" => "-webkit-background-clip:text;background-clip:text;",

        // Background origin
        "bg-origin-border" => "background-origin:border-box;",
        "bg-origin-padding" => "background-origin:padding-box;",
        "bg-origin-content" => "background-origin:content-box;",

        _ => {
            return resolve_prefix(class);
        }
    };
    Some(ResolvedUtility::Standard(String::from(decls)))
}

fn resolve_prefix(class: &str) -> Option<ResolvedUtility> {
    // Background color (with opacity support)
    if let Some(rest) = class.strip_prefix("bg-") {
        if let Some(decls) = resolve_color_with_opacity(rest, "background-color") {
            return Some(ResolvedUtility::Standard(decls));
        }
        // Arbitrary value: bg-[#161b22], bg-[rgb(...)], etc.
        if let Some(val) = super::parse_arbitrary(rest) {
            return Some(ResolvedUtility::Standard(crate::vformat!("background-color:{};", val)));
        }
        return None;
    }

    // Gradient from
    if let Some(rest) = class.strip_prefix("from-") {
        let hex = palette::color_hex(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!(
            "--tw-gradient-from:{} var(--tw-gradient-from-position);--tw-gradient-to:rgb(255 255 255 / 0) var(--tw-gradient-to-position);--tw-gradient-stops:var(--tw-gradient-from),var(--tw-gradient-to);",
            hex
        )));
    }

    // Gradient via
    if let Some(rest) = class.strip_prefix("via-") {
        let hex = palette::color_hex(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!(
            "--tw-gradient-to:rgb(255 255 255 / 0) var(--tw-gradient-to-position);--tw-gradient-stops:var(--tw-gradient-from),{} var(--tw-gradient-via-position),var(--tw-gradient-to);",
            hex
        )));
    }

    // Gradient to
    if let Some(rest) = class.strip_prefix("to-") {
        let hex = palette::color_hex(rest)?;
        return Some(ResolvedUtility::Standard(crate::vformat!(
            "--tw-gradient-to:{} var(--tw-gradient-to-position);",
            hex
        )));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::resolve;

    #[test]
    fn test_bg_color() {
        assert_eq!(resolve("bg-blue-500").unwrap().as_str(), ".bg-blue-500{background-color:#3b82f6;}");
    }

    #[test]
    fn test_bg_color_opacity() {
        assert_eq!(
            resolve("bg-red-500/50").unwrap().as_str(),
            ".bg-red-500\\/50{background-color:rgb(239 68 68 / 0.5);}"
        );
    }

    #[test]
    fn test_gradient_direction() {
        assert!(resolve("bg-gradient-to-r").unwrap().as_str().contains("linear-gradient(to right"));
        assert!(resolve("bg-gradient-to-br").unwrap().as_str().contains("to bottom right"));
    }

    #[test]
    fn test_gradient_from() {
        let r = resolve("from-red-500").unwrap();
        assert!(r.as_str().contains("--tw-gradient-from:#ef4444"));
    }

    #[test]
    fn test_gradient_to() {
        let r = resolve("to-blue-500").unwrap();
        assert!(r.as_str().contains("--tw-gradient-to:#3b82f6"));
    }

    #[test]
    fn test_bg_size() {
        assert_eq!(resolve("bg-cover").unwrap().as_str(), ".bg-cover{background-size:cover;}");
    }

    #[test]
    fn test_bg_position() {
        assert_eq!(resolve("bg-center").unwrap().as_str(), ".bg-center{background-position:center;}");
    }

    #[test]
    fn test_bg_repeat() {
        assert_eq!(resolve("bg-no-repeat").unwrap().as_str(), ".bg-no-repeat{background-repeat:no-repeat;}");
    }

    #[test]
    fn test_bg_attachment() {
        assert_eq!(resolve("bg-fixed").unwrap().as_str(), ".bg-fixed{background-attachment:fixed;}");
    }

    #[test]
    fn test_bg_clip() {
        assert!(resolve("bg-clip-text").unwrap().as_str().contains("background-clip:text;"));
    }

    #[test]
    fn test_bg_arbitrary_hex() {
        assert_eq!(
            resolve("bg-[#161b22]").unwrap().as_str(),
            ".bg-\\[\\#161b22\\]{background-color:#161b22;}"
        );
    }
}
