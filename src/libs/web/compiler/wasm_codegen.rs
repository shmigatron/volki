//! WASM Code Generator — produces standalone `no_std` Rust files targeting `wasm32-unknown-unknown`.
//!
//! For each `-> Client` function, generates:
//! - `#![no_std]` preamble with a bump allocator
//! - `extern "C"` import block for DOM bindings
//! - `#[no_mangle] pub extern "C"` exported wrappers with flattened WASM ABI types

use crate::core::volkiwithstds::collections::{String, Vec};
use super::parser::RsxNode;
use super::scanner::{self, RsxFunction};
use super::wasm_rsx_codegen;
use crate::libs::web::wasm::types::{WasmAbi, rust_type_to_wasm, wasm_type_str};

#[derive(Clone, Copy, PartialEq, Eq)]
enum StateHelperKind {
    I32,
    F32,
}

struct StateHelperBinding {
    getter: String,
    setter: String,
    comp_id: u32,
    slot: u32,
    kind: StateHelperKind,
}

/// Generate a complete WASM-targeted Rust module from a set of Client and Component functions.
///
/// `client_fns` — only the Client-type functions from the scan.
/// `component_fns` — only the Component-type functions from the scan.
/// `source` — the full original source (used to extract function bodies).
pub fn generate_wasm_module(
    client_fns: &[&RsxFunction],
    component_fns: &[&RsxFunction],
    source: &str,
    component_rsx: &[Option<Vec<RsxNode>>],
) -> String {
    let mut out = String::with_capacity(4096);

    // Build component ID mapping: name → id (0, 1, 2...)
    let component_ids: Vec<(String, u32)> = component_fns.iter().enumerate()
        .filter_map(|(i, f)| {
            f.name.as_ref().map(|n| (n.clone(), i as u32))
        })
        .collect();
    let state_helpers = collect_state_helper_bindings(component_fns, source, &component_ids);

    // no_std preamble
    out.push_str("#![no_std]\n");
    out.push_str("#![no_main]\n\n");

    // Panic handler
    out.push_str("#[panic_handler]\n");
    out.push_str("fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }\n\n");

    // Bump allocator for string passing
    out.push_str("static mut HEAP: [u8; 65536] = [0u8; 65536];\n");
    out.push_str("static mut HEAP_PTR: usize = 0;\n\n");

    out.push_str("#[unsafe(no_mangle)]\n");
    out.push_str("pub extern \"C\" fn __volki_alloc(size: i32) -> i32 {\n");
    out.push_str("    unsafe {\n");
    out.push_str("        let ptr = core::ptr::addr_of!(HEAP_PTR).read();\n");
    out.push_str("        let new_ptr = ptr + size as usize;\n");
    out.push_str("        if new_ptr > 65536 { return 0; }\n");
    out.push_str("        core::ptr::addr_of_mut!(HEAP_PTR).write(new_ptr);\n");
    out.push_str("        core::ptr::addr_of_mut!(HEAP).cast::<u8>().add(ptr) as i32\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out.push_str("#[unsafe(no_mangle)]\n");
    out.push_str("pub extern \"C\" fn __volki_dealloc() {\n");
    out.push_str("    unsafe { core::ptr::addr_of_mut!(HEAP_PTR).write(0); }\n");
    out.push_str("}\n\n");

    let all_fns: Vec<&RsxFunction> = client_fns.iter().chain(component_fns.iter()).copied().collect();

    // Collect which DOM imports are needed
    let mut needs_query = false;
    let mut needs_set_text = false;
    let mut needs_get_value = false;
    let mut needs_set_attr = false;
    let mut needs_add_class = false;
    let mut needs_remove_class = false;
    let mut needs_log = false;

    // Collect which state imports are needed
    let needs_component_lifecycle = !component_fns.is_empty();
    let mut needs_state_init_i32 = false;
    let mut needs_state_init_f32 = false;
    let mut needs_state_init_str = false;
    let mut needs_xstate_get_i32 = false;
    let mut needs_xstate_set_i32 = false;
    let mut needs_xstate_get_f32 = false;
    let mut needs_xstate_set_f32 = false;
    let mut needs_xstate_get_str = false;
    let mut needs_xstate_set_str = false;
    let mut needs_fmt_i32 = false;
    let mut needs_fmt_f32 = false;

    // Effect imports
    let mut needs_effect = false;

    // Memo imports
    let mut needs_memo_i32 = false;
    let mut needs_memo_f32 = false;

    // Ref imports
    let mut needs_ref_init_i32 = false;
    let mut needs_ref_init_f32 = false;
    let mut needs_ref_init_el = false;
    let mut needs_ref_get_i32 = false;
    let mut needs_ref_set_i32 = false;
    let mut needs_ref_get_f32 = false;
    let mut needs_ref_set_f32 = false;

    // New DOM operations
    let mut needs_create = false;
    let mut needs_append = false;
    let mut needs_remove = false;
    let mut needs_set_html = false;
    let mut needs_toggle_class = false;
    let mut needs_get_attr = false;
    let mut needs_remove_attr = false;
    let mut needs_query_all_count = false;
    let mut needs_query_all_get = false;

    // RSX-specific externs
    let mut needs_create_text = false;
    let mut needs_is_mounted = false;
    let mut needs_mount_point = false;

    // Also collect user-declared extern blocks from function bodies
    let mut user_externs: Vec<String> = Vec::new();

    for func in &all_fns {
        let body = &source[func.body_span.0..func.body_span.1];
        // dom::query but not dom::query_all_count or dom::query_all_get
        if body.contains("dom::query(") { needs_query = true; }
        if body.contains(".set_text(") { needs_set_text = true; }
        if body.contains(".get_value(") { needs_get_value = true; }
        if body.contains(".set_attr(") { needs_set_attr = true; }
        if body.contains(".add_class(") { needs_add_class = true; }
        if body.contains(".remove_class(") { needs_remove_class = true; }
        if body.contains("dom::log") { needs_log = true; }

        // New DOM operations
        if body.contains("dom::create(") { needs_create = true; }
        if body.contains("dom::append(") { needs_append = true; }
        if body.contains("dom::remove(") { needs_remove = true; }
        if body.contains("dom::set_html(") { needs_set_html = true; }
        if body.contains(".toggle_class(") { needs_toggle_class = true; }
        if body.contains(".get_attr(") { needs_get_attr = true; }
        if body.contains(".remove_attr(") { needs_remove_attr = true; }
        if body.contains("dom::query_all_count(") { needs_query_all_count = true; }
        if body.contains("dom::query_all_get(") { needs_query_all_get = true; }

        // State imports
        if body.contains("use_state(") {
            // Check for string state: use_state("...")
            if body.contains("use_state(\"") {
                needs_state_init_str = true;
            }
            // Infer type from argument
            if body.contains("_i32") || body.contains("true") || body.contains("false") {
                needs_state_init_i32 = true;
            }
            if body.contains("_f32") {
                needs_state_init_f32 = true;
            }
            // Default to i32 if no clear suffix and no string
            if !body.contains("_f32") && !body.contains("use_state(\"") {
                needs_state_init_i32 = true;
            }
        }
        if body.contains("state::get_i32(") { needs_xstate_get_i32 = true; }
        if body.contains("state::set_i32(") { needs_xstate_set_i32 = true; }
        if body.contains("state::get_f32(") { needs_xstate_get_f32 = true; }
        if body.contains("state::set_f32(") { needs_xstate_set_f32 = true; }
        if body.contains("state::get_str(") { needs_xstate_get_str = true; }
        if body.contains("state::set_str(") { needs_xstate_set_str = true; }
        if body.contains("state::fmt_i32(") { needs_fmt_i32 = true; }
        if body.contains("state::fmt_f32(") { needs_fmt_f32 = true; }

        // Ref imports
        if body.contains("use_ref(") && !body.contains("use_ref_el(") {
            if body.contains("_i32") { needs_ref_init_i32 = true; }
            if body.contains("_f32") { needs_ref_init_f32 = true; }
            if !body.contains("_f32") { needs_ref_init_i32 = true; }
        }
        if body.contains("use_ref_el(") { needs_ref_init_el = true; }
        if body.contains("ref::get_i32(") { needs_ref_get_i32 = true; }
        if body.contains("ref::set_i32(") { needs_ref_set_i32 = true; }
        if body.contains("ref::get_f32(") { needs_ref_get_f32 = true; }
        if body.contains("ref::set_f32(") { needs_ref_set_f32 = true; }

        // Effect imports
        if body.contains("use_effect(") { needs_effect = true; }

        // Memo imports
        if body.contains("use_memo_i32(") { needs_memo_i32 = true; }
        if body.contains("use_memo_f32(") { needs_memo_f32 = true; }

        // Extract user extern "C" blocks
        extract_user_externs(body, &mut user_externs);
    }

    // Generated state helpers rely on cross-function state imports.
    if !state_helpers.is_empty() {
        needs_xstate_get_i32 = true;
        needs_xstate_set_i32 = true;
        for h in &state_helpers {
            if h.kind == StateHelperKind::F32 {
                needs_xstate_get_f32 = true;
                needs_xstate_set_f32 = true;
            }
        }
    }

    // Pre-generate RSX output for components that have RSX, and merge needs flags
    let mut rsx_outputs: Vec<Option<wasm_rsx_codegen::WasmRsxOutput>> = Vec::new();
    for (i, func) in component_fns.iter().enumerate() {
        let rsx_nodes = component_rsx.get(i).and_then(|o| o.as_ref());
        if let Some(nodes) = rsx_nodes {
            // Count user use_ref calls in the logic section to get ref_slot_offset
            let ref_slot_offset = count_user_refs(func, source);
            let rsx_out = wasm_rsx_codegen::generate_component_rsx(nodes, i as u32, ref_slot_offset);
            // Merge needs flags
            if rsx_out.needs_create { needs_create = true; }
            if rsx_out.needs_create_text { needs_create_text = true; }
            if rsx_out.needs_append { needs_append = true; }
            if rsx_out.needs_add_class { needs_add_class = true; }
            if rsx_out.needs_set_attr { needs_set_attr = true; }
            if rsx_out.needs_set_text { needs_set_text = true; }
            if rsx_out.needs_mount_point { needs_mount_point = true; }
            if rsx_out.needs_is_mounted { needs_is_mounted = true; }
            if rsx_out.needs_ref_get_i32 { needs_ref_get_i32 = true; }
            if rsx_out.needs_ref_set_i32 { needs_ref_set_i32 = true; }
            if rsx_out.needs_fmt_i32 { needs_fmt_i32 = true; }
            if rsx_out.needs_fmt_f32 { needs_fmt_f32 = true; }
            rsx_outputs.push(Some(rsx_out));
        } else {
            rsx_outputs.push(None);
        }
    }

    // Generate extern imports block
    out.push_str("unsafe extern \"C\" {\n");

    // Component lifecycle
    if needs_component_lifecycle {
        out.push_str("    fn __volki_component_begin(id: i32);\n");
        out.push_str("    fn __volki_component_end();\n");
    }

    // RSX component externs
    if needs_is_mounted {
        out.push_str("    fn __volki_component_is_mounted(id: i32) -> i32;\n");
    }
    if needs_mount_point {
        out.push_str("    fn __volki_component_mount_point(id: i32) -> i32;\n");
    }

    // State init
    if needs_state_init_i32 {
        out.push_str("    fn __volki_state_init_i32(slot: i32, initial: i32) -> i32;\n");
    }
    if needs_state_init_f32 {
        out.push_str("    fn __volki_state_init_f32(slot: i32, initial: f32) -> f32;\n");
    }
    if needs_state_init_str {
        out.push_str("    fn __volki_state_init_str(slot: i32, ptr: i32, len: i32) -> i32;\n");
        out.push_str("    fn __volki_state_init_str_len(slot: i32) -> i32;\n");
    }

    // Cross-function state access
    if needs_xstate_get_i32 {
        out.push_str("    fn __volki_xstate_get_i32(comp_id: i32, slot: i32) -> i32;\n");
    }
    if needs_xstate_set_i32 {
        out.push_str("    fn __volki_xstate_set_i32(comp_id: i32, slot: i32, value: i32);\n");
    }
    if needs_xstate_get_f32 {
        out.push_str("    fn __volki_xstate_get_f32(comp_id: i32, slot: i32) -> f32;\n");
    }
    if needs_xstate_set_f32 {
        out.push_str("    fn __volki_xstate_set_f32(comp_id: i32, slot: i32, value: f32);\n");
    }
    if needs_xstate_get_str {
        out.push_str("    fn __volki_xstate_get_str(comp_id: i32, slot: i32) -> i32;\n");
        out.push_str("    fn __volki_xstate_get_str_len(comp_id: i32, slot: i32) -> i32;\n");
    }
    if needs_xstate_set_str {
        out.push_str("    fn __volki_xstate_set_str(comp_id: i32, slot: i32, ptr: i32, len: i32);\n");
    }

    // Formatting
    if needs_fmt_i32 {
        out.push_str("    fn __volki_state_fmt_i32(value: i32, buf_ptr: i32, buf_len: i32) -> i32;\n");
    }
    if needs_fmt_f32 {
        out.push_str("    fn __volki_state_fmt_f32(value: f32, buf_ptr: i32, buf_len: i32) -> i32;\n");
    }

    // Effect imports
    if needs_effect {
        out.push_str("    fn __volki_effect_register(slot: i32, dep_count: i32);\n");
        out.push_str("    fn __volki_effect_set_dep(slot: i32, dep_idx: i32, value: i32);\n");
    }

    // Memo imports
    if needs_memo_i32 || needs_memo_f32 {
        out.push_str("    fn __volki_memo_begin(slot: i32, dep_count: i32);\n");
        out.push_str("    fn __volki_memo_set_dep(slot: i32, dep_idx: i32, value: i32);\n");
        out.push_str("    fn __volki_memo_changed(slot: i32) -> i32;\n");
    }
    if needs_memo_i32 {
        out.push_str("    fn __volki_memo_store_i32(slot: i32, value: i32);\n");
        out.push_str("    fn __volki_memo_load_i32(slot: i32) -> i32;\n");
    }
    if needs_memo_f32 {
        out.push_str("    fn __volki_memo_store_f32(slot: i32, value: f32);\n");
        out.push_str("    fn __volki_memo_load_f32(slot: i32) -> f32;\n");
    }

    // Ref imports
    if needs_ref_init_i32 {
        out.push_str("    fn __volki_ref_init_i32(slot: i32, initial: i32) -> i32;\n");
    }
    if needs_ref_init_f32 {
        out.push_str("    fn __volki_ref_init_f32(slot: i32, initial: f32) -> f32;\n");
    }
    if needs_ref_init_el {
        out.push_str("    fn __volki_ref_init_el(slot: i32, sel_ptr: i32, sel_len: i32) -> i32;\n");
    }
    if needs_ref_get_i32 {
        out.push_str("    fn __volki_ref_get_i32(slot: i32) -> i32;\n");
    }
    if needs_ref_set_i32 {
        out.push_str("    fn __volki_ref_set_i32(slot: i32, value: i32);\n");
    }
    if needs_ref_get_f32 {
        out.push_str("    fn __volki_ref_get_f32(slot: i32) -> f32;\n");
    }
    if needs_ref_set_f32 {
        out.push_str("    fn __volki_ref_set_f32(slot: i32, value: f32);\n");
    }

    // DOM imports
    if needs_query {
        out.push_str("    fn __volki_dom_query(sel_ptr: i32, sel_len: i32) -> i32;\n");
    }
    if needs_set_text {
        out.push_str("    fn __volki_dom_set_text(handle: i32, text_ptr: i32, text_len: i32);\n");
    }
    if needs_get_value {
        out.push_str("    fn __volki_dom_get_value(handle: i32) -> i32;\n");
        out.push_str("    fn __volki_dom_get_value_len(handle: i32) -> i32;\n");
    }
    if needs_set_attr {
        out.push_str("    fn __volki_dom_set_attr(handle: i32, name_ptr: i32, name_len: i32, val_ptr: i32, val_len: i32);\n");
    }
    if needs_add_class {
        out.push_str("    fn __volki_dom_add_class(handle: i32, cls_ptr: i32, cls_len: i32);\n");
    }
    if needs_remove_class {
        out.push_str("    fn __volki_dom_remove_class(handle: i32, cls_ptr: i32, cls_len: i32);\n");
    }
    if needs_log {
        out.push_str("    fn __volki_console_log(msg_ptr: i32, msg_len: i32);\n");
        out.push_str("    fn __volki_console_log_i32(value: i32);\n");
    }
    // New DOM operations
    if needs_create {
        out.push_str("    fn __volki_dom_create(tag_ptr: i32, tag_len: i32) -> i32;\n");
    }
    if needs_create_text {
        out.push_str("    fn __volki_dom_create_text(text_ptr: i32, text_len: i32) -> i32;\n");
    }
    if needs_append {
        out.push_str("    fn __volki_dom_append(parent: i32, child: i32);\n");
    }
    if needs_remove {
        out.push_str("    fn __volki_dom_remove(handle: i32);\n");
    }
    if needs_set_html {
        out.push_str("    fn __volki_dom_set_html(handle: i32, html_ptr: i32, html_len: i32);\n");
    }
    if needs_toggle_class {
        out.push_str("    fn __volki_dom_toggle_class(handle: i32, cls_ptr: i32, cls_len: i32);\n");
    }
    if needs_get_attr {
        out.push_str("    fn __volki_dom_get_attr(handle: i32, name_ptr: i32, name_len: i32) -> i32;\n");
        out.push_str("    fn __volki_dom_get_attr_len(handle: i32, name_ptr: i32, name_len: i32) -> i32;\n");
    }
    if needs_remove_attr {
        out.push_str("    fn __volki_dom_remove_attr(handle: i32, name_ptr: i32, name_len: i32);\n");
    }
    if needs_query_all_count {
        out.push_str("    fn __volki_dom_query_all_count(sel_ptr: i32, sel_len: i32) -> i32;\n");
    }
    if needs_query_all_get {
        out.push_str("    fn __volki_dom_query_all_get(sel_ptr: i32, sel_len: i32, idx: i32) -> i32;\n");
    }
    // Append user extern declarations
    for ext in &user_externs {
        out.push_str("    ");
        out.push_str(ext.as_str());
        out.push('\n');
    }
    out.push_str("}\n\n");

    if needs_log {
        out.push_str("fn __volki_log_str(msg: &str) {\n");
        out.push_str("    unsafe { __volki_console_log(msg.as_ptr() as i32, msg.len() as i32); }\n");
        out.push_str("}\n\n");
        out.push_str("fn __volki_log_i32(val: i32) {\n");
        out.push_str("    unsafe { __volki_console_log_i32(val); }\n");
        out.push_str("}\n\n");
    }

    // Generated helper fns for tuple use_state declarations.
    for helper in &state_helpers {
        match helper.kind {
            StateHelperKind::I32 => {
                out.push_str("fn ");
                out.push_str(helper.getter.as_str());
                out.push_str("() -> i32 {\n");
                out.push_str("    unsafe { __volki_xstate_get_i32(");
                out.push_str(crate::vformat!("{}", helper.comp_id).as_str());
                out.push_str(", ");
                out.push_str(crate::vformat!("{}", helper.slot).as_str());
                out.push_str(") }\n");
                out.push_str("}\n\n");

                out.push_str("fn ");
                out.push_str(helper.setter.as_str());
                out.push_str("(value: i32) {\n");
                out.push_str("    unsafe { __volki_xstate_set_i32(");
                out.push_str(crate::vformat!("{}", helper.comp_id).as_str());
                out.push_str(", ");
                out.push_str(crate::vformat!("{}", helper.slot).as_str());
                out.push_str(", value); }\n");
                out.push_str("}\n\n");
            }
            StateHelperKind::F32 => {
                out.push_str("fn ");
                out.push_str(helper.getter.as_str());
                out.push_str("() -> f32 {\n");
                out.push_str("    unsafe { __volki_xstate_get_f32(");
                out.push_str(crate::vformat!("{}", helper.comp_id).as_str());
                out.push_str(", ");
                out.push_str(crate::vformat!("{}", helper.slot).as_str());
                out.push_str(") }\n");
                out.push_str("}\n\n");

                out.push_str("fn ");
                out.push_str(helper.setter.as_str());
                out.push_str("(value: f32) {\n");
                out.push_str("    unsafe { __volki_xstate_set_f32(");
                out.push_str(crate::vformat!("{}", helper.comp_id).as_str());
                out.push_str(", ");
                out.push_str(crate::vformat!("{}", helper.slot).as_str());
                out.push_str(", value); }\n");
                out.push_str("}\n\n");
            }
        }
    }

    // Generate Component functions
    for (i, func) in component_fns.iter().enumerate() {
        let rsx_out = if i < rsx_outputs.len() {
            rsx_outputs[i].as_ref()
        } else {
            None
        };
        generate_component_fn(func, source, i as u32, rsx_out, &mut out);
    }

    // Generate Client functions
    for func in client_fns {
        generate_client_fn(func, source, &component_ids, &mut out);
    }

    out
}

/// Generate a single `#[no_mangle] pub extern "C"` Client function.
fn generate_client_fn(
    func: &RsxFunction,
    source: &str,
    component_ids: &[(String, u32)],
    out: &mut String,
) {
    let name = match &func.name {
        Some(n) => n.as_str(),
        None => return,
    };

    out.push_str("#[unsafe(no_mangle)]\n");
    out.push_str("pub extern \"C\" fn ");
    out.push_str(name);
    out.push('(');

    // Flatten params to WASM ABI
    let mut first = true;
    for param in &func.params {
        let abi = rust_type_to_wasm(param.ty.as_str());
        if !first { out.push_str(", "); }
        first = false;

        match abi {
            WasmAbi::StringPair => {
                out.push_str(param.name.as_str());
                out.push_str("_ptr: i32, ");
                out.push_str(param.name.as_str());
                out.push_str("_len: i32");
            }
            WasmAbi::Direct(wt) => {
                out.push_str(param.name.as_str());
                out.push_str(": ");
                out.push_str(wasm_type_str(wt));
            }
            WasmAbi::Void => {}
        }
    }

    out.push_str(") {\n");

    // Type reconstruction preamble for string params
    for param in &func.params {
        let abi = rust_type_to_wasm(param.ty.as_str());
        if abi == WasmAbi::StringPair {
            out.push_str("    let ");
            out.push_str(param.name.as_str());
            out.push_str(" = unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(");
            out.push_str(param.name.as_str());
            out.push_str("_ptr as *const u8, ");
            out.push_str(param.name.as_str());
            out.push_str("_len as usize)) };\n");
        }
    }

    // Transform and emit the function body
    let body = &source[func.body_span.0..func.body_span.1];
    let transformed = transform_client_body(body, component_ids);
    out.push_str("    unsafe {\n");
    for line in transformed.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        // Skip user extern "C" blocks (already hoisted)
        if trimmed.starts_with("extern ") { continue; }
        out.push_str("        ");
        out.push_str(trimmed);
        out.push('\n');
    }
    out.push_str("    }\n");
    out.push_str("}\n\n");
}

/// Generate a single `#[no_mangle] pub extern "C"` Component function.
///
/// Components export as `__volki_component_<name>()` with no parameters.
/// The body is wrapped with `__volki_component_begin(id)` / `__volki_component_end()`.
///
/// If `rsx_output` is Some, this is an RSX component with mount/update phases.
/// Otherwise it's the old-style imperative component.
fn generate_component_fn(
    func: &RsxFunction,
    source: &str,
    component_id: u32,
    rsx_output: Option<&wasm_rsx_codegen::WasmRsxOutput>,
    out: &mut String,
) {
    let name = match &func.name {
        Some(n) => n.as_str(),
        None => return,
    };

    out.push_str("#[unsafe(no_mangle)]\n");
    out.push_str("pub extern \"C\" fn __volki_component_");
    out.push_str(name);
    out.push_str("() {\n");

    out.push_str("    unsafe {\n");
    out.push_str("        __volki_component_begin(");
    out.push_str(crate::vformat!("{}", component_id).as_str());
    out.push_str(");\n");

    if let Some(rsx_out) = rsx_output {
        // RSX path: split body into logic + RSX
        let split = scanner::split_component_body(source, func.body_span);
        let logic_body = if let Some(ref s) = split {
            &source[s.logic_span.0..s.logic_span.1]
        } else {
            // Shouldn't happen if rsx_output is Some, but fallback
            &source[func.body_span.0..func.body_span.1]
        };

        // Transform and emit the logic section
        let transformed_logic = transform_component_body(logic_body, component_id);
        for line in transformed_logic.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if trimmed.starts_with("extern ") { continue; }
            out.push_str("        ");
            out.push_str(trimmed);
            out.push('\n');
        }

        // Mount phase: runs only on first render
        out.push_str("        if __volki_component_is_mounted(");
        out.push_str(crate::vformat!("{}", component_id).as_str());
        out.push_str(") == 0 {\n");
        for line in rsx_out.mount_code.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            out.push_str("            ");
            out.push_str(trimmed);
            out.push('\n');
        }
        out.push_str("        }\n");

        // Update phase: runs every render
        if !rsx_out.update_code.is_empty() {
            for line in rsx_out.update_code.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                out.push_str("        ");
                out.push_str(trimmed);
                out.push('\n');
            }
        }
    } else {
        // Old-style imperative path (no RSX)
        let body = &source[func.body_span.0..func.body_span.1];
        let transformed = transform_component_body(body, component_id);
        for line in transformed.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if trimmed.starts_with("extern ") { continue; }
            out.push_str("        ");
            out.push_str(trimmed);
            out.push('\n');
        }
    }

    out.push_str("        __volki_component_end();\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
}

/// Count the number of `use_ref` / `use_ref_el` calls in a component's logic section.
/// Used to determine the ref slot offset for RSX-generated refs.
fn count_user_refs(func: &RsxFunction, source: &str) -> u32 {
    let body = if let Some(split) = scanner::split_component_body(source, func.body_span) {
        &source[split.logic_span.0..split.logic_span.1]
    } else {
        &source[func.body_span.0..func.body_span.1]
    };
    let mut count: u32 = 0;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.contains("use_ref_el(") {
            count += 1;
        } else if trimmed.contains("use_ref(") {
            count += 1;
        }
    }
    count
}

/// Transform `dom::` and `state::` API calls in a Client function body to extern calls.
///
/// Handles:
/// - `dom::query`, `.set_text`, `.add_class`, `.remove_class`, `.set_attr`, `dom::log`
/// - `state::get_i32("comp", slot)` → `__volki_xstate_get_i32(comp_id, slot)`
/// - `state::set_i32("comp", slot, value)` → `__volki_xstate_set_i32(comp_id, slot, value)`
fn transform_client_body(body: &str, component_ids: &[(String, u32)]) -> String {
    let mut out = String::with_capacity(body.len() * 2);
    let mut var_counter: u32 = 0;
    let mut in_extern = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        // Track extern "C" blocks — skip entirely (contents already hoisted)
        if trimmed.starts_with("extern ") && trimmed.contains('{') {
            in_extern = true;
            continue;
        }
        if in_extern {
            if trimmed.contains('}') {
                in_extern = false;
            }
            continue;
        }

        // ref::get_i32/f32
        if let Some(transformed) = transform_ref_get(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // ref::set_i32/f32
        if let Some(transformed) = transform_ref_set(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // state::get_str("comp", slot) → two-call pattern
        if let Some(transformed) = transform_state_get_str(trimmed, component_ids) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // state::set_str("comp", slot, value) → extern call
        if let Some(transformed) = transform_state_set_str(trimmed, component_ids, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // state::get_i32("comp", slot) → __volki_xstate_get_i32(comp_id, slot)
        if let Some(transformed) = transform_state_get(trimmed, component_ids) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // state::set_i32("comp", slot, value) → __volki_xstate_set_i32(comp_id, slot, value)
        if let Some(transformed) = transform_state_set(trimmed, component_ids) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::create(tag) → __volki_dom_create(ptr, len)
        if let Some(transformed) = transform_dom_create(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::append(parent, child) → __volki_dom_append(parent, child)
        if let Some(transformed) = transform_dom_append(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::remove(handle) → __volki_dom_remove(handle)
        if let Some(transformed) = transform_dom_remove(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::set_html(handle, html) → __volki_dom_set_html(handle, ptr, len)
        if let Some(transformed) = transform_dom_set_html(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::query_all_count(sel) → __volki_dom_query_all_count(ptr, len)
        if let Some(transformed) = transform_dom_query_all_count(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::query_all_get(sel, idx) → __volki_dom_query_all_get(ptr, len, idx)
        if let Some(transformed) = transform_dom_query_all_get(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::query(expr) → __volki_dom_query(ptr, len)
        if let Some(transformed) = transform_dom_query(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.get_attr(name) → two-call pattern
        if let Some(transformed) = transform_get_attr(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // let v = el.get_value() → ptr+len string reconstruction
        if let Some(transformed) = transform_get_value(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.set_text(expr) → __volki_dom_set_text(handle, ptr, len)
        if let Some(transformed) = transform_method_call(trimmed, ".set_text(", "__volki_dom_set_text", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.add_class(expr)
        if let Some(transformed) = transform_method_call(trimmed, ".add_class(", "__volki_dom_add_class", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.remove_class(expr)
        if let Some(transformed) = transform_method_call(trimmed, ".remove_class(", "__volki_dom_remove_class", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.toggle_class(expr)
        if let Some(transformed) = transform_method_call(trimmed, ".toggle_class(", "__volki_dom_toggle_class", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.remove_attr(name)
        if let Some(transformed) = transform_method_call(trimmed, ".remove_attr(", "__volki_dom_remove_attr", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.set_attr(name, value)
        if let Some(transformed) = transform_set_attr(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::log(expr)
        if let Some(transformed) = transform_dom_log(trimmed, &mut var_counter, body) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // Pass through unchanged
        out.push_str(trimmed);
        out.push('\n');
    }

    out
}

/// Transform a Component function body. Handles:
/// - `use_state(initial)` → `__volki_state_init_<type>(slot, initial)`
/// - `state::fmt_i32(val)` → alloc + `__volki_state_fmt_i32(val, buf, len)`
/// - All dom:: transforms (Components use DOM too)
fn transform_component_body(body: &str, component_id: u32) -> String {
    let mut out = String::with_capacity(body.len() * 2);
    let mut var_counter: u32 = 0;
    let mut slot_counter: u32 = 0;
    let mut ref_slot_counter: u32 = 0;
    let mut effect_slot_counter: u32 = 0;
    let mut memo_slot_counter: u32 = 0;
    let mut fmt_counter: u32 = 0;
    let mut in_extern = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        // Track extern "C" blocks — skip entirely (contents already hoisted)
        if trimmed.starts_with("extern ") && trimmed.contains('{') {
            in_extern = true;
            continue;
        }
        if in_extern {
            if trimmed.contains('}') {
                in_extern = false;
            }
            continue;
        }

        // use_memo_i32/f32
        if let Some(transformed) = transform_use_memo(trimmed, &mut memo_slot_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // use_effect(&[dep1, dep2])
        if let Some(transformed) = transform_use_effect(trimmed, &mut effect_slot_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // use_ref_el("selector") → ref init el (must be checked before use_ref)
        if let Some(transformed) = transform_use_ref_el(trimmed, &mut ref_slot_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // use_ref(initial) → ref init (must be checked before use_state to avoid collision)
        if let Some(transformed) = transform_use_ref(trimmed, &mut ref_slot_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // ref::get_i32/f32
        if let Some(transformed) = transform_ref_get(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // ref::set_i32/f32
        if let Some(transformed) = transform_ref_set(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // use_state("string") → string state init (must be checked before use_state)
        if let Some(transformed) = transform_use_state_str(trimmed, &mut slot_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // let (state, set_state) = use_state(...) tuple API
        if let Some(transformed) = transform_use_state_tuple(trimmed, &mut slot_counter, component_id) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // use_state(initial) → __volki_state_init_<type>(slot, initial)
        if let Some(transformed) = transform_use_state(trimmed, &mut slot_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // Handle lines containing state::fmt_i32/f32 (may be nested in other calls)
        if trimmed.contains("state::fmt_i32(") || trimmed.contains("state::fmt_f32(") {
            if let Some(transformed) = transform_state_fmt_line(trimmed, &mut var_counter, &mut fmt_counter) {
                out.push_str(transformed.as_str());
                out.push('\n');
                continue;
            }
        }

        // dom::create(tag)
        if let Some(transformed) = transform_dom_create(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::append(parent, child)
        if let Some(transformed) = transform_dom_append(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::remove(handle)
        if let Some(transformed) = transform_dom_remove(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::set_html(handle, html)
        if let Some(transformed) = transform_dom_set_html(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::query_all_count(sel)
        if let Some(transformed) = transform_dom_query_all_count(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::query_all_get(sel, idx)
        if let Some(transformed) = transform_dom_query_all_get(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::query(expr) → __volki_dom_query(ptr, len)
        if let Some(transformed) = transform_dom_query(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.get_attr(name)
        if let Some(transformed) = transform_get_attr(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // let v = el.get_value() → ptr+len string reconstruction
        if let Some(transformed) = transform_get_value(trimmed) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.set_text(expr) — check if arg is a __fb variable (from fmt transform)
        if let Some(transformed) = transform_method_call(trimmed, ".set_text(", "__volki_dom_set_text", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.add_class(expr)
        if let Some(transformed) = transform_method_call(trimmed, ".add_class(", "__volki_dom_add_class", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.remove_class(expr)
        if let Some(transformed) = transform_method_call(trimmed, ".remove_class(", "__volki_dom_remove_class", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.toggle_class(expr)
        if let Some(transformed) = transform_method_call(trimmed, ".toggle_class(", "__volki_dom_toggle_class", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.remove_attr(name)
        if let Some(transformed) = transform_method_call(trimmed, ".remove_attr(", "__volki_dom_remove_attr", &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // el.set_attr(name, value)
        if let Some(transformed) = transform_set_attr(trimmed, &mut var_counter) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // dom::log(expr)
        if let Some(transformed) = transform_dom_log(trimmed, &mut var_counter, body) {
            out.push_str(transformed.as_str());
            out.push('\n');
            continue;
        }

        // Pass through unchanged
        out.push_str(trimmed);
        out.push('\n');
    }

    out
}

/// Transform `let x = use_state(0_i32);` → `let x = __volki_state_init_i32(slot, 0);`
fn transform_use_state(line: &str, slot_counter: &mut u32) -> Option<String> {
    let use_idx = line.find("use_state(")?;

    // Extract variable name
    let before = &line[..use_idx];
    let var_name = extract_let_var(before)?;

    // Extract argument
    let arg_start = use_idx + "use_state(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let slot = *slot_counter;
    *slot_counter += 1;

    let mut out = String::new();

    // Infer type and strip suffix
    let (extern_fn, clean_arg) = if arg.ends_with("_i32") {
        ("__volki_state_init_i32", &arg[..arg.len() - 4])
    } else if arg.ends_with("_f32") {
        ("__volki_state_init_f32", &arg[..arg.len() - 4])
    } else if arg == "true" {
        ("__volki_state_init_i32", "1")
    } else if arg == "false" {
        ("__volki_state_init_i32", "0")
    } else {
        // Default to i32
        ("__volki_state_init_i32", arg)
    };

    out.push_str("let ");
    out.push_str(var_name);
    out.push_str(" = ");
    out.push_str(extern_fn);
    out.push('(');
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(", ");
    out.push_str(clean_arg);
    out.push_str(");");

    Some(out)
}

/// Transform tuple state declaration:
/// `let (count, set_count) = use_state(0_i32);`
fn transform_use_state_tuple(line: &str, slot_counter: &mut u32, component_id: u32) -> Option<String> {
    let use_idx = line.find("use_state(")?;
    let before = line[..use_idx].trim();
    if !before.starts_with("let (") || !before.ends_with('=') {
        return None;
    }

    let tuple_inner = before.trim_start_matches("let (").trim_end_matches('=').trim();
    let tuple_inner = tuple_inner.strip_suffix(')')?;
    let comma = tuple_inner.find(',')?;
    let state_var = tuple_inner[..comma].trim();
    let setter_var = tuple_inner[comma + 1..].trim();
    if state_var.is_empty() || setter_var.is_empty() {
        return None;
    }

    let arg_start = use_idx + "use_state(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let slot = *slot_counter;
    *slot_counter += 1;

    let (state_init_fn, setter_fn, clean_arg, setter_ty) = if arg.ends_with("_i32") {
        ("__volki_state_init_i32", "__volki_xstate_set_i32", &arg[..arg.len() - 4], "i32")
    } else if arg.ends_with("_f32") {
        ("__volki_state_init_f32", "__volki_xstate_set_f32", &arg[..arg.len() - 4], "f32")
    } else if arg == "true" {
        ("__volki_state_init_i32", "__volki_xstate_set_i32", "1", "i32")
    } else if arg == "false" {
        ("__volki_state_init_i32", "__volki_xstate_set_i32", "0", "i32")
    } else {
        ("__volki_state_init_i32", "__volki_xstate_set_i32", arg, "i32")
    };

    let mut out = String::new();
    out.push_str("let ");
    out.push_str(state_var);
    out.push_str(" = ");
    out.push_str(state_init_fn);
    out.push('(');
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(", ");
    out.push_str(clean_arg);
    out.push_str(");\n");
    out.push_str("let ");
    out.push_str(setter_var);
    out.push_str(" = |value: ");
    out.push_str(setter_ty);
    out.push_str("| {\n");
    out.push_str("    ");
    out.push_str(setter_fn);
    out.push('(');
    out.push_str(crate::vformat!("{}", component_id).as_str());
    out.push_str(", ");
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(", value);\n");
    out.push_str("};");

    Some(out)
}

/// Transform a line containing `state::fmt_i32(val)` or `state::fmt_f32(val)`.
///
/// If the line is like `el.set_text(state::fmt_i32(count));`, generates:
/// ```text
/// let __fb0 = __volki_alloc(20);
/// let __fl0 = __volki_state_fmt_i32(count, __fb0, 20);
/// __volki_dom_set_text(el, __fb0, __fl0);
/// ```
fn transform_state_fmt_line(line: &str, _var_counter: &mut u32, fmt_counter: &mut u32) -> Option<String> {
    // Detect which format call is present
    let (fmt_idx, fmt_fn, buf_size) = if let Some(idx) = line.find("state::fmt_i32(") {
        (idx, "__volki_state_fmt_i32", 20)
    } else if let Some(idx) = line.find("state::fmt_f32(") {
        (idx, "__volki_state_fmt_f32", 32)
    } else {
        return None;
    };

    // Extract the value argument inside state::fmt_xxx(value)
    let prefix_len = if line[fmt_idx..].starts_with("state::fmt_i32(") {
        "state::fmt_i32(".len()
    } else {
        "state::fmt_f32(".len()
    };
    let val_start = fmt_idx + prefix_len;
    let val_end = find_closing_paren(line, val_start)?;
    let value_arg = line[val_start..val_end].trim();

    let fc = *fmt_counter;
    *fmt_counter += 1;
    let buf_var = crate::vformat!("__fb{}", fc);
    let len_var = crate::vformat!("__fl{}", fc);

    let mut out = String::new();

    // Allocate buffer
    out.push_str("let ");
    out.push_str(buf_var.as_str());
    out.push_str(" = __volki_alloc(");
    out.push_str(crate::vformat!("{}", buf_size).as_str());
    out.push_str(");\n");

    // Call formatter
    out.push_str("let ");
    out.push_str(len_var.as_str());
    out.push_str(" = ");
    out.push_str(fmt_fn);
    out.push('(');
    out.push_str(value_arg);
    out.push_str(", ");
    out.push_str(buf_var.as_str());
    out.push_str(", ");
    out.push_str(crate::vformat!("{}", buf_size).as_str());
    out.push_str(");\n");

    // Now check if this is nested inside a .set_text() call
    if let Some(method_idx) = line.find(".set_text(") {
        let obj = line[..method_idx].trim().trim_end_matches(';');
        out.push_str("__volki_dom_set_text(");
        out.push_str(obj);
        out.push_str(", ");
        out.push_str(buf_var.as_str());
        out.push_str(", ");
        out.push_str(len_var.as_str());
        out.push_str(");");
    } else if line.trim().starts_with("let ") {
        // let result = state::fmt_i32(val); — store as ptr+len pair
        // Just emit the alloc + fmt, the variable holds the len
        // Remove trailing newline from the fmt call
        if out.ends_with("\n") {
            out.truncate(out.len() - 1);
        }
    }

    Some(out)
}

/// Transform `let x = state::get_i32("comp", slot);` → `let x = __volki_xstate_get_i32(comp_id, slot);`
fn transform_state_get(line: &str, component_ids: &[(String, u32)]) -> Option<String> {
    // Try both i32 and f32 variants
    let (get_idx, extern_fn) = if let Some(idx) = line.find("state::get_i32(") {
        (idx, "__volki_xstate_get_i32")
    } else if let Some(idx) = line.find("state::get_f32(") {
        (idx, "__volki_xstate_get_f32")
    } else {
        return None;
    };

    // Extract variable name
    let before = &line[..get_idx];
    let var_name = extract_let_var(before)?;

    // Extract arguments
    let prefix_len = if line[get_idx..].starts_with("state::get_i32(") {
        "state::get_i32(".len()
    } else {
        "state::get_f32(".len()
    };
    let arg_start = get_idx + prefix_len;
    let arg_end = find_closing_paren(line, arg_start)?;
    let args_str = &line[arg_start..arg_end];

    // Split on comma: "counter", 0
    let comma = args_str.find(',')?;
    let comp_arg = args_str[..comma].trim();
    let slot_arg = args_str[comma + 1..].trim();

    // Resolve component name to ID
    let comp_name = comp_arg.trim_matches('"');
    let comp_id = resolve_component_id(comp_name, component_ids);

    let mut out = String::new();
    out.push_str("let ");
    out.push_str(var_name);
    out.push_str(" = ");
    out.push_str(extern_fn);
    out.push('(');
    out.push_str(crate::vformat!("{}", comp_id).as_str());
    out.push_str(", ");
    out.push_str(slot_arg);
    out.push_str(");");

    Some(out)
}

/// Transform `state::set_i32("comp", slot, value);` → `__volki_xstate_set_i32(comp_id, slot, value);`
fn transform_state_set(line: &str, component_ids: &[(String, u32)]) -> Option<String> {
    let (set_idx, extern_fn) = if let Some(idx) = line.find("state::set_i32(") {
        (idx, "__volki_xstate_set_i32")
    } else if let Some(idx) = line.find("state::set_f32(") {
        (idx, "__volki_xstate_set_f32")
    } else {
        return None;
    };

    let prefix_len = if line[set_idx..].starts_with("state::set_i32(") {
        "state::set_i32(".len()
    } else {
        "state::set_f32(".len()
    };
    let arg_start = set_idx + prefix_len;
    let arg_end = find_closing_paren(line, arg_start)?;
    let args_str = &line[arg_start..arg_end];

    // Split: "comp", slot, value
    let first_comma = args_str.find(',')?;
    let comp_arg = args_str[..first_comma].trim();
    let rest = &args_str[first_comma + 1..];
    let second_comma = rest.find(',')?;
    let slot_arg = rest[..second_comma].trim();
    let value_arg = rest[second_comma + 1..].trim();

    let comp_name = comp_arg.trim_matches('"');
    let comp_id = resolve_component_id(comp_name, component_ids);

    let mut out = String::new();
    out.push_str(extern_fn);
    out.push('(');
    out.push_str(crate::vformat!("{}", comp_id).as_str());
    out.push_str(", ");
    out.push_str(slot_arg);
    out.push_str(", ");
    out.push_str(value_arg);
    out.push_str(");");

    Some(out)
}

/// Resolve a component name string to its numeric ID.
fn resolve_component_id(name: &str, component_ids: &[(String, u32)]) -> u32 {
    for (comp_name, id) in component_ids {
        if comp_name.as_str() == name {
            return *id;
        }
    }
    0 // fallback
}

fn collect_state_helper_bindings(
    component_fns: &[&RsxFunction],
    source: &str,
    component_ids: &[(String, u32)],
) -> Vec<StateHelperBinding> {
    let mut out = Vec::new();

    for func in component_fns {
        let Some(comp_name) = &func.name else { continue; };
        let comp_id = resolve_component_id(comp_name.as_str(), component_ids);
        let body = &source[func.body_span.0..func.body_span.1];
        let mut slot = 0_u32;

        for raw in body.lines() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((state_var, setter_var, kind)) = parse_use_state_tuple_decl(line) {
                out.push(StateHelperBinding {
                    getter: crate::vformat!("get_{}", state_var),
                    setter: setter_var,
                    comp_id,
                    slot,
                    kind,
                });
                slot += 1;
                continue;
            }
            if line.contains("use_state(") {
                slot += 1;
            }
        }
    }

    out
}

fn parse_use_state_tuple_decl(line: &str) -> Option<(String, String, StateHelperKind)> {
    let use_idx = line.find("use_state(")?;
    let before = line[..use_idx].trim();
    if !before.starts_with("let (") || !before.ends_with('=') {
        return None;
    }

    let tuple_inner = before.trim_start_matches("let (").trim_end_matches('=').trim();
    let tuple_inner = tuple_inner.strip_suffix(')')?;
    let comma = tuple_inner.find(',')?;
    let state_var = tuple_inner[..comma].trim();
    let setter_var = tuple_inner[comma + 1..].trim();
    if state_var.is_empty() || setter_var.is_empty() {
        return None;
    }

    let arg_start = use_idx + "use_state(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();
    let kind = if arg.ends_with("_f32") {
        StateHelperKind::F32
    } else {
        StateHelperKind::I32
    };

    Some((String::from(state_var), String::from(setter_var), kind))
}

/// Transform `let var = dom::query("sel");` or `let var = dom::query(expr);`
fn transform_dom_query(line: &str, _counter: &mut u32) -> Option<String> {
    // Match: let <var> = dom::query(<arg>);
    let query_idx = line.find("dom::query(")?;

    // Extract variable name from `let <var> = `
    let before = &line[..query_idx];
    let var_name = extract_let_var(before)?;

    // Extract argument
    let arg_start = query_idx + "dom::query(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let mut out = String::new();
    // If arg is a string literal
    if arg.starts_with('"') {
        let s = &arg[1..arg.len() - 1]; // strip quotes
        out.push_str("let __q = \"");
        out.push_str(s);
        out.push_str("\";\n");
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_dom_query(__q.as_ptr() as i32, __q.len() as i32);");
    } else {
        // arg is a variable reference
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_dom_query(");
        out.push_str(arg);
        out.push_str(".as_ptr() as i32, ");
        out.push_str(arg);
        out.push_str(".len() as i32);");
    }

    Some(out)
}

/// Transform `el.set_text("value");` → `__volki_dom_set_text(el, ptr, len);`
fn transform_method_call(line: &str, method: &str, extern_name: &str, counter: &mut u32) -> Option<String> {
    let method_idx = line.find(method)?;

    // Extract the object (variable before the method call)
    let obj = line[..method_idx].trim().trim_end_matches(';');

    // Extract argument
    let arg_start = method_idx + method.len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let tmp = crate::vformat!("__s{}", counter);
    *counter += 1;

    let mut out = String::new();
    if arg.starts_with('"') {
        let s = &arg[1..arg.len() - 1];
        out.push_str("let ");
        out.push_str(tmp.as_str());
        out.push_str(" = \"");
        out.push_str(s);
        out.push_str("\";\n");
        out.push_str(extern_name);
        out.push('(');
        out.push_str(obj);
        out.push_str(", ");
        out.push_str(tmp.as_str());
        out.push_str(".as_ptr() as i32, ");
        out.push_str(tmp.as_str());
        out.push_str(".len() as i32);");
    } else {
        out.push_str(extern_name);
        out.push('(');
        out.push_str(obj);
        out.push_str(", ");
        out.push_str(arg);
        out.push_str(".as_ptr() as i32, ");
        out.push_str(arg);
        out.push_str(".len() as i32);");
    }

    Some(out)
}

/// Transform `el.set_attr("name", "value");`
fn transform_set_attr(line: &str, counter: &mut u32) -> Option<String> {
    let method_idx = line.find(".set_attr(")?;
    let obj = line[..method_idx].trim();

    let arg_start = method_idx + ".set_attr(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let args_str = &line[arg_start..arg_end];

    // Split on comma (simple — assumes no commas in string literals)
    let comma = args_str.find(',')?;
    let name_arg = args_str[..comma].trim();
    let val_arg = args_str[comma + 1..].trim();

    let tmp_n = crate::vformat!("__n{}", counter);
    let tmp_v = crate::vformat!("__v{}", counter);
    *counter += 1;

    let mut out = String::new();

    // Name
    if name_arg.starts_with('"') {
        let s = &name_arg[1..name_arg.len() - 1];
        out.push_str("let ");
        out.push_str(tmp_n.as_str());
        out.push_str(" = \"");
        out.push_str(s);
        out.push_str("\";\n");
    } else {
        out.push_str("let ");
        out.push_str(tmp_n.as_str());
        out.push_str(" = ");
        out.push_str(name_arg);
        out.push_str(";\n");
    }

    // Value
    if val_arg.starts_with('"') {
        let s = &val_arg[1..val_arg.len() - 1];
        out.push_str("let ");
        out.push_str(tmp_v.as_str());
        out.push_str(" = \"");
        out.push_str(s);
        out.push_str("\";\n");
    } else {
        out.push_str("let ");
        out.push_str(tmp_v.as_str());
        out.push_str(" = ");
        out.push_str(val_arg);
        out.push_str(";\n");
    }

    out.push_str("__volki_dom_set_attr(");
    out.push_str(obj);
    out.push_str(", ");
    out.push_str(tmp_n.as_str());
    out.push_str(".as_ptr() as i32, ");
    out.push_str(tmp_n.as_str());
    out.push_str(".len() as i32, ");
    out.push_str(tmp_v.as_str());
    out.push_str(".as_ptr() as i32, ");
    out.push_str(tmp_v.as_str());
    out.push_str(".len() as i32);");

    Some(out)
}

/// Transform `dom::log("message");` or `dom::log(i32_value);`
///
/// Auto-detects whether the argument is a string or i32:
/// - String literals (`"..."`) → `__volki_log_str`
/// - i32 values (numeric literals, variables from `get_*()`, arithmetic) → `__volki_log_i32`
/// - String variables (from `.get_value()`, `state::get_str`) → `__volki_log_str`
fn transform_dom_log(line: &str, counter: &mut u32, body: &str) -> Option<String> {
    let idx = line.find("dom::log(")?;
    let arg_start = idx + "dom::log(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let tmp = crate::vformat!("__l{}", counter);
    *counter += 1;

    let mut out = String::new();
    if arg.starts_with('"') {
        // String literal
        let s = &arg[1..arg.len() - 1];
        out.push_str("let ");
        out.push_str(tmp.as_str());
        out.push_str(" = \"");
        out.push_str(s);
        out.push_str("\";\n");
        out.push_str("__volki_log_str(");
        out.push_str(tmp.as_str());
        out.push_str(");");
    } else if is_i32_arg(arg, body) {
        // i32 value — log directly as number
        out.push_str("__volki_log_i32(");
        out.push_str(arg);
        out.push_str(");");
    } else {
        // String variable
        out.push_str("__volki_log_str(");
        out.push_str(arg);
        out.push_str(");");
    }

    Some(out)
}

/// Heuristic: is this `dom::log` argument an i32 expression?
///
/// Checks for numeric literals, arithmetic expressions, and variables
/// declared from i32-returning sources (cross-component getters, `state::get_i32`).
fn is_i32_arg(arg: &str, body: &str) -> bool {
    // Pure numeric literal (e.g. 42, -1)
    let trimmed = arg.trim_start_matches('-');
    if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    // Arithmetic expression containing operators
    if arg.contains(" + ") || arg.contains(" - ") || arg.contains(" * ") || arg.contains(" / ") || arg.contains(" % ") {
        return true;
    }
    // Function call returning i32 (get_* pattern from cross-component state)
    if arg.starts_with("get_") && arg.ends_with(')') {
        return true;
    }
    // Check how the variable was declared in the function body
    let let_pat = crate::vformat!("let {} = ", arg);
    for bline in body.lines() {
        let bt = bline.trim();
        if let Some(rest) = bt.strip_prefix(let_pat.as_str()) {
            // i32-returning sources
            if rest.starts_with("get_") { return true; }
            if rest.starts_with("state::get_i32(") { return true; }
            if rest.starts_with("state::get_f32(") { return true; }
            // String-returning sources
            if rest.contains(".get_value()") { return false; }
            if rest.starts_with("state::get_str(") { return false; }
            if rest.starts_with("state::fmt_") { return false; }
            if rest.starts_with('"') { return false; }
        }
    }
    // Check for tuple destructuring: let (arg, set_arg) = use_state(0_i32);
    let tuple_pat = crate::vformat!("let ({},", arg);
    let tuple_pat2 = crate::vformat!("let ({} ,", arg);
    for bline in body.lines() {
        let bt = bline.trim();
        if (bt.starts_with(tuple_pat.as_str()) || bt.starts_with(tuple_pat2.as_str()))
            && bt.contains("use_state(")
        {
            if bt.contains("_i32") || bt.contains("_f32") {
                return true;
            }
        }
    }
    false
}

/// Transform `let x = use_state("hello");` → string state init with two-call pattern.
fn transform_use_state_str(line: &str, slot_counter: &mut u32) -> Option<String> {
    let use_idx = line.find("use_state(")?;

    // Check if arg starts with a quote (string state)
    let arg_start = use_idx + "use_state(".len();
    let trimmed_arg = line[arg_start..].trim_start();
    if !trimmed_arg.starts_with('"') {
        return None;
    }

    let before = &line[..use_idx];
    let var_name = extract_let_var(before)?;

    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let slot = *slot_counter;
    *slot_counter += 1;

    // arg should be a string literal like "hello"
    let s = &arg[1..arg.len() - 1]; // strip quotes

    let mut out = String::new();
    out.push_str("let __sinit");
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(" = \"");
    out.push_str(s);
    out.push_str("\";\n");
    out.push_str("let __sptr = __volki_state_init_str(");
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(", __sinit");
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(".as_ptr() as i32, __sinit");
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(".len() as i32);\n");
    out.push_str("let __slen = __volki_state_init_str_len(");
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(");\n");
    out.push_str("let ");
    out.push_str(var_name);
    out.push_str(" = core::str::from_utf8_unchecked(core::slice::from_raw_parts(__sptr as *const u8, __slen as usize));");

    Some(out)
}

/// Transform `let val = state::get_str("comp", slot);` → two-call ptr+len pattern.
fn transform_state_get_str(line: &str, component_ids: &[(String, u32)]) -> Option<String> {
    let get_idx = line.find("state::get_str(")?;
    let before = &line[..get_idx];
    let var_name = extract_let_var(before)?;

    let arg_start = get_idx + "state::get_str(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let args_str = &line[arg_start..arg_end];

    let comma = args_str.find(',')?;
    let comp_arg = args_str[..comma].trim();
    let slot_arg = args_str[comma + 1..].trim();

    let comp_name = comp_arg.trim_matches('"');
    let comp_id = resolve_component_id(comp_name, component_ids);

    let mut out = String::new();
    out.push_str("let __gsptr = __volki_xstate_get_str(");
    out.push_str(crate::vformat!("{}", comp_id).as_str());
    out.push_str(", ");
    out.push_str(slot_arg);
    out.push_str(");\n");
    out.push_str("let __gslen = __volki_xstate_get_str_len(");
    out.push_str(crate::vformat!("{}", comp_id).as_str());
    out.push_str(", ");
    out.push_str(slot_arg);
    out.push_str(");\n");
    out.push_str("let ");
    out.push_str(var_name);
    out.push_str(" = core::str::from_utf8_unchecked(core::slice::from_raw_parts(__gsptr as *const u8, __gslen as usize));");

    Some(out)
}

/// Transform `state::set_str("comp", slot, value);` → extern call with ptr+len.
fn transform_state_set_str(line: &str, component_ids: &[(String, u32)], counter: &mut u32) -> Option<String> {
    let set_idx = line.find("state::set_str(")?;
    let arg_start = set_idx + "state::set_str(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let args_str = &line[arg_start..arg_end];

    let first_comma = args_str.find(',')?;
    let comp_arg = args_str[..first_comma].trim();
    let rest = &args_str[first_comma + 1..];
    let second_comma = rest.find(',')?;
    let slot_arg = rest[..second_comma].trim();
    let value_arg = rest[second_comma + 1..].trim();

    let comp_name = comp_arg.trim_matches('"');
    let comp_id = resolve_component_id(comp_name, component_ids);

    let tmp = crate::vformat!("__ss{}", counter);
    *counter += 1;

    let mut out = String::new();
    if value_arg.starts_with('"') {
        let s = &value_arg[1..value_arg.len() - 1];
        out.push_str("let ");
        out.push_str(tmp.as_str());
        out.push_str(" = \"");
        out.push_str(s);
        out.push_str("\";\n");
    } else {
        out.push_str("let ");
        out.push_str(tmp.as_str());
        out.push_str(" = ");
        out.push_str(value_arg);
        out.push_str(";\n");
    }
    out.push_str("__volki_xstate_set_str(");
    out.push_str(crate::vformat!("{}", comp_id).as_str());
    out.push_str(", ");
    out.push_str(slot_arg);
    out.push_str(", ");
    out.push_str(tmp.as_str());
    out.push_str(".as_ptr() as i32, ");
    out.push_str(tmp.as_str());
    out.push_str(".len() as i32);");

    Some(out)
}

/// Transform `let div = dom::create("div");` → `__volki_dom_create(ptr, len)`
fn transform_dom_create(line: &str, _counter: &mut u32) -> Option<String> {
    let idx = line.find("dom::create(")?;
    let before = &line[..idx];
    let var_name = extract_let_var(before)?;

    let arg_start = idx + "dom::create(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let mut out = String::new();
    if arg.starts_with('"') {
        let s = &arg[1..arg.len() - 1];
        out.push_str("let __ct = \"");
        out.push_str(s);
        out.push_str("\";\n");
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_dom_create(__ct.as_ptr() as i32, __ct.len() as i32);");
    } else {
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_dom_create(");
        out.push_str(arg);
        out.push_str(".as_ptr() as i32, ");
        out.push_str(arg);
        out.push_str(".len() as i32);");
    }

    Some(out)
}

/// Transform `dom::append(parent, child);` → `__volki_dom_append(parent, child);`
fn transform_dom_append(line: &str) -> Option<String> {
    let idx = line.find("dom::append(")?;
    let arg_start = idx + "dom::append(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let args_str = &line[arg_start..arg_end];

    let comma = args_str.find(',')?;
    let parent_arg = args_str[..comma].trim();
    let child_arg = args_str[comma + 1..].trim();

    let mut out = String::new();
    out.push_str("__volki_dom_append(");
    out.push_str(parent_arg);
    out.push_str(", ");
    out.push_str(child_arg);
    out.push_str(");");

    Some(out)
}

/// Transform `dom::remove(handle);` → `__volki_dom_remove(handle);`
fn transform_dom_remove(line: &str) -> Option<String> {
    let idx = line.find("dom::remove(")?;
    let arg_start = idx + "dom::remove(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let mut out = String::new();
    out.push_str("__volki_dom_remove(");
    out.push_str(arg);
    out.push_str(");");

    Some(out)
}

/// Transform `dom::set_html(handle, "html");` → `__volki_dom_set_html(handle, ptr, len);`
fn transform_dom_set_html(line: &str, counter: &mut u32) -> Option<String> {
    let idx = line.find("dom::set_html(")?;
    let arg_start = idx + "dom::set_html(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let args_str = &line[arg_start..arg_end];

    let comma = args_str.find(',')?;
    let handle_arg = args_str[..comma].trim();
    let html_arg = args_str[comma + 1..].trim();

    let tmp = crate::vformat!("__h{}", counter);
    *counter += 1;

    let mut out = String::new();
    if html_arg.starts_with('"') {
        let s = &html_arg[1..html_arg.len() - 1];
        out.push_str("let ");
        out.push_str(tmp.as_str());
        out.push_str(" = \"");
        out.push_str(s);
        out.push_str("\";\n");
    } else {
        out.push_str("let ");
        out.push_str(tmp.as_str());
        out.push_str(" = ");
        out.push_str(html_arg);
        out.push_str(";\n");
    }
    out.push_str("__volki_dom_set_html(");
    out.push_str(handle_arg);
    out.push_str(", ");
    out.push_str(tmp.as_str());
    out.push_str(".as_ptr() as i32, ");
    out.push_str(tmp.as_str());
    out.push_str(".len() as i32);");

    Some(out)
}

/// Transform `el.get_attr("name")` → two-call ptr+len pattern for string return.
fn transform_get_attr(line: &str, counter: &mut u32) -> Option<String> {
    let method_idx = line.find(".get_attr(")?;
    let obj = line[..method_idx].trim();

    // Check if this is a let binding
    let before_obj = line[..line.find(obj)?].trim();
    let var_name = if before_obj.ends_with('=') {
        let pre = before_obj[..before_obj.len() - 1].trim();
        if pre.starts_with("let ") {
            Some(&pre[4..])
        } else {
            None
        }
    } else {
        None
    };
    let var_name = var_name?;

    let arg_start = method_idx + ".get_attr(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let tmp = crate::vformat!("__ga{}", counter);
    *counter += 1;

    let mut out = String::new();
    if arg.starts_with('"') {
        let s = &arg[1..arg.len() - 1];
        out.push_str("let ");
        out.push_str(tmp.as_str());
        out.push_str(" = \"");
        out.push_str(s);
        out.push_str("\";\n");
    } else {
        out.push_str("let ");
        out.push_str(tmp.as_str());
        out.push_str(" = ");
        out.push_str(arg);
        out.push_str(";\n");
    }
    out.push_str("let __gaptr = __volki_dom_get_attr(");
    out.push_str(obj);
    out.push_str(", ");
    out.push_str(tmp.as_str());
    out.push_str(".as_ptr() as i32, ");
    out.push_str(tmp.as_str());
    out.push_str(".len() as i32);\n");
    out.push_str("let __galen = __volki_dom_get_attr_len(");
    out.push_str(obj);
    out.push_str(", ");
    out.push_str(tmp.as_str());
    out.push_str(".as_ptr() as i32, ");
    out.push_str(tmp.as_str());
    out.push_str(".len() as i32);\n");
    out.push_str("let ");
    out.push_str(var_name);
    out.push_str(" = core::str::from_utf8_unchecked(core::slice::from_raw_parts(__gaptr as *const u8, __galen as usize));");

    Some(out)
}

/// Transform `let val = el.get_value();` → two-call ptr+len pattern for string return.
fn transform_get_value(line: &str) -> Option<String> {
    let method_idx = line.find(".get_value(")?;
    let lhs = line[..method_idx].trim();
    let (var_name, obj) = if lhs.starts_with("let ") {
        let eq = lhs.rfind('=')?;
        let var = lhs[4..eq].trim();
        let obj = lhs[eq + 1..].trim();
        if var.is_empty() || obj.is_empty() {
            return None;
        }
        (var, obj)
    } else {
        return None;
    };

    // Validate closing paren exists.
    let arg_start = method_idx + ".get_value(".len();
    let _arg_end = find_closing_paren(line, arg_start)?;

    let mut out = String::new();
    out.push_str("let __gvptr = __volki_dom_get_value(");
    out.push_str(obj);
    out.push_str(");\n");
    out.push_str("let __gvlen = __volki_dom_get_value_len(");
    out.push_str(obj);
    out.push_str(");\n");
    out.push_str("let ");
    out.push_str(var_name);
    out.push_str(" = core::str::from_utf8_unchecked(core::slice::from_raw_parts(__gvptr as *const u8, __gvlen as usize));");
    Some(out)
}

/// Transform `let sum = use_memo_i32(a + b, &[a, b]);` → memo begin/changed/store/load.
fn transform_use_memo(line: &str, memo_slot_counter: &mut u32) -> Option<String> {
    let (memo_idx, is_i32) = if let Some(idx) = line.find("use_memo_i32(") {
        (idx, true)
    } else if let Some(idx) = line.find("use_memo_f32(") {
        (idx, false)
    } else {
        return None;
    };

    let before = &line[..memo_idx];
    let var_name = extract_let_var(before)?;

    let prefix_len = if is_i32 { "use_memo_i32(".len() } else { "use_memo_f32(".len() };
    let arg_start = memo_idx + prefix_len;
    let arg_end = find_closing_paren(line, arg_start)?;
    let full_args = &line[arg_start..arg_end];

    // Split on `, &[` to separate expression from deps
    let dep_marker = ", &[";
    let split_idx = full_args.find(dep_marker)?;
    let expr = full_args[..split_idx].trim();
    let deps_part = &full_args[split_idx + dep_marker.len()..];

    // deps_part ends with `]`
    let deps_inner = deps_part.strip_suffix(']')?.trim();

    let deps: Vec<&str> = if deps_inner.is_empty() {
        Vec::new()
    } else {
        deps_inner.split(',').map(|d| d.trim()).collect()
    };

    let slot = *memo_slot_counter;
    *memo_slot_counter += 1;

    let (store_fn, load_fn) = if is_i32 {
        ("__volki_memo_store_i32", "__volki_memo_load_i32")
    } else {
        ("__volki_memo_store_f32", "__volki_memo_load_f32")
    };

    let mut out = String::new();
    out.push_str("__volki_memo_begin(");
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(", ");
    out.push_str(crate::vformat!("{}", deps.len()).as_str());
    out.push_str(");\n");

    for (i, dep) in deps.iter().enumerate() {
        out.push_str("__volki_memo_set_dep(");
        out.push_str(crate::vformat!("{}", slot).as_str());
        out.push_str(", ");
        out.push_str(crate::vformat!("{}", i).as_str());
        out.push_str(", ");
        out.push_str(dep);
        out.push_str(");\n");
    }

    out.push_str("let ");
    out.push_str(var_name);
    out.push_str(" = if __volki_memo_changed(");
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(") == 1 {\n");
    out.push_str("    let __mv = ");
    out.push_str(expr);
    out.push_str(";\n");
    out.push_str("    ");
    out.push_str(store_fn);
    out.push('(');
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(", __mv);\n");
    out.push_str("    __mv\n");
    out.push_str("} else {\n");
    out.push_str("    ");
    out.push_str(load_fn);
    out.push('(');
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(")\n");
    out.push_str("};");

    Some(out)
}

/// Transform `use_effect(&[dep1, dep2]);` → register + set_dep calls.
fn transform_use_effect(line: &str, effect_slot_counter: &mut u32) -> Option<String> {
    let idx = line.find("use_effect(")?;
    let arg_start = idx + "use_effect(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    // arg should be `&[dep1, dep2]`
    let inner = arg.strip_prefix("&[")?;
    let inner = inner.strip_suffix(']')?;
    let inner = inner.trim();

    let slot = *effect_slot_counter;
    *effect_slot_counter += 1;

    // Parse deps
    let deps: Vec<&str> = if inner.is_empty() {
        Vec::new()
    } else {
        inner.split(',').map(|d| d.trim()).collect()
    };

    let mut out = String::new();
    out.push_str("__volki_effect_register(");
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(", ");
    out.push_str(crate::vformat!("{}", deps.len()).as_str());
    out.push_str(");\n");

    for (i, dep) in deps.iter().enumerate() {
        out.push_str("__volki_effect_set_dep(");
        out.push_str(crate::vformat!("{}", slot).as_str());
        out.push_str(", ");
        out.push_str(crate::vformat!("{}", i).as_str());
        out.push_str(", ");
        out.push_str(dep);
        out.push_str(");\n");
    }

    // Remove trailing newline
    if out.ends_with("\n") {
        out.truncate(out.len() - 1);
    }

    Some(out)
}

/// Transform `let my_ref = use_ref(0_i32);` → `__volki_ref_init_i32(slot, 0)`
fn transform_use_ref(line: &str, ref_slot_counter: &mut u32) -> Option<String> {
    let use_idx = line.find("use_ref(")?;

    // Make sure it's not use_ref_el
    if line[use_idx..].starts_with("use_ref_el(") {
        return None;
    }

    let before = &line[..use_idx];
    let var_name = extract_let_var(before)?;

    let arg_start = use_idx + "use_ref(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let slot = *ref_slot_counter;
    *ref_slot_counter += 1;

    let (extern_fn, clean_arg) = if arg.ends_with("_i32") {
        ("__volki_ref_init_i32", &arg[..arg.len() - 4])
    } else if arg.ends_with("_f32") {
        ("__volki_ref_init_f32", &arg[..arg.len() - 4])
    } else {
        ("__volki_ref_init_i32", arg)
    };

    let mut out = String::new();
    out.push_str("let ");
    out.push_str(var_name);
    out.push_str(" = ");
    out.push_str(extern_fn);
    out.push('(');
    out.push_str(crate::vformat!("{}", slot).as_str());
    out.push_str(", ");
    out.push_str(clean_arg);
    out.push_str(");");

    Some(out)
}

/// Transform `let el = use_ref_el("#my-element");` → `__volki_ref_init_el(slot, ptr, len)`
fn transform_use_ref_el(line: &str, ref_slot_counter: &mut u32) -> Option<String> {
    let use_idx = line.find("use_ref_el(")?;
    let before = &line[..use_idx];
    let var_name = extract_let_var(before)?;

    let arg_start = use_idx + "use_ref_el(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let slot = *ref_slot_counter;
    *ref_slot_counter += 1;

    let mut out = String::new();
    if arg.starts_with('"') {
        let s = &arg[1..arg.len() - 1];
        out.push_str("let __rsel = \"");
        out.push_str(s);
        out.push_str("\";\n");
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_ref_init_el(");
        out.push_str(crate::vformat!("{}", slot).as_str());
        out.push_str(", __rsel.as_ptr() as i32, __rsel.len() as i32);");
    } else {
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_ref_init_el(");
        out.push_str(crate::vformat!("{}", slot).as_str());
        out.push_str(", ");
        out.push_str(arg);
        out.push_str(".as_ptr() as i32, ");
        out.push_str(arg);
        out.push_str(".len() as i32);");
    }

    Some(out)
}

/// Transform `let val = ref::get_i32(slot);` → `__volki_ref_get_i32(slot)`
fn transform_ref_get(line: &str) -> Option<String> {
    let (get_idx, extern_fn) = if let Some(idx) = line.find("ref::get_i32(") {
        (idx, "__volki_ref_get_i32")
    } else if let Some(idx) = line.find("ref::get_f32(") {
        (idx, "__volki_ref_get_f32")
    } else {
        return None;
    };

    let before = &line[..get_idx];
    let var_name = extract_let_var(before)?;

    let prefix_len = if line[get_idx..].starts_with("ref::get_i32(") {
        "ref::get_i32(".len()
    } else {
        "ref::get_f32(".len()
    };
    let arg_start = get_idx + prefix_len;
    let arg_end = find_closing_paren(line, arg_start)?;
    let slot_arg = line[arg_start..arg_end].trim();

    let mut out = String::new();
    out.push_str("let ");
    out.push_str(var_name);
    out.push_str(" = ");
    out.push_str(extern_fn);
    out.push('(');
    out.push_str(slot_arg);
    out.push_str(");");

    Some(out)
}

/// Transform `ref::set_i32(slot, value);` → `__volki_ref_set_i32(slot, value)`
fn transform_ref_set(line: &str) -> Option<String> {
    let (set_idx, extern_fn) = if let Some(idx) = line.find("ref::set_i32(") {
        (idx, "__volki_ref_set_i32")
    } else if let Some(idx) = line.find("ref::set_f32(") {
        (idx, "__volki_ref_set_f32")
    } else {
        return None;
    };

    let prefix_len = if line[set_idx..].starts_with("ref::set_i32(") {
        "ref::set_i32(".len()
    } else {
        "ref::set_f32(".len()
    };
    let arg_start = set_idx + prefix_len;
    let arg_end = find_closing_paren(line, arg_start)?;
    let args_str = &line[arg_start..arg_end];

    let comma = args_str.find(',')?;
    let slot_arg = args_str[..comma].trim();
    let value_arg = args_str[comma + 1..].trim();

    let mut out = String::new();
    out.push_str(extern_fn);
    out.push('(');
    out.push_str(slot_arg);
    out.push_str(", ");
    out.push_str(value_arg);
    out.push_str(");");

    Some(out)
}

/// Transform `let count = dom::query_all_count(".item");` → extern call.
fn transform_dom_query_all_count(line: &str, _counter: &mut u32) -> Option<String> {
    let idx = line.find("dom::query_all_count(")?;
    let before = &line[..idx];
    let var_name = extract_let_var(before)?;

    let arg_start = idx + "dom::query_all_count(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let arg = line[arg_start..arg_end].trim();

    let mut out = String::new();
    if arg.starts_with('"') {
        let s = &arg[1..arg.len() - 1];
        out.push_str("let __qac = \"");
        out.push_str(s);
        out.push_str("\";\n");
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_dom_query_all_count(__qac.as_ptr() as i32, __qac.len() as i32);");
    } else {
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_dom_query_all_count(");
        out.push_str(arg);
        out.push_str(".as_ptr() as i32, ");
        out.push_str(arg);
        out.push_str(".len() as i32);");
    }

    Some(out)
}

/// Transform `let item = dom::query_all_get(".item", 0);` → extern call.
fn transform_dom_query_all_get(line: &str, _counter: &mut u32) -> Option<String> {
    let idx = line.find("dom::query_all_get(")?;
    let before = &line[..idx];
    let var_name = extract_let_var(before)?;

    let arg_start = idx + "dom::query_all_get(".len();
    let arg_end = find_closing_paren(line, arg_start)?;
    let args_str = &line[arg_start..arg_end];

    let comma = args_str.find(',')?;
    let sel_arg = args_str[..comma].trim();
    let idx_arg = args_str[comma + 1..].trim();

    let mut out = String::new();
    if sel_arg.starts_with('"') {
        let s = &sel_arg[1..sel_arg.len() - 1];
        out.push_str("let __qag = \"");
        out.push_str(s);
        out.push_str("\";\n");
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_dom_query_all_get(__qag.as_ptr() as i32, __qag.len() as i32, ");
        out.push_str(idx_arg);
        out.push_str(");");
    } else {
        out.push_str("let ");
        out.push_str(var_name);
        out.push_str(" = __volki_dom_query_all_get(");
        out.push_str(sel_arg);
        out.push_str(".as_ptr() as i32, ");
        out.push_str(sel_arg);
        out.push_str(".len() as i32, ");
        out.push_str(idx_arg);
        out.push_str(");");
    }

    Some(out)
}

/// Extract variable name from `let <name> = ` prefix.
fn extract_let_var(before: &str) -> Option<&str> {
    let s = before.trim();
    let s = s.strip_prefix("let ")?;
    let s = s.trim().strip_suffix('=')?.trim();
    Some(s)
}

/// Find closing paren matching the one at `start - 1` (i.e., we're inside the parens).
fn find_closing_paren(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 1;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'"' => {
                // Skip string literal
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' { i += 2; continue; }
                    if bytes[i] == b'"' { break; }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Extract user-declared `extern "C" { ... }` blocks from a function body,
/// collecting the function declarations inside.
fn extract_user_externs(body: &str, externs: &mut Vec<String>) {
    let mut search_from = 0;
    while let Some(idx) = body[search_from..].find("extern \"C\"") {
        let abs_idx = search_from + idx;
        // Find opening brace
        if let Some(brace_start) = body[abs_idx..].find('{') {
            let brace_abs = abs_idx + brace_start;
            if let Some(brace_end) = body[brace_abs..].find('}') {
                let inner = &body[brace_abs + 1..brace_abs + brace_end];
                for line in inner.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("fn ") {
                        externs.push(String::from(trimmed));
                    }
                }
                search_from = brace_abs + brace_end + 1;
                continue;
            }
        }
        break;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::web::compiler::scanner;
    use crate::libs::web::compiler::scanner::RsxReturnType;

    fn empty_components() -> Vec<&'static RsxFunction> {
        Vec::new()
    }

    #[test]
    fn test_generate_simple_client_fn() {
        let source = r#"
pub fn on_click(target: &str) -> Client {
    let el = dom::query(target);
    el.set_text("Clicked!");
}
"#;
        let fns = scanner::scan_functions(source);
        let client_fns: Vec<&RsxFunction> = fns.iter()
            .filter(|f| f.return_type == RsxReturnType::Client)
            .collect();

        let wasm = generate_wasm_module(&client_fns, &empty_components(), source, &Vec::new());

        assert!(wasm.contains("#![no_std]"));
        assert!(wasm.contains("fn __volki_alloc("));
        assert!(wasm.contains("fn __volki_dom_query("));
        assert!(wasm.contains("fn __volki_dom_set_text("));
        assert!(wasm.contains("pub extern \"C\" fn on_click(target_ptr: i32, target_len: i32)"));
        assert!(wasm.contains("core::str::from_utf8_unchecked"));
    }

    #[test]
    fn test_generate_no_params() {
        let source = r#"
pub fn init() -> Client {
    dom::log("initialized");
}
"#;
        let fns = scanner::scan_functions(source);
        let client_fns: Vec<&RsxFunction> = fns.iter()
            .filter(|f| f.return_type == RsxReturnType::Client)
            .collect();

        let wasm = generate_wasm_module(&client_fns, &empty_components(), source, &Vec::new());
        assert!(wasm.contains("pub extern \"C\" fn init()"));
        assert!(wasm.contains("fn __volki_console_log("));
    }

    #[test]
    fn test_generate_mixed_params() {
        let source = r##"
pub fn update(id: i32, text: &str) -> Client {
    let el = dom::query("#item");
    el.set_text(text);
}
"##;
        let fns = scanner::scan_functions(source);
        let client_fns: Vec<&RsxFunction> = fns.iter()
            .filter(|f| f.return_type == RsxReturnType::Client)
            .collect();

        let wasm = generate_wasm_module(&client_fns, &empty_components(), source, &Vec::new());
        assert!(wasm.contains("pub extern \"C\" fn update(id: i32, text_ptr: i32, text_len: i32)"));
    }

    #[test]
    fn test_only_needed_imports() {
        let source = r#"
pub fn log_it() -> Client {
    dom::log("hello");
}
"#;
        let fns = scanner::scan_functions(source);
        let client_fns: Vec<&RsxFunction> = fns.iter()
            .filter(|f| f.return_type == RsxReturnType::Client)
            .collect();

        let wasm = generate_wasm_module(&client_fns, &empty_components(), source, &Vec::new());
        assert!(wasm.contains("fn __volki_console_log("));
        assert!(wasm.contains("fn __volki_console_log_i32("));
        assert!(!wasm.contains("fn __volki_dom_query("));
        assert!(!wasm.contains("fn __volki_dom_set_text("));
    }

    #[test]
    fn test_user_extern_hoisting() {
        let source = r#"
pub fn custom(msg: &str) -> Client {
    extern "C" {
        fn alert(s_ptr: i32, s_len: i32);
    }
    unsafe { alert(msg.as_ptr() as i32, msg.len() as i32); }
}
"#;
        let fns = scanner::scan_functions(source);
        let client_fns: Vec<&RsxFunction> = fns.iter()
            .filter(|f| f.return_type == RsxReturnType::Client)
            .collect();

        let wasm = generate_wasm_module(&client_fns, &empty_components(), source, &Vec::new());
        // User extern should be hoisted to module-level extern block
        assert!(wasm.contains("fn alert(s_ptr: i32, s_len: i32);"));
    }

    #[test]
    fn test_transform_dom_query_literal() {
        let mut counter = 0;
        let result = transform_dom_query("let el = dom::query(\"#btn\");", &mut counter).unwrap();
        assert!(result.contains("__volki_dom_query("));
        assert!(result.contains("\"#btn\""));
    }

    #[test]
    fn test_transform_dom_query_variable() {
        let mut counter = 0;
        let result = transform_dom_query("let el = dom::query(sel);", &mut counter).unwrap();
        assert!(result.contains("sel.as_ptr() as i32"));
        assert!(result.contains("sel.len() as i32"));
    }

    #[test]
    fn test_transform_set_text() {
        let mut counter = 0;
        let result = transform_method_call("el.set_text(\"hello\");", ".set_text(", "__volki_dom_set_text", &mut counter).unwrap();
        assert!(result.contains("__volki_dom_set_text(el"));
    }

    #[test]
    fn test_transform_get_value() {
        let result = transform_get_value("let term = input.get_value();").unwrap();
        assert!(result.contains("__volki_dom_get_value(input)"));
        assert!(result.contains("__volki_dom_get_value_len(input)"));
        assert!(result.contains("let term = core::str::from_utf8_unchecked"));
    }

    // ── Component tests ──

    #[test]
    fn test_generate_component_fn() {
        let source = r##"
pub fn counter() -> Component {
    let count = use_state(0_i32);
    let el = dom::query("#count");
    el.set_text(state::fmt_i32(count));
}
"##;
        let fns = scanner::scan_functions(source);
        let component_fns: Vec<&RsxFunction> = fns.iter()
            .filter(|f| f.return_type == RsxReturnType::Component)
            .collect();

        let component_rsx: Vec<Option<Vec<RsxNode>>> = component_fns.iter().map(|_| None).collect();
        let wasm = generate_wasm_module(&Vec::new(), &component_fns, source, &component_rsx);

        // Component lifecycle
        assert!(wasm.contains("fn __volki_component_begin(id: i32);"));
        assert!(wasm.contains("fn __volki_component_end();"));
        // State init
        assert!(wasm.contains("fn __volki_state_init_i32(slot: i32, initial: i32) -> i32;"));
        // Fmt
        assert!(wasm.contains("fn __volki_state_fmt_i32(value: i32, buf_ptr: i32, buf_len: i32) -> i32;"));
        // Exported with component prefix
        assert!(wasm.contains("pub extern \"C\" fn __volki_component_counter()"));
        // Body wrapping
        assert!(wasm.contains("__volki_component_begin(0)"));
        assert!(wasm.contains("__volki_component_end()"));
        // use_state transformed
        assert!(wasm.contains("__volki_state_init_i32(0, 0)"));
    }

    #[test]
    fn test_transform_use_state_i32() {
        let mut slot = 0;
        let result = transform_use_state("let count = use_state(0_i32);", &mut slot).unwrap();
        assert_eq!(result, "let count = __volki_state_init_i32(0, 0);");
        assert_eq!(slot, 1);
    }

    #[test]
    fn test_transform_use_state_f32() {
        let mut slot = 0;
        let result = transform_use_state("let temp = use_state(0.0_f32);", &mut slot).unwrap();
        assert_eq!(result, "let temp = __volki_state_init_f32(0, 0.0);");
        assert_eq!(slot, 1);
    }

    #[test]
    fn test_transform_use_state_bool() {
        let mut slot = 2;
        let result = transform_use_state("let active = use_state(false);", &mut slot).unwrap();
        assert_eq!(result, "let active = __volki_state_init_i32(2, 0);");
        assert_eq!(slot, 3);
    }

    #[test]
    fn test_transform_use_state_increments_slots() {
        let mut slot = 0;
        let _ = transform_use_state("let a = use_state(0_i32);", &mut slot);
        assert_eq!(slot, 1);
        let _ = transform_use_state("let b = use_state(false);", &mut slot);
        assert_eq!(slot, 2);
    }

    #[test]
    fn test_transform_state_get() {
        let ids = vec![
            (String::from("counter"), 0),
            (String::from("timer"), 1),
        ];
        let result = transform_state_get("let count = state::get_i32(\"counter\", 0);", &ids).unwrap();
        assert_eq!(result, "let count = __volki_xstate_get_i32(0, 0);");
    }

    #[test]
    fn test_transform_state_set() {
        let ids = vec![(String::from("counter"), 0)];
        let result = transform_state_set("state::set_i32(\"counter\", 0, count + 1);", &ids).unwrap();
        assert_eq!(result, "__volki_xstate_set_i32(0, 0, count + 1);");
    }

    #[test]
    fn test_cross_function_state_access() {
        let source = r##"
pub fn counter() -> Component {
    let count = use_state(0_i32);
    let el = dom::query("#count");
    el.set_text(state::fmt_i32(count));
}

pub fn on_increment() -> Client {
    let count = state::get_i32("counter", 0);
    state::set_i32("counter", 0, count + 1);
}
"##;
        let fns = scanner::scan_functions(source);
        let client_fns: Vec<&RsxFunction> = fns.iter()
            .filter(|f| f.return_type == RsxReturnType::Client)
            .collect();
        let component_fns: Vec<&RsxFunction> = fns.iter()
            .filter(|f| f.return_type == RsxReturnType::Component)
            .collect();

        let component_rsx: Vec<Option<Vec<RsxNode>>> = component_fns.iter().map(|_| None).collect();
        let wasm = generate_wasm_module(&client_fns, &component_fns, source, &component_rsx);

        // Cross-function state access externs
        assert!(wasm.contains("fn __volki_xstate_get_i32("));
        assert!(wasm.contains("fn __volki_xstate_set_i32("));
        // Resolved component ID 0
        assert!(wasm.contains("__volki_xstate_get_i32(0, 0)"));
        assert!(wasm.contains("__volki_xstate_set_i32(0, 0, count + 1)"));
    }

    // ── If/else control flow tests ──

    #[test]
    fn test_transform_client_body_if_else() {
        let body = r##"
            let status = dom::query("#conn-status");
            if visible == 1 {
                status.set_text("visible");
            } else {
                status.set_text("hidden");
            }
        "##;
        let ids: Vec<(String, u32)> = Vec::new();
        let result = transform_client_body(body, &ids);
        // Closing braces and else must be preserved
        assert!(result.contains("}"));
        assert!(result.contains("} else {"));
        assert!(result.contains("if visible == 1 {"));
    }

    #[test]
    fn test_transform_component_body_if_else() {
        let body = r##"
            let visible = use_state(0_i32);
            let overlay = dom::query("#dialog");
            if visible == 1 {
                overlay.add_class("visible");
            } else {
                overlay.remove_class("visible");
            }
        "##;
        let result = transform_component_body(body, 0);
        assert!(result.contains("}"));
        assert!(result.contains("} else {"));
        assert!(result.contains("if visible == 1 {"));
        // use_state still transforms
        assert!(result.contains("__volki_state_init_i32(0, 0)"));
    }

    #[test]
    fn test_transform_client_body_extern_still_stripped() {
        let body = r##"
            extern "C" {
                fn custom_fn(x: i32);
            }
            let el = dom::query("#btn");
            el.set_text("hi");
        "##;
        let ids: Vec<(String, u32)> = Vec::new();
        let result = transform_client_body(body, &ids);
        // extern block contents should be stripped
        assert!(!result.contains("custom_fn"));
        assert!(!result.contains("extern"));
        // DOM calls still transform
        assert!(result.contains("__volki_dom_query("));
        assert!(result.contains("__volki_dom_set_text("));
    }

    #[test]
    fn test_generate_component_with_if_else() {
        let source = r##"
pub fn delete_dialog() -> Component {
    let visible = use_state(0_i32);
    let overlay = dom::query("#delete-dialog");
    if visible == 1 {
        overlay.add_class("visible");
    } else {
        overlay.remove_class("visible");
    }
}
"##;
        let fns = scanner::scan_functions(source);
        let component_fns: Vec<&RsxFunction> = fns.iter()
            .filter(|f| f.return_type == RsxReturnType::Component)
            .collect();

        let component_rsx: Vec<Option<Vec<RsxNode>>> = component_fns.iter().map(|_| None).collect();
        let wasm = generate_wasm_module(&Vec::new(), &component_fns, source, &component_rsx);

        // if/else preserved in output
        assert!(wasm.contains("if visible == 1 {"));
        assert!(wasm.contains("} else {"));
        // DOM + state calls transform correctly
        assert!(wasm.contains("__volki_state_init_i32(0, 0)"));
        assert!(wasm.contains("__volki_dom_add_class("));
        assert!(wasm.contains("__volki_dom_remove_class("));
    }
}
