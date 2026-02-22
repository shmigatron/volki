//! Ref bindings API — mutable references that do NOT trigger rerender.
//!
//! ```rust,ignore
//! pub fn counter() -> Component {
//!     let my_ref = use_ref(0_i32);            // mutable ref slot, NO rerender
//!     ref::set_i32(my_ref, 42);               // set value
//!     let val = ref::get_i32(my_ref);          // get value
//!     let el = use_ref_el("#my-element");      // cached DOM handle
//! }
//! ```

/// Describes how a ref API call maps to WASM ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefCallStyle {
    /// `use_ref(initial)` — initialize a ref slot, returns current value.
    InitSlot,
    /// `use_ref_el("selector")` — initialize a ref slot with a cached DOM element handle.
    InitEl,
    /// `ref::get_i32(slot)` / `ref::get_f32(slot)` — read a ref slot.
    Get,
    /// `ref::set_i32(slot, value)` / `ref::set_f32(slot, value)` — write a ref slot (no rerender).
    Set,
}

/// The set of ref API patterns the compiler recognizes and their extern mappings.
///
/// Each entry: `(source_pattern, extern_fn_name, call_style)`
pub const REF_API_MAP: &[(&str, &str, RefCallStyle)] = &[
    ("use_ref(",       "__volki_ref_init",    RefCallStyle::InitSlot),
    ("use_ref_el(",    "__volki_ref_init_el", RefCallStyle::InitEl),
    ("ref::get_i32(",  "__volki_ref_get_i32", RefCallStyle::Get),
    ("ref::set_i32(",  "__volki_ref_set_i32", RefCallStyle::Set),
    ("ref::get_f32(",  "__volki_ref_get_f32", RefCallStyle::Get),
    ("ref::set_f32(",  "__volki_ref_set_f32", RefCallStyle::Set),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_api_map_has_all_entries() {
        assert_eq!(REF_API_MAP.len(), 6);
    }

    #[test]
    fn test_ref_api_map_use_ref() {
        let (pattern, extern_name, style) = REF_API_MAP[0];
        assert_eq!(pattern, "use_ref(");
        assert_eq!(extern_name, "__volki_ref_init");
        assert_eq!(style, RefCallStyle::InitSlot);
    }

    #[test]
    fn test_ref_api_map_use_ref_el() {
        let (pattern, extern_name, style) = REF_API_MAP[1];
        assert_eq!(pattern, "use_ref_el(");
        assert_eq!(extern_name, "__volki_ref_init_el");
        assert_eq!(style, RefCallStyle::InitEl);
    }

    #[test]
    fn test_ref_api_map_get_set() {
        let (_, _, get_style) = REF_API_MAP[2];
        assert_eq!(get_style, RefCallStyle::Get);
        let (_, _, set_style) = REF_API_MAP[3];
        assert_eq!(set_style, RefCallStyle::Set);
    }
}
