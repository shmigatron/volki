//! DOM bindings API — marker types and patterns the compiler recognizes in `-> Client` functions.
//!
//! This module defines the high-level API that users write in `.volki` files:
//!
//! ```rust,ignore
//! pub fn on_click(target: &str) -> Client {
//!     let el = dom::query("#my-button");
//!     el.set_text("clicked!");
//!     el.add_class("active");
//!     el.set_attr("disabled", "true");
//!     let val = dom::query("#input").get_value();
//!     dom::log("debug message");
//! }
//! ```
//!
//! The compiler text-transforms these calls into WASM extern function calls.
//! This module exists as documentation and for potential future use in
//! type-checking or IDE support — the actual transformation is done by
//! `wasm_codegen.rs`.

/// DOM API namespace. Functions in this namespace are recognized by the compiler
/// and transformed to WASM extern calls.
///
/// # Available functions
///
/// - `dom::query(selector)` — Query a DOM element by CSS selector. Returns a handle (i32).
/// - `dom::log(message)` — Log a message to the browser console (auto-detects string vs i32).
///
/// # Handle methods (called on the i32 handle returned by `dom::query`)
///
/// - `handle.set_text(text)` — Set the `textContent` of the element.
/// - `handle.get_value()` — Get the `value` of an input element.
/// - `handle.set_attr(name, value)` — Set an attribute on the element.
/// - `handle.add_class(class)` — Add a CSS class to the element.
/// - `handle.remove_class(class)` — Remove a CSS class from the element.
/// - `handle.toggle_class(class)` — Toggle a CSS class on the element.
/// - `handle.get_attr(name)` — Get an attribute value from the element.
/// - `handle.remove_attr(name)` — Remove an attribute from the element.
///
/// # Element creation and manipulation
///
/// - `dom::create(tag)` — Create a new DOM element. Returns a handle (i32).
/// - `dom::create_text(content)` — Create a text node. Returns a handle (i32).
/// - `dom::append(parent, child)` — Append a child element to a parent.
/// - `dom::insert_before(parent, new_child, reference)` — Insert before a reference node.
/// - `dom::remove(handle)` — Remove an element from the DOM.
/// - `dom::replace_child(parent, new_child, old_child)` — Replace a child element.
/// - `dom::clone_node(handle, deep)` — Clone a node (deep or shallow).
/// - `dom::set_html(handle, html)` — Set the innerHTML of an element.
/// - `dom::get_html(handle)` — Get the innerHTML of an element.
/// - `dom::get_outer_html(handle)` — Get the outerHTML of an element.
/// - `dom::query_all_count(selector)` — Count elements matching a CSS selector.
/// - `dom::query_all_get(selector, index)` — Get a handle to the element at index.
///
/// # Event handling
///
/// - `dom::add_event(handle, event_type, callback_id)` — Add an event listener.
/// - `dom::remove_event(handle, event_type, callback_id)` — Remove an event listener.
/// - `dom::dispatch(handle, event_type)` — Dispatch an event on an element.
pub mod dom {
    // These are marker definitions. The compiler intercepts calls to these
    // patterns and transforms them — they are never actually compiled or linked.
    // This module exists so `dom::query` and `dom::log` are valid identifiers
    // in the user's source code during development.
}

/// The set of DOM API patterns the compiler recognizes and their extern mappings.
///
/// Each entry: `(source_pattern, extern_fn_name, param_style)`
pub const DOM_API_MAP: &[(&str, &str, DomCallStyle)] = &[
    ("dom::query",           "__volki_dom_query",           DomCallStyle::StringToHandle),
    ("dom::log",             "__volki_console_log",          DomCallStyle::StringVoid),
    (".set_text(",           "__volki_dom_set_text",         DomCallStyle::HandleString),
    (".get_value(",          "__volki_dom_get_value",        DomCallStyle::HandleToString),
    (".set_attr(",           "__volki_dom_set_attr",         DomCallStyle::HandleStringString),
    (".add_class(",          "__volki_dom_add_class",        DomCallStyle::HandleString),
    (".remove_class(",       "__volki_dom_remove_class",     DomCallStyle::HandleString),
    // New DOM operations
    ("dom::create_text(",    "__volki_dom_create_text",      DomCallStyle::StringToHandle),
    ("dom::create(",         "__volki_dom_create",           DomCallStyle::StringToHandle),
    ("dom::append(",         "__volki_dom_append",           DomCallStyle::HandleHandle),
    ("dom::remove(",         "__volki_dom_remove",           DomCallStyle::HandleVoid),
    ("dom::set_html(",       "__volki_dom_set_html",         DomCallStyle::HandleString),
    (".toggle_class(",       "__volki_dom_toggle_class",     DomCallStyle::HandleString),
    (".get_attr(",           "__volki_dom_get_attr",         DomCallStyle::HandleStringToString),
    (".remove_attr(",        "__volki_dom_remove_attr",      DomCallStyle::HandleString),
    ("dom::query_all_count(","__volki_dom_query_all_count",  DomCallStyle::StringToI32),
    ("dom::query_all_get(",  "__volki_dom_query_all_get",    DomCallStyle::StringI32ToHandle),
    // Full DOM API extensions
    ("dom::insert_before(",  "__volki_dom_insert_before",    DomCallStyle::HandleHandleHandle),
    ("dom::replace_child(",  "__volki_dom_replace_child",    DomCallStyle::HandleHandleHandle),
    ("dom::clone_node(",     "__volki_dom_clone_node",       DomCallStyle::HandleI32ToHandle),
    ("dom::get_html(",       "__volki_dom_get_html",         DomCallStyle::HandleToString),
    ("dom::get_outer_html(", "__volki_dom_get_outer_html",   DomCallStyle::HandleToString),
    (".has_attr(",           "__volki_dom_has_attr",         DomCallStyle::HandleStringToI32),
    ("dom::add_event(",      "__volki_dom_add_event",        DomCallStyle::HandleStringI32),
    ("dom::remove_event(",   "__volki_dom_remove_event",     DomCallStyle::HandleStringI32),
    ("dom::dispatch(",       "__volki_dom_dispatch",         DomCallStyle::HandleString),
];

/// Describes how a DOM API call's parameters map to WASM ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomCallStyle {
    /// `dom::query(selector: &str) -> i32` — string in, handle out.
    StringToHandle,
    /// `dom::log(msg: &str)` — string in, void out.
    StringVoid,
    /// `dom::log(val: i32)` — i32 in, void out (auto-detected).
    I32Void,
    /// `handle.set_text(text: &str)` — handle + string in, void out.
    HandleString,
    /// `handle.get_value() -> &str` — handle in, string out.
    HandleToString,
    /// `handle.set_attr(name: &str, value: &str)` — handle + two strings in, void out.
    HandleStringString,
    /// `dom::append(parent, child)` — two handles in, void out.
    HandleHandle,
    /// `dom::remove(handle)` — handle in, void out.
    HandleVoid,
    /// `.get_attr(name)` — handle + string in, string out.
    HandleStringToString,
    /// `dom::query_all_count(sel)` — string in, i32 out.
    StringToI32,
    /// `dom::query_all_get(sel, idx)` — string + i32 in, handle out.
    StringI32ToHandle,
    /// `dom::insert_before(parent, child, ref)` — three handles in, void out.
    HandleHandleHandle,
    /// `dom::clone_node(handle, deep)` — handle + i32 in, handle out.
    HandleI32ToHandle,
    /// `.has_attr(name)` — handle + string in, i32 out.
    HandleStringToI32,
    /// `dom::add_event(handle, type, cb_id)` — handle + string + i32 in, void out.
    HandleStringI32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_api_map_has_all_entries() {
        assert_eq!(DOM_API_MAP.len(), 26);
    }

    #[test]
    fn test_dom_api_map_query() {
        let (pattern, extern_name, style) = DOM_API_MAP[0];
        assert_eq!(pattern, "dom::query");
        assert_eq!(extern_name, "__volki_dom_query");
        assert_eq!(style, DomCallStyle::StringToHandle);
    }

    #[test]
    fn test_dom_api_map_log() {
        let (pattern, extern_name, style) = DOM_API_MAP[1];
        assert_eq!(pattern, "dom::log");
        assert_eq!(extern_name, "__volki_console_log");
        assert_eq!(style, DomCallStyle::StringVoid);
    }

    #[test]
    fn test_dom_api_map_set_text() {
        let (pattern, extern_name, style) = DOM_API_MAP[2];
        assert_eq!(pattern, ".set_text(");
        assert_eq!(extern_name, "__volki_dom_set_text");
        assert_eq!(style, DomCallStyle::HandleString);
    }
}
