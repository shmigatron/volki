//! RSX → WASM codegen — generates mount and update code for Component RSX bodies.
//!
//! Given parsed RSX nodes from a `return (RSX)` block, produces:
//! - **Mount code**: DOM creation via `__volki_dom_create`/`__volki_dom_append` (runs once)
//! - **Update code**: Dynamic expression slot updates via stored refs (runs every render)

use crate::core::volkiwithstds::collections::String;

use super::parser::{RsxAttr, RsxAttrValue, RsxNode};

/// Output from RSX → WASM codegen.
pub struct WasmRsxOutput {
    /// Code for first render (DOM creation).
    pub mount_code: String,
    /// Code for every render (dynamic expression updates).
    pub update_code: String,
    /// Number of ref slots consumed by RSX (for dynamic expression handles).
    pub ref_slots_used: u32,
    // Feature flags — what externs the generated code needs.
    pub needs_create: bool,
    pub needs_create_text: bool,
    pub needs_append: bool,
    pub needs_add_class: bool,
    pub needs_set_attr: bool,
    pub needs_set_text: bool,
    pub needs_mount_point: bool,
    pub needs_is_mounted: bool,
    pub needs_ref_get_i32: bool,
    pub needs_ref_set_i32: bool,
    pub needs_fmt_i32: bool,
    pub needs_fmt_f32: bool,
}

struct RsxWalker {
    mount: String,
    update: String,
    node_counter: u32,
    dyn_slot_counter: u32,
    ref_slot_offset: u32,
    needs_create: bool,
    needs_create_text: bool,
    needs_append: bool,
    needs_add_class: bool,
    needs_set_attr: bool,
    needs_set_text: bool,
    needs_ref_get_i32: bool,
    needs_ref_set_i32: bool,
    needs_fmt_i32: bool,
    needs_fmt_f32: bool,
}

/// Generate WASM mount/update code for a Component's RSX body.
///
/// `nodes` — parsed RSX nodes from inside `return (...)`.
/// `component_id` — the numeric component ID.
/// `ref_slot_offset` — first ref slot available (after user's `use_ref` calls).
pub fn generate_component_rsx(
    nodes: &[RsxNode],
    component_id: u32,
    ref_slot_offset: u32,
) -> WasmRsxOutput {
    let mut walker = RsxWalker {
        mount: String::with_capacity(1024),
        update: String::with_capacity(512),
        node_counter: 0,
        dyn_slot_counter: 0,
        ref_slot_offset,
        needs_create: false,
        needs_create_text: false,
        needs_append: false,
        needs_add_class: false,
        needs_set_attr: false,
        needs_set_text: false,
        needs_ref_get_i32: false,
        needs_ref_set_i32: false,
        needs_fmt_i32: false,
        needs_fmt_f32: false,
    };

    // Get mount point
    let mp_var = "__rsx_mp";
    walker.mount.push_str("let ");
    walker.mount.push_str(mp_var);
    walker.mount.push_str(" = __volki_component_mount_point(");
    walker.mount.push_str(crate::vformat!("{}", component_id).as_str());
    walker.mount.push_str(");\n");

    // Walk all top-level RSX nodes
    for node in nodes {
        walker.walk_node(node, mp_var);
    }

    WasmRsxOutput {
        mount_code: walker.mount,
        update_code: walker.update,
        ref_slots_used: walker.dyn_slot_counter,
        needs_create: walker.needs_create,
        needs_create_text: walker.needs_create_text,
        needs_append: walker.needs_append,
        needs_add_class: walker.needs_add_class,
        needs_set_attr: walker.needs_set_attr,
        needs_set_text: walker.needs_set_text,
        needs_mount_point: true,
        needs_is_mounted: true,
        needs_ref_get_i32: walker.needs_ref_get_i32,
        needs_ref_set_i32: walker.needs_ref_set_i32,
        needs_fmt_i32: walker.needs_fmt_i32,
        needs_fmt_f32: walker.needs_fmt_f32,
    }
}

impl RsxWalker {
    fn walk_node(&mut self, node: &RsxNode, parent_var: &str) {
        match node {
            RsxNode::Element { tag, attrs, children, .. } => {
                self.walk_element(tag.as_str(), attrs, children, parent_var);
            }
            RsxNode::Text(text) => {
                self.walk_text(text.as_str(), parent_var);
            }
            RsxNode::Expr(expr) => {
                self.walk_expr(expr.as_str(), parent_var);
            }
            RsxNode::CondAnd { .. } | RsxNode::Ternary { .. } => {
                // V1: conditionals in RSX are deferred — emit nothing (skip)
            }
        }
    }

    fn walk_element(&mut self, tag: &str, attrs: &[RsxAttr], children: &[RsxNode], parent_var: &str) {
        let n = self.node_counter;
        self.node_counter += 1;
        let var = crate::vformat!("__rn{}", n);

        // Mount: create element
        self.mount.push_str("let ");
        self.mount.push_str(var.as_str());
        self.mount.push_str(" = __volki_dom_create(\"");
        self.mount.push_str(tag);
        self.mount.push_str("\".as_ptr() as i32, ");
        self.mount.push_str(crate::vformat!("{}", tag.len()).as_str());
        self.mount.push_str(");\n");
        self.needs_create = true;

        // Mount: set attributes
        for attr in attrs {
            match &attr.value {
                RsxAttrValue::Literal(value) => {
                    if attr.name.as_str() == "class" {
                        // Split classes and add each one
                        for cls in value.as_str().split(' ') {
                            if cls.is_empty() { continue; }
                            self.mount.push_str("__volki_dom_add_class(");
                            self.mount.push_str(var.as_str());
                            self.mount.push_str(", \"");
                            self.mount.push_str(cls);
                            self.mount.push_str("\".as_ptr() as i32, ");
                            self.mount.push_str(crate::vformat!("{}", cls.len()).as_str());
                            self.mount.push_str(");\n");
                            self.needs_add_class = true;
                        }
                    } else {
                        // Generic attribute (id, data-*, etc.)
                        self.emit_set_attr(var.as_str(), attr.name.as_str(), value.as_str());
                    }
                }
                RsxAttrValue::Expr(expr) => {
                    if is_event_attr(attr.name.as_str()) {
                        // Event handlers → data-volki-on* attributes
                        let data_attr = crate::vformat!("data-volki-{}", attr.name);
                        self.emit_set_attr(var.as_str(), data_attr.as_str(), expr.as_str());
                    }
                    // Non-event expression attrs deferred to V2
                }
            }
        }

        // Mount: process children
        for child in children {
            self.walk_node(child, var.as_str());
        }

        // Mount: append to parent
        self.mount.push_str("__volki_dom_append(");
        self.mount.push_str(parent_var);
        self.mount.push_str(", ");
        self.mount.push_str(var.as_str());
        self.mount.push_str(");\n");
        self.needs_append = true;
    }

    fn walk_text(&mut self, text: &str, parent_var: &str) {
        let n = self.node_counter;
        self.node_counter += 1;
        let var = crate::vformat!("__rt{}", n);

        // Mount: create text node with content
        self.mount.push_str("let ");
        self.mount.push_str(var.as_str());
        self.mount.push_str(" = __volki_dom_create_text(\"");
        self.mount.push_str(text);
        self.mount.push_str("\".as_ptr() as i32, ");
        self.mount.push_str(crate::vformat!("{}", text.len()).as_str());
        self.mount.push_str(");\n");
        self.needs_create_text = true;

        // Mount: append to parent
        self.mount.push_str("__volki_dom_append(");
        self.mount.push_str(parent_var);
        self.mount.push_str(", ");
        self.mount.push_str(var.as_str());
        self.mount.push_str(");\n");
        self.needs_append = true;
    }

    fn walk_expr(&mut self, expr: &str, parent_var: &str) {
        let slot = self.dyn_slot_counter;
        self.dyn_slot_counter += 1;
        let ref_slot = self.ref_slot_offset + slot;

        let n = self.node_counter;
        self.node_counter += 1;
        let var = crate::vformat!("__rd{}", n);

        // Mount: create empty text node placeholder
        self.mount.push_str("let ");
        self.mount.push_str(var.as_str());
        self.mount.push_str(" = __volki_dom_create_text(\"\".as_ptr() as i32, 0);\n");
        self.needs_create_text = true;

        // Mount: append to parent
        self.mount.push_str("__volki_dom_append(");
        self.mount.push_str(parent_var);
        self.mount.push_str(", ");
        self.mount.push_str(var.as_str());
        self.mount.push_str(");\n");
        self.needs_append = true;

        // Mount: store handle in ref slot
        self.mount.push_str("__volki_ref_set_i32(");
        self.mount.push_str(crate::vformat!("{}", ref_slot).as_str());
        self.mount.push_str(", ");
        self.mount.push_str(var.as_str());
        self.mount.push_str(");\n");
        self.needs_ref_set_i32 = true;

        // Update: retrieve handle and set text content
        let dyn_var = crate::vformat!("__dyn{}", slot);
        self.update.push_str("let ");
        self.update.push_str(dyn_var.as_str());
        self.update.push_str(" = __volki_ref_get_i32(");
        self.update.push_str(crate::vformat!("{}", ref_slot).as_str());
        self.update.push_str(");\n");
        self.needs_ref_get_i32 = true;

        // Generate update code based on expression type
        self.generate_expr_update(dyn_var.as_str(), expr, slot);
    }

    fn generate_expr_update(&mut self, handle_var: &str, expr: &str, slot: u32) {
        let expr = expr.trim();

        // Case 1: state::fmt_i32(var)
        if let Some(inner) = extract_fmt_call(expr, "state::fmt_i32(") {
            let fb = crate::vformat!("__rfb{}", slot);
            let fl = crate::vformat!("__rfl{}", slot);
            self.update.push_str("let ");
            self.update.push_str(fb.as_str());
            self.update.push_str(" = __volki_alloc(20);\n");
            self.update.push_str("let ");
            self.update.push_str(fl.as_str());
            self.update.push_str(" = __volki_state_fmt_i32(");
            self.update.push_str(inner);
            self.update.push_str(", ");
            self.update.push_str(fb.as_str());
            self.update.push_str(", 20);\n");
            self.update.push_str("__volki_dom_set_text(");
            self.update.push_str(handle_var);
            self.update.push_str(", ");
            self.update.push_str(fb.as_str());
            self.update.push_str(", ");
            self.update.push_str(fl.as_str());
            self.update.push_str(");\n");
            self.needs_set_text = true;
            self.needs_fmt_i32 = true;
            return;
        }

        // Case 2: state::fmt_f32(var)
        if let Some(inner) = extract_fmt_call(expr, "state::fmt_f32(") {
            let fb = crate::vformat!("__rfb{}", slot);
            let fl = crate::vformat!("__rfl{}", slot);
            self.update.push_str("let ");
            self.update.push_str(fb.as_str());
            self.update.push_str(" = __volki_alloc(20);\n");
            self.update.push_str("let ");
            self.update.push_str(fl.as_str());
            self.update.push_str(" = __volki_state_fmt_f32(");
            self.update.push_str(inner);
            self.update.push_str(", ");
            self.update.push_str(fb.as_str());
            self.update.push_str(", 20);\n");
            self.update.push_str("__volki_dom_set_text(");
            self.update.push_str(handle_var);
            self.update.push_str(", ");
            self.update.push_str(fb.as_str());
            self.update.push_str(", ");
            self.update.push_str(fl.as_str());
            self.update.push_str(");\n");
            self.needs_set_text = true;
            self.needs_fmt_f32 = true;
            return;
        }

        // Case 3: string literal "..."
        if expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2 {
            let inner = &expr[1..expr.len() - 1];
            self.update.push_str("__volki_dom_set_text(");
            self.update.push_str(handle_var);
            self.update.push_str(", \"");
            self.update.push_str(inner);
            self.update.push_str("\".as_ptr() as i32, ");
            self.update.push_str(crate::vformat!("{}", inner.len()).as_str());
            self.update.push_str(");\n");
            self.needs_set_text = true;
            return;
        }

        // Case 4: general expression — treat as &str
        self.update.push_str("__volki_dom_set_text(");
        self.update.push_str(handle_var);
        self.update.push_str(", (");
        self.update.push_str(expr);
        self.update.push_str(").as_ptr() as i32, (");
        self.update.push_str(expr);
        self.update.push_str(").len() as i32);\n");
        self.needs_set_text = true;
    }

    fn emit_set_attr(&mut self, var: &str, attr_name: &str, attr_value: &str) {
        self.mount.push_str("__volki_dom_set_attr(");
        self.mount.push_str(var);
        self.mount.push_str(", \"");
        self.mount.push_str(attr_name);
        self.mount.push_str("\".as_ptr() as i32, ");
        self.mount.push_str(crate::vformat!("{}", attr_name.len()).as_str());
        self.mount.push_str(", \"");
        self.mount.push_str(attr_value);
        self.mount.push_str("\".as_ptr() as i32, ");
        self.mount.push_str(crate::vformat!("{}", attr_value.len()).as_str());
        self.mount.push_str(");\n");
        self.needs_set_attr = true;
    }
}

/// Extract the inner argument from a `state::fmt_i32(arg)` or `state::fmt_f32(arg)` call.
fn extract_fmt_call<'a>(expr: &'a str, prefix: &str) -> Option<&'a str> {
    if !expr.starts_with(prefix) {
        return None;
    }
    let inner_start = prefix.len();
    // Find matching closing paren
    let bytes = expr.as_bytes();
    let mut depth = 1;
    let mut i = inner_start;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(expr[inner_start..i].trim());
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn is_event_attr(name: &str) -> bool {
    name.starts_with("on") && name.len() > 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::collections::Vec;
    use crate::libs::web::compiler::parser::{RsxAttr, RsxAttrValue, RsxNode};
    use crate::vvec;

    fn s(v: &str) -> String { String::from(v) }

    #[test]
    fn test_rsx_static_element() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("counter")) }],
            children: vvec![RsxNode::Text(s("hello"))],
            self_closing: false,
        }];
        let output = generate_component_rsx(&nodes, 0, 0);

        assert!(output.mount_code.contains("__volki_dom_create(\"div\""));
        assert!(output.mount_code.contains("__volki_dom_add_class("));
        assert!(output.mount_code.contains("\"counter\""));
        assert!(output.mount_code.contains("__volki_dom_create_text(\"hello\""));
        assert!(output.mount_code.contains("__volki_dom_append("));
        assert!(output.update_code.is_empty());
        assert_eq!(output.ref_slots_used, 0);
        assert!(output.needs_create);
        assert!(output.needs_create_text);
        assert!(output.needs_append);
        assert!(output.needs_add_class);
    }

    #[test]
    fn test_rsx_dynamic_expression() {
        let nodes = vvec![RsxNode::Element {
            tag: s("span"),
            attrs: Vec::new(),
            children: vvec![RsxNode::Expr(s("state::fmt_i32(count)"))],
            self_closing: false,
        }];
        let output = generate_component_rsx(&nodes, 0, 0);

        // Mount should create empty text placeholder and store ref
        assert!(output.mount_code.contains("__volki_dom_create_text(\"\".as_ptr()"));
        assert!(output.mount_code.contains("__volki_ref_set_i32(0,"));

        // Update should get ref, alloc, fmt, set_text
        assert!(output.update_code.contains("__volki_ref_get_i32(0)"));
        assert!(output.update_code.contains("__volki_alloc(20)"));
        assert!(output.update_code.contains("__volki_state_fmt_i32(count,"));
        assert!(output.update_code.contains("__volki_dom_set_text("));

        assert_eq!(output.ref_slots_used, 1);
        assert!(output.needs_fmt_i32);
        assert!(output.needs_set_text);
    }

    #[test]
    fn test_rsx_event_attr() {
        let nodes = vvec![RsxNode::Element {
            tag: s("button"),
            attrs: vvec![RsxAttr { name: s("onclick"), value: RsxAttrValue::Expr(s("on_click")) }],
            children: vvec![RsxNode::Text(s("+"))],
            self_closing: false,
        }];
        let output = generate_component_rsx(&nodes, 0, 0);

        assert!(output.mount_code.contains("__volki_dom_set_attr("));
        assert!(output.mount_code.contains("data-volki-onclick"));
        assert!(output.mount_code.contains("on_click"));
    }

    #[test]
    fn test_rsx_ref_slot_offset() {
        let nodes = vvec![RsxNode::Element {
            tag: s("span"),
            attrs: Vec::new(),
            children: vvec![RsxNode::Expr(s("state::fmt_i32(count)"))],
            self_closing: false,
        }];
        // User has 3 use_ref calls, so RSX refs start at slot 3
        let output = generate_component_rsx(&nodes, 0, 3);

        assert!(output.mount_code.contains("__volki_ref_set_i32(3,"));
        assert!(output.update_code.contains("__volki_ref_get_i32(3)"));
    }

    #[test]
    fn test_rsx_string_literal_expr() {
        let nodes = vvec![RsxNode::Element {
            tag: s("span"),
            attrs: Vec::new(),
            children: vvec![RsxNode::Expr(s("\"hello world\""))],
            self_closing: false,
        }];
        let output = generate_component_rsx(&nodes, 0, 0);

        // String literal update should pass the literal directly
        assert!(output.update_code.contains("\"hello world\".as_ptr()"));
    }

    #[test]
    fn test_rsx_multiple_dynamic_slots() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: Vec::new(),
            children: vvec![
                RsxNode::Expr(s("state::fmt_i32(a)")),
                RsxNode::Text(s(" + ")),
                RsxNode::Expr(s("state::fmt_i32(b)"))
            ],
            self_closing: false,
        }];
        let output = generate_component_rsx(&nodes, 0, 0);

        assert_eq!(output.ref_slots_used, 2);
        assert!(output.mount_code.contains("__volki_ref_set_i32(0,"));
        assert!(output.mount_code.contains("__volki_ref_set_i32(1,"));
        assert!(output.update_code.contains("__volki_ref_get_i32(0)"));
        assert!(output.update_code.contains("__volki_ref_get_i32(1)"));
    }
}
