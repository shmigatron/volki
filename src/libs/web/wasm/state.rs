//! State bindings API — marker types and patterns the compiler recognizes in `-> Component` functions.
//!
//! This module defines the high-level API for stateful components:
//!
//! ```rust,ignore
//! // Inside a Component function — slot-based state
//! pub fn counter() -> Component {
//!     let count = use_state(0_i32);        // slot 0
//!     let active = use_state(false);       // slot 1
//!     let el = dom::query("#count");
//!     el.set_text(state::fmt_i32(count));
//! }
//!
//! // Inside a Client function — cross-component state access
//! pub fn on_increment() -> Client {
//!     let count = state::get_i32("counter", 0);
//!     state::set_i32("counter", 0, count + 1);
//! }
//! ```
//!
//! The compiler text-transforms these calls into WASM extern function calls.
//! This module exists as documentation and for potential future use in
//! type-checking or IDE support — the actual transformation is done by
//! `wasm_codegen.rs`.

/// State API namespace. Functions in this namespace are recognized by the compiler
/// and transformed to WASM extern calls.
///
/// # Component-internal (use_state)
///
/// - `use_state(initial)` — Initialize a state slot, returns the current value.
///   Type is inferred from the suffix: `0_i32`, `0.0_f32`, `true`/`false`.
///
/// # Cross-function access (inside Client functions)
///
/// - `state::get_i32(component, slot)` — Read an i32 state slot from a component.
/// - `state::set_i32(component, slot, value)` — Write an i32 state slot (triggers rerender).
/// - `state::get_f32(component, slot)` — Read an f32 state slot from a component.
/// - `state::set_f32(component, slot, value)` — Write an f32 state slot (triggers rerender).
///
/// # Formatting helpers (inside Component functions)
///
/// - `state::fmt_i32(value)` — Format an i32 as a string (for DOM text).
/// - `state::fmt_f32(value)` — Format an f32 as a string (for DOM text).
pub mod state {
    // These are marker definitions. The compiler intercepts calls to these
    // patterns and transforms them — they are never actually compiled or linked.
    // This module exists so `state::get_i32`, `state::set_i32`, `state::fmt_i32`
    // etc. are valid identifiers in the user's source code during development.
}

/// The set of state API patterns the compiler recognizes and their extern mappings.
///
/// Each entry: `(source_pattern, extern_fn_name, call_style)`
pub const STATE_API_MAP: &[(&str, &str, StateCallStyle)] = &[
    // Component-internal: use_state
    ("use_state(",          "__volki_state_init",        StateCallStyle::InitSlot),
    // Cross-function access (i32/f32)
    ("state::get_i32(",     "__volki_xstate_get_i32",    StateCallStyle::CrossGet),
    ("state::set_i32(",     "__volki_xstate_set_i32",    StateCallStyle::CrossSet),
    ("state::get_f32(",     "__volki_xstate_get_f32",    StateCallStyle::CrossGet),
    ("state::set_f32(",     "__volki_xstate_set_f32",    StateCallStyle::CrossSet),
    // Cross-function access (string)
    ("state::get_str(",     "__volki_xstate_get_str",    StateCallStyle::CrossGetStr),
    ("state::set_str(",     "__volki_xstate_set_str",    StateCallStyle::CrossSetStr),
    // Formatting helpers
    ("state::fmt_i32(",     "__volki_state_fmt_i32",     StateCallStyle::FmtToStr),
    ("state::fmt_f32(",     "__volki_state_fmt_f32",     StateCallStyle::FmtToStr),
];

/// Describes how a state API call's parameters map to WASM ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateCallStyle {
    /// `use_state(initial)` — initializes a slot, returns current value.
    /// Compiler assigns slot index and infers type from the argument suffix.
    InitSlot,
    /// `state::get_i32("component", slot)` — reads from another component's state.
    /// Compiler resolves component name to ID at compile time.
    CrossGet,
    /// `state::set_i32("component", slot, value)` — writes to another component's state.
    /// Triggers rerender of the target component.
    CrossSet,
    /// `state::fmt_i32(value)` — formats a number to a string for DOM display.
    /// Allocates a buffer in the WASM bump allocator, JS writes digits into it.
    FmtToStr,
    /// `state::get_str("component", slot)` — reads a string state slot.
    /// Uses two-call pattern: ptr then len.
    CrossGetStr,
    /// `state::set_str("component", slot, value)` — writes a string state slot.
    /// Triggers rerender of the target component.
    CrossSetStr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_api_map_has_all_entries() {
        assert_eq!(STATE_API_MAP.len(), 9);
    }

    #[test]
    fn test_state_api_map_use_state() {
        let (pattern, extern_name, style) = STATE_API_MAP[0];
        assert_eq!(pattern, "use_state(");
        assert_eq!(extern_name, "__volki_state_init");
        assert_eq!(style, StateCallStyle::InitSlot);
    }

    #[test]
    fn test_state_api_map_get_i32() {
        let (pattern, extern_name, style) = STATE_API_MAP[1];
        assert_eq!(pattern, "state::get_i32(");
        assert_eq!(extern_name, "__volki_xstate_get_i32");
        assert_eq!(style, StateCallStyle::CrossGet);
    }

    #[test]
    fn test_state_api_map_set_i32() {
        let (pattern, extern_name, style) = STATE_API_MAP[2];
        assert_eq!(pattern, "state::set_i32(");
        assert_eq!(extern_name, "__volki_xstate_set_i32");
        assert_eq!(style, StateCallStyle::CrossSet);
    }

    #[test]
    fn test_state_api_map_fmt_i32() {
        let (pattern, extern_name, style) = STATE_API_MAP[7];
        assert_eq!(pattern, "state::fmt_i32(");
        assert_eq!(extern_name, "__volki_state_fmt_i32");
        assert_eq!(style, StateCallStyle::FmtToStr);
    }

    #[test]
    fn test_state_api_map_f32_variants() {
        let (pattern_get, _, style_get) = STATE_API_MAP[3];
        assert_eq!(pattern_get, "state::get_f32(");
        assert_eq!(style_get, StateCallStyle::CrossGet);

        let (pattern_set, _, style_set) = STATE_API_MAP[4];
        assert_eq!(pattern_set, "state::set_f32(");
        assert_eq!(style_set, StateCallStyle::CrossSet);

        let (pattern_fmt, _, style_fmt) = STATE_API_MAP[8];
        assert_eq!(pattern_fmt, "state::fmt_f32(");
        assert_eq!(style_fmt, StateCallStyle::FmtToStr);
    }

    #[test]
    fn test_state_api_map_str_variants() {
        let (pattern_get, extern_get, style_get) = STATE_API_MAP[5];
        assert_eq!(pattern_get, "state::get_str(");
        assert_eq!(extern_get, "__volki_xstate_get_str");
        assert_eq!(style_get, StateCallStyle::CrossGetStr);

        let (pattern_set, extern_set, style_set) = STATE_API_MAP[6];
        assert_eq!(pattern_set, "state::set_str(");
        assert_eq!(extern_set, "__volki_xstate_set_str");
        assert_eq!(style_set, StateCallStyle::CrossSetStr);
    }
}
