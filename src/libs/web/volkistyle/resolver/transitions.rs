//! Transition and animation utilities.

use crate::core::volkiwithstds::collections::String;
use super::{ResolvedUtility, parse_u32};

pub fn resolve(class: &str) -> Option<ResolvedUtility> {
    let decls: &str = match class {
        // Transition property
        "transition" => "transition-property:color,background-color,border-color,text-decoration-color,fill,stroke,opacity,box-shadow,transform,filter,backdrop-filter;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;",
        "transition-none" => "transition-property:none;",
        "transition-all" => "transition-property:all;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;",
        "transition-colors" => "transition-property:color,background-color,border-color,text-decoration-color,fill,stroke;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;",
        "transition-opacity" => "transition-property:opacity;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;",
        "transition-shadow" => "transition-property:box-shadow;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;",
        "transition-transform" => "transition-property:transform;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;",

        // Timing function
        "ease-linear" => "transition-timing-function:linear;",
        "ease-in" => "transition-timing-function:cubic-bezier(0.4,0,1,1);",
        "ease-out" => "transition-timing-function:cubic-bezier(0,0,0.2,1);",
        "ease-in-out" => "transition-timing-function:cubic-bezier(0.4,0,0.2,1);",

        // Animations
        "animate-none" => "animation:none;",
        "animate-spin" => "animation:spin 1s linear infinite;",
        "animate-ping" => "animation:ping 1s cubic-bezier(0,0,0.2,1) infinite;",
        "animate-pulse" => "animation:pulse 2s cubic-bezier(0.4,0,0.6,1) infinite;",
        "animate-bounce" => "animation:bounce 1s infinite;",

        _ => {
            // Duration
            if let Some(rest) = class.strip_prefix("duration-") {
                let n = parse_u32(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("transition-duration:{}ms;", n)));
            }

            // Delay
            if let Some(rest) = class.strip_prefix("delay-") {
                let n = parse_u32(rest)?;
                return Some(ResolvedUtility::Standard(crate::vformat!("transition-delay:{}ms;", n)));
            }

            return None;
        }
    };
    Some(ResolvedUtility::Standard(String::from(decls)))
}

/// Returns @keyframes definitions needed for animation utilities.
/// Call this after resolving all classes to check which keyframes are needed.
pub fn keyframes_css(classes: &[&str]) -> String {
    let mut out = String::new();
    let mut has_spin = false;
    let mut has_ping = false;
    let mut has_pulse = false;
    let mut has_bounce = false;

    for c in classes {
        match *c {
            "animate-spin" => has_spin = true,
            "animate-ping" => has_ping = true,
            "animate-pulse" => has_pulse = true,
            "animate-bounce" => has_bounce = true,
            _ => {}
        }
    }

    if has_spin {
        out.push_str("@keyframes spin{to{transform:rotate(360deg)}}");
    }
    if has_ping {
        out.push_str("@keyframes ping{75%,100%{transform:scale(2);opacity:0}}");
    }
    if has_pulse {
        out.push_str("@keyframes pulse{50%{opacity:.5}}");
    }
    if has_bounce {
        out.push_str("@keyframes bounce{0%,100%{transform:translateY(-25%);animation-timing-function:cubic-bezier(0.8,0,1,1)}50%{transform:none;animation-timing-function:cubic-bezier(0,0,0.2,1)}}");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::super::resolve;
    use super::keyframes_css;

    #[test]
    fn test_transition() {
        assert!(resolve("transition").unwrap().as_str().contains("transition-property:"));
        assert!(resolve("transition-colors").unwrap().as_str().contains("color,background-color"));
        assert_eq!(resolve("transition-none").unwrap().as_str(), ".transition-none{transition-property:none;}");
    }

    #[test]
    fn test_transition_opacity_shadow_transform() {
        assert!(resolve("transition-opacity").unwrap().as_str().contains("opacity"));
        assert!(resolve("transition-shadow").unwrap().as_str().contains("box-shadow"));
        assert!(resolve("transition-transform").unwrap().as_str().contains("transform"));
    }

    #[test]
    fn test_duration() {
        assert_eq!(resolve("duration-150").unwrap().as_str(), ".duration-150{transition-duration:150ms;}");
        assert_eq!(resolve("duration-300").unwrap().as_str(), ".duration-300{transition-duration:300ms;}");
    }

    #[test]
    fn test_delay() {
        assert_eq!(resolve("delay-100").unwrap().as_str(), ".delay-100{transition-delay:100ms;}");
        assert_eq!(resolve("delay-500").unwrap().as_str(), ".delay-500{transition-delay:500ms;}");
    }

    #[test]
    fn test_ease() {
        assert_eq!(resolve("ease-linear").unwrap().as_str(), ".ease-linear{transition-timing-function:linear;}");
        assert!(resolve("ease-in").unwrap().as_str().contains("cubic-bezier(0.4,0,1,1)"));
    }

    #[test]
    fn test_animate() {
        assert!(resolve("animate-spin").unwrap().as_str().contains("animation:spin"));
        assert!(resolve("animate-ping").unwrap().as_str().contains("animation:ping"));
        assert_eq!(resolve("animate-none").unwrap().as_str(), ".animate-none{animation:none;}");
    }

    #[test]
    fn test_keyframes() {
        let kf = keyframes_css(&["animate-spin", "animate-bounce"]);
        assert!(kf.as_str().contains("@keyframes spin"));
        assert!(kf.as_str().contains("@keyframes bounce"));
        assert!(!kf.as_str().contains("@keyframes ping"));
    }

    #[test]
    fn test_keyframes_empty() {
        let kf = keyframes_css(&["flex", "p-4"]);
        assert!(kf.as_str().is_empty());
    }
}
