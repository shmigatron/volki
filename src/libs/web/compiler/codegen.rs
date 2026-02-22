//! Code Generator — transforms AST nodes into Rust source code.

use crate::core::volkiwithstds::collections::String;

use super::parser::{RsxAttr, RsxAttrValue, RsxNode};

/// Generate code for a `-> Html` function body.
/// Produces `HtmlDocument::new().inline_style(...).body_node(...)...` chain.
pub fn generate_html_fn(nodes: &[RsxNode]) -> String {
    generate_html_fn_with_client(nodes, None)
}

/// Generate code for a `-> Html` function body, optionally injecting a client glue script.
/// If `glue_url` is Some, appends `.script_module("url")` referencing a statically served JS file.
pub fn generate_html_fn_with_client(nodes: &[RsxNode], glue_url: Option<&str>) -> String {
    let mut out = String::from("HtmlDocument::new()\n");

    for node in nodes {
        match node {
            // Special <Style>{expr}</Style> element
            RsxNode::Element { tag, children, self_closing: false, .. }
                if is_special_tag(tag.as_str(), "Style") =>
            {
                if let Some(RsxNode::Expr(expr)) = children.first() {
                    out.push_str("        .inline_style(");
                    out.push_str(expr.as_str());
                    out.push_str(")\n");
                }
            }
            // Special <Head>...</Head> element
            RsxNode::Element { tag, children, self_closing: false, .. }
                if is_special_tag(tag.as_str(), "Head") =>
            {
                for child in children {
                    out.push_str("        .head_node(\n");
                    out.push_str("            ");
                    generate_node(child, &mut out, 3);
                    out.push_str("\n        )\n");
                }
            }
            // Special <Stylesheet href="..." /> element
            RsxNode::Element { tag, attrs, self_closing: true, .. }
                if is_special_tag(tag.as_str(), "Stylesheet") =>
            {
                if let Some(href) = find_attr(attrs, "href") {
                    out.push_str("        .stylesheet(\"");
                    out.push_str(href.as_str());
                    out.push_str("\")\n");
                }
            }
            // Conditional render
            RsxNode::CondAnd { condition, body } => {
                out.push_str("        .body_nodes(");
                generate_cond_and_vec(condition.as_str(), body, &mut out);
                out.push_str(")\n");
            }
            RsxNode::Ternary { condition, if_true, if_false } => {
                if if_true.len() == 1 && if_false.len() == 1 {
                    out.push_str("        .body_node(\n");
                    out.push_str("            ");
                    generate_ternary_single(condition.as_str(), &if_true[0], &if_false[0], &mut out);
                    out.push_str("\n        )\n");
                } else {
                    out.push_str("        .body_nodes(");
                    generate_ternary_vec(condition.as_str(), if_true, if_false, &mut out);
                    out.push_str(")\n");
                }
            }
            // Top-level expression (e.g. component function call) -> body_nodes
            RsxNode::Expr(expr) => {
                out.push_str("        .body_nodes((");
                out.push_str(expr.as_str());
                out.push_str(").into_children())\n");
            }
            // Regular top-level elements -> body_node
            _ => {
                out.push_str("        .body_node(\n");
                out.push_str("            ");
                generate_node(node, &mut out, 3);
                out.push_str("\n        )\n");
            }
        }
    }

    // Reference client-side glue script as a static module
    if let Some(url) = glue_url {
        out.push_str("        .script_module(\"");
        out.push_str(url);
        out.push_str("\")\n");
    }

    out
}

/// Generate code for a `-> Fragment` function body.
/// Produces `let mut __rsx_nodes = Vec::new(); ... __rsx_nodes`
pub fn generate_fragment_fn(nodes: &[RsxNode]) -> String {
    let mut out = String::from("let mut __rsx_nodes = Vec::new();\n");

    for node in nodes {
        match node {
            RsxNode::Expr(expr) => {
                // Top-level expressions extend (could return Vec or single node)
                out.push_str("    __rsx_nodes.extend((");
                out.push_str(expr.as_str());
                out.push_str(").into_children());\n");
            }
            RsxNode::CondAnd { condition, body } => {
                out.push_str("    if ");
                out.push_str(condition.as_str());
                out.push_str(" {\n");
                for child in body {
                    out.push_str("        __rsx_nodes.push(");
                    generate_node(child, &mut out, 2);
                    out.push_str(");\n");
                }
                out.push_str("    }\n");
            }
            RsxNode::Ternary { condition, if_true, if_false } => {
                if if_true.len() == 1 && if_false.len() == 1 {
                    out.push_str("    __rsx_nodes.push(if ");
                    out.push_str(condition.as_str());
                    out.push_str(" { ");
                    generate_node(&if_true[0], &mut out, 2);
                    out.push_str(" } else { ");
                    generate_node(&if_false[0], &mut out, 2);
                    out.push_str(" });\n");
                } else {
                    out.push_str("    if ");
                    out.push_str(condition.as_str());
                    out.push_str(" {\n");
                    for child in if_true {
                        out.push_str("        __rsx_nodes.push(");
                        generate_node(child, &mut out, 2);
                        out.push_str(");\n");
                    }
                    out.push_str("    } else {\n");
                    for child in if_false {
                        out.push_str("        __rsx_nodes.push(");
                        generate_node(child, &mut out, 2);
                        out.push_str(");\n");
                    }
                    out.push_str("    }\n");
                }
            }
            _ => {
                out.push_str("    __rsx_nodes.push(\n        ");
                generate_node(node, &mut out, 2);
                out.push_str("\n    );\n");
            }
        }
    }

    out.push_str("    __rsx_nodes");
    out
}

/// Generate a block expression that builds a `Vec<HtmlNode>` from child nodes.
/// Used to compile component tag children into a function argument.
pub fn generate_children_expr(nodes: &[RsxNode]) -> String {
    let mut out = String::from("{ let mut __c = Vec::new(); ");
    for node in nodes {
        match node {
            RsxNode::Expr(expr) => {
                out.push_str("__c.extend((");
                out.push_str(expr.as_str());
                out.push_str(").into_children()); ");
            }
            RsxNode::CondAnd { condition, body } => {
                out.push_str("if ");
                out.push_str(condition.as_str());
                out.push_str(" { ");
                for child in body {
                    out.push_str("__c.push(");
                    generate_node(child, &mut out, 0);
                    out.push_str("); ");
                }
                out.push_str("} ");
            }
            RsxNode::Ternary { condition, if_true, if_false } => {
                out.push_str("if ");
                out.push_str(condition.as_str());
                out.push_str(" { ");
                for child in if_true {
                    out.push_str("__c.push(");
                    generate_node(child, &mut out, 0);
                    out.push_str("); ");
                }
                out.push_str("} else { ");
                for child in if_false {
                    out.push_str("__c.push(");
                    generate_node(child, &mut out, 0);
                    out.push_str("); ");
                }
                out.push_str("} ");
            }
            _ => {
                out.push_str("__c.push(");
                generate_node(node, &mut out, 0);
                out.push_str("); ");
            }
        }
    }
    out.push_str("__c }");
    out
}

fn is_special_tag(tag: &str, expected: &str) -> bool {
    tag == expected
}

/// Find an attribute value by name from a list of RSX attributes.
fn find_attr<'a>(attrs: &'a [RsxAttr], name: &str) -> Option<&'a String> {
    for attr in attrs {
        if attr.name.as_str() == name {
            if let RsxAttrValue::Literal(v) = &attr.value {
                return Some(v);
            }
        }
    }
    None
}

/// Generate Rust code for a single RSX node.
fn generate_node(node: &RsxNode, out: &mut String, _depth: usize) {
    match node {
        RsxNode::Element { tag, attrs, children, self_closing } => {
            generate_element(tag.as_str(), attrs, children, *self_closing, out);
        }
        RsxNode::Text(text) => {
            out.push_str("text(\"");
            out.push_str(text.as_str());
            out.push_str("\")");
        }
        RsxNode::Expr(expr) => {
            out.push_str("(");
            out.push_str(expr.as_str());
            out.push_str(").into_children()");
        }
        RsxNode::CondAnd { condition, body } => {
            generate_cond_and_vec(condition.as_str(), body, out);
        }
        RsxNode::Ternary { condition, if_true, if_false } => {
            if if_true.len() == 1 && if_false.len() == 1 {
                generate_ternary_single(condition.as_str(), &if_true[0], &if_false[0], out);
            } else {
                generate_ternary_vec(condition.as_str(), if_true, if_false, out);
            }
        }
    }
}

/// Generate a block expression that conditionally pushes nodes into a Vec.
fn generate_cond_and_vec(condition: &str, body: &[RsxNode], out: &mut String) {
    out.push_str("{ let mut __c = Vec::new(); if ");
    out.push_str(condition);
    out.push_str(" { ");
    for node in body {
        out.push_str("__c.push(");
        generate_node(node, out, 0);
        out.push_str("); ");
    }
    out.push_str("} __c }");
}

/// Generate an if/else expression that produces a single node.
fn generate_ternary_single(condition: &str, if_true: &RsxNode, if_false: &RsxNode, out: &mut String) {
    out.push_str("if ");
    out.push_str(condition);
    out.push_str(" { ");
    generate_node(if_true, out, 0);
    out.push_str(" } else { ");
    generate_node(if_false, out, 0);
    out.push_str(" }");
}

/// Generate an if/else expression where each branch builds a Vec of nodes.
fn generate_ternary_vec(condition: &str, if_true: &[RsxNode], if_false: &[RsxNode], out: &mut String) {
    out.push_str("if ");
    out.push_str(condition);
    out.push_str(" { let mut __t = Vec::new(); ");
    for node in if_true {
        out.push_str("__t.push(");
        generate_node(node, out, 0);
        out.push_str("); ");
    }
    out.push_str("__t } else { let mut __f = Vec::new(); ");
    for node in if_false {
        out.push_str("__f.push(");
        generate_node(node, out, 0);
        out.push_str("); ");
    }
    out.push_str("__f }");
}

fn generate_element(
    tag: &str,
    attrs: &[RsxAttr],
    children: &[RsxNode],
    _self_closing: bool,
    out: &mut String,
) {
    // Element constructor
    out.push_str(tag);
    out.push_str("()");

    // Attributes
    for attr in attrs {
        let name = attr.name.as_str();
        match &attr.value {
            RsxAttrValue::Literal(value) => match name {
                "class" => {
                    out.push_str(".class(\"");
                    out.push_str(value.as_str());
                    out.push_str("\")");
                }
                "id" => {
                    out.push_str(".id(\"");
                    out.push_str(value.as_str());
                    out.push_str("\")");
                }
                _ => {
                    out.push_str(".attr(\"");
                    out.push_str(name);
                    out.push_str("\", \"");
                    out.push_str(value.as_str());
                    out.push_str("\")");
                }
            },
            RsxAttrValue::Expr(expr) => {
                // Event handler expressions are lowered to data attributes for JS auto-binding.
                if is_event_attr(name) {
                    out.push_str(".attr(\"data-volki-");
                    out.push_str(name);
                    out.push_str("\", \"");
                    out.push_str(expr.as_str());
                    out.push_str("\")");
                }
            }
        }
    }

    // Children
    for child in children {
        match child {
            RsxNode::Text(text) => {
                out.push_str(".text(\"");
                out.push_str(text.as_str());
                out.push_str("\")");
            }
            RsxNode::Expr(expr) => {
                out.push_str(".children((");
                out.push_str(expr.as_str());
                out.push_str(").into_children())");
            }
            RsxNode::Element { tag, attrs, children, self_closing } => {
                out.push_str(".child(");
                generate_element(tag.as_str(), attrs, children, *self_closing, out);
                out.push_str(")");
            }
            RsxNode::CondAnd { condition, body } => {
                out.push_str(".children(");
                generate_cond_and_vec(condition.as_str(), body, out);
                out.push_str(")");
            }
            RsxNode::Ternary { condition, if_true, if_false } => {
                if if_true.len() == 1 && if_false.len() == 1 {
                    out.push_str(".child(");
                    generate_ternary_single(condition.as_str(), &if_true[0], &if_false[0], out);
                    out.push_str(")");
                } else {
                    out.push_str(".children(");
                    generate_ternary_vec(condition.as_str(), if_true, if_false, out);
                    out.push_str(")");
                }
            }
        }
    }

    out.push_str(".into_node()");
}

fn is_event_attr(name: &str) -> bool {
    name.starts_with("on") && name.len() > 2
}

/// Generate code for a `-> Html` function body with utility CSS injected.
/// Collects all utility classes, generates CSS, and injects `.inline_style("...")`.
pub fn generate_html_fn_styled(nodes: &[RsxNode], css: &str, glue_url: Option<&str>) -> String {
    let mut out = String::from("HtmlDocument::new()\n");

    // Inject generated utility CSS as inline style
    if !css.is_empty() {
        out.push_str("        .inline_style(\"");
        // Escape any double quotes in the CSS (shouldn't happen with our output, but be safe)
        for ch in css.chars() {
            if ch == '"' {
                out.push_str("\\\"");
            } else if ch == '\\' {
                out.push_str("\\\\");
            } else {
                out.push(ch);
            }
        }
        out.push_str("\")\n");
    }

    for node in nodes {
        match node {
            // Special <Style>{expr}</Style> element
            RsxNode::Element { tag, children, self_closing: false, .. }
                if is_special_tag(tag.as_str(), "Style") =>
            {
                if let Some(RsxNode::Expr(expr)) = children.first() {
                    out.push_str("        .inline_style(");
                    out.push_str(expr.as_str());
                    out.push_str(")\n");
                }
            }
            // Special <Head>...</Head> element
            RsxNode::Element { tag, children, self_closing: false, .. }
                if is_special_tag(tag.as_str(), "Head") =>
            {
                for child in children {
                    out.push_str("        .head_node(\n");
                    out.push_str("            ");
                    generate_node(child, &mut out, 3);
                    out.push_str("\n        )\n");
                }
            }
            // Special <Stylesheet href="..." /> element
            RsxNode::Element { tag, attrs, self_closing: true, .. }
                if is_special_tag(tag.as_str(), "Stylesheet") =>
            {
                if let Some(href) = find_attr(attrs, "href") {
                    out.push_str("        .stylesheet(\"");
                    out.push_str(href.as_str());
                    out.push_str("\")\n");
                }
            }
            // Conditional render
            RsxNode::CondAnd { condition, body } => {
                out.push_str("        .body_nodes(");
                generate_cond_and_vec(condition.as_str(), body, &mut out);
                out.push_str(")\n");
            }
            RsxNode::Ternary { condition, if_true, if_false } => {
                if if_true.len() == 1 && if_false.len() == 1 {
                    out.push_str("        .body_node(\n");
                    out.push_str("            ");
                    generate_ternary_single(condition.as_str(), &if_true[0], &if_false[0], &mut out);
                    out.push_str("\n        )\n");
                } else {
                    out.push_str("        .body_nodes(");
                    generate_ternary_vec(condition.as_str(), if_true, if_false, &mut out);
                    out.push_str(")\n");
                }
            }
            // Top-level expression (e.g. component function call) -> body_nodes
            RsxNode::Expr(expr) => {
                out.push_str("        .body_nodes((");
                out.push_str(expr.as_str());
                out.push_str(").into_children())\n");
            }
            // Regular top-level elements -> body_node
            _ => {
                out.push_str("        .body_node(\n");
                out.push_str("            ");
                generate_node(node, &mut out, 3);
                out.push_str("\n        )\n");
            }
        }
    }

    // Reference client-side glue script as a static module
    if let Some(url) = glue_url {
        out.push_str("        .script_module(\"");
        out.push_str(url);
        out.push_str("\")\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::web::compiler::parser::{RsxAttr, RsxAttrValue, RsxNode};
    use crate::vvec;

    fn s(val: &str) -> String {
        String::from(val)
    }

    fn empty_nodes() -> crate::core::volkiwithstds::collections::Vec<RsxNode> {
        crate::core::volkiwithstds::collections::Vec::new()
    }

    fn empty_attrs() -> crate::core::volkiwithstds::collections::Vec<RsxAttr> {
        crate::core::volkiwithstds::collections::Vec::new()
    }

    #[test]
    fn test_codegen_simple_div() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("foo")) }],
            children: vvec![RsxNode::Text(s("hello"))],
            self_closing: false,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains("div().class(\"foo\").text(\"hello\").into_node()"));
    }

    #[test]
    fn test_codegen_nested() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: empty_attrs(),
            children: vvec![RsxNode::Element {
                tag: s("span"),
                attrs: empty_attrs(),
                children: vvec![RsxNode::Text(s("inner"))],
                self_closing: false,
            }],
            self_closing: false,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains("div().child(span().text(\"inner\").into_node()).into_node()"));
    }

    #[test]
    fn test_codegen_html_fn() {
        let nodes = vvec![
            RsxNode::Element {
                tag: s("Style"),
                attrs: empty_attrs(),
                children: vvec![RsxNode::Expr(s("CSS"))],
                self_closing: false,
            },
            RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("main")) }],
                children: empty_nodes(),
                self_closing: false,
            },
        ];
        let code = generate_html_fn(&nodes);
        assert!(code.contains("HtmlDocument::new()"));
        assert!(code.contains(".inline_style(CSS)"));
        assert!(code.contains(".body_node("));
        assert!(code.contains("div().class(\"main\").into_node()"));
    }

    #[test]
    fn test_codegen_fragment_fn() {
        let nodes = vvec![
            RsxNode::Element {
                tag: s("div"),
                attrs: empty_attrs(),
                children: vvec![RsxNode::Text(s("one"))],
                self_closing: false,
            },
            RsxNode::Element {
                tag: s("span"),
                attrs: empty_attrs(),
                children: vvec![RsxNode::Text(s("two"))],
                self_closing: false,
            },
        ];
        let code = generate_fragment_fn(&nodes);
        assert!(code.contains("let mut __rsx_nodes = Vec::new();"));
        assert!(code.contains("__rsx_nodes.push("));
        assert!(code.contains("div().text(\"one\").into_node()"));
        assert!(code.contains("span().text(\"two\").into_node()"));
        assert!(code.contains("__rsx_nodes"));
    }

    #[test]
    fn test_codegen_style_element() {
        let nodes = vvec![RsxNode::Element {
            tag: s("Style"),
            attrs: empty_attrs(),
            children: vvec![RsxNode::Expr(s("CSS"))],
            self_closing: false,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains(".inline_style(CSS)"));
        assert!(!code.contains(".body_node("));
    }

    #[test]
    fn test_codegen_expression_child() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: empty_attrs(),
            children: vvec![RsxNode::Expr(s("sidebar_content()"))],
            self_closing: false,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains(".children((sidebar_content()).into_children())"));
    }

    #[test]
    fn test_codegen_self_closing() {
        let nodes = vvec![RsxNode::Element {
            tag: s("br"),
            attrs: empty_attrs(),
            children: empty_nodes(),
            self_closing: true,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains("br().into_node()"));
    }

    #[test]
    fn test_codegen_attr_shorthand() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: vvec![
                RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("container")) },
                RsxAttr { name: s("id"), value: RsxAttrValue::Literal(s("main")) },
                RsxAttr { name: s("data-x"), value: RsxAttrValue::Literal(s("y")) },
            ],
            children: empty_nodes(),
            self_closing: false,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains(".class(\"container\")"));
        assert!(code.contains(".id(\"main\")"));
        assert!(code.contains(".attr(\"data-x\", \"y\")"));
    }

    #[test]
    fn test_codegen_event_expr_attr_to_data_binding() {
        let nodes = vvec![RsxNode::Element {
            tag: s("button"),
            attrs: vvec![
                RsxAttr { name: s("onclick"), value: RsxAttrValue::Expr(s("on_increment")) },
            ],
            children: vvec![RsxNode::Text(s("+"))],
            self_closing: false,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains("button().attr(\"data-volki-onclick\", \"on_increment\")"));
    }

    #[test]
    fn test_codegen_head_element() {
        let nodes = vvec![RsxNode::Element {
            tag: s("Head"),
            attrs: empty_attrs(),
            children: vvec![RsxNode::Element {
                tag: s("meta"),
                attrs: vvec![RsxAttr { name: s("charset"), value: RsxAttrValue::Literal(s("utf-8")) }],
                children: empty_nodes(),
                self_closing: true,
            }],
            self_closing: false,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains(".head_node("));
        assert!(code.contains("meta().attr(\"charset\", \"utf-8\").into_node()"));
    }

    #[test]
    fn test_codegen_styled_injects_css() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("flex")) }],
            children: vvec![RsxNode::Text(s("hello"))],
            self_closing: false,
        }];
        let css = ".flex{display:flex;}";
        let code = generate_html_fn_styled(&nodes, css, None);
        assert!(code.contains("HtmlDocument::new()"));
        assert!(code.contains(".inline_style(\".flex{display:flex;}\")"));
        assert!(code.contains("div().class(\"flex\").text(\"hello\").into_node()"));
    }

    #[test]
    fn test_codegen_styled_empty_css() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: empty_attrs(),
            children: vvec![RsxNode::Text(s("hello"))],
            self_closing: false,
        }];
        let code = generate_html_fn_styled(&nodes, "", None);
        assert!(code.contains("HtmlDocument::new()"));
        assert!(!code.contains(".inline_style("));
    }

    #[test]
    fn test_codegen_styled_with_glue() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("flex")) }],
            children: empty_nodes(),
            self_closing: false,
        }];
        let css = ".flex{display:flex;}";
        let glue_url = "/wasm/page_glue.js";
        let code = generate_html_fn_styled(&nodes, css, Some(glue_url));
        assert!(code.contains(".inline_style(\".flex{display:flex;}\")"));
        assert!(code.contains(".script_module(\"/wasm/page_glue.js\")"));
    }

    #[test]
    fn test_codegen_styled_with_existing_style() {
        let nodes = vvec![
            RsxNode::Element {
                tag: s("Style"),
                attrs: empty_attrs(),
                children: vvec![RsxNode::Expr(s("CSS"))],
                self_closing: false,
            },
            RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("flex")) }],
                children: empty_nodes(),
                self_closing: false,
            },
        ];
        let css = ".flex{display:flex;}";
        let code = generate_html_fn_styled(&nodes, css, None);
        // Utility CSS should come first (before body nodes and user styles)
        let utility_pos = code.as_str().find(".inline_style(\".flex").unwrap();
        let user_pos = code.as_str().find(".inline_style(CSS)").unwrap();
        assert!(utility_pos < user_pos);
    }

    // ── Conditional rendering codegen tests ──

    #[test]
    fn test_codegen_cond_and_in_fragment() {
        let nodes = vvec![RsxNode::CondAnd {
            condition: s("is_admin"),
            body: vvec![RsxNode::Element {
                tag: s("span"),
                attrs: empty_attrs(),
                children: vvec![RsxNode::Text(s("Admin"))],
                self_closing: false,
            }],
        }];
        let code = generate_fragment_fn(&nodes);
        assert!(code.contains("if is_admin {"));
        assert!(code.contains("__rsx_nodes.push("));
        assert!(code.contains("span().text(\"Admin\").into_node()"));
    }

    #[test]
    fn test_codegen_ternary_in_fragment() {
        let nodes = vvec![RsxNode::Ternary {
            condition: s("flag"),
            if_true: vvec![RsxNode::Element {
                tag: s("span"),
                attrs: empty_attrs(),
                children: vvec![RsxNode::Text(s("yes"))],
                self_closing: false,
            }],
            if_false: vvec![RsxNode::Element {
                tag: s("span"),
                attrs: empty_attrs(),
                children: vvec![RsxNode::Text(s("no"))],
                self_closing: false,
            }],
        }];
        let code = generate_fragment_fn(&nodes);
        assert!(code.contains("__rsx_nodes.push(if flag {"));
        assert!(code.contains("span().text(\"yes\").into_node()"));
        assert!(code.contains("} else {"));
        assert!(code.contains("span().text(\"no\").into_node()"));
    }

    #[test]
    fn test_codegen_cond_and_as_element_child() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: empty_attrs(),
            children: vvec![RsxNode::CondAnd {
                condition: s("show"),
                body: vvec![RsxNode::Element {
                    tag: s("span"),
                    attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("flex")) }],
                    children: vvec![RsxNode::Text(s("hello"))],
                    self_closing: false,
                }],
            }],
            self_closing: false,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains(".children("));
        assert!(code.contains("if show {"));
        assert!(code.contains("__c.push("));
        assert!(code.contains("span().class(\"flex\").text(\"hello\").into_node()"));
    }

    #[test]
    fn test_codegen_ternary_as_element_child() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: empty_attrs(),
            children: vvec![RsxNode::Ternary {
                condition: s("active"),
                if_true: vvec![RsxNode::Element {
                    tag: s("b"),
                    attrs: empty_attrs(),
                    children: vvec![RsxNode::Text(s("on"))],
                    self_closing: false,
                }],
                if_false: vvec![RsxNode::Element {
                    tag: s("i"),
                    attrs: empty_attrs(),
                    children: vvec![RsxNode::Text(s("off"))],
                    self_closing: false,
                }],
            }],
            self_closing: false,
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains(".child(if active {"));
        assert!(code.contains("b().text(\"on\").into_node()"));
        assert!(code.contains("} else {"));
        assert!(code.contains("i().text(\"off\").into_node()"));
    }

    #[test]
    fn test_codegen_cond_and_in_html_toplevel() {
        let nodes = vvec![RsxNode::CondAnd {
            condition: s("show_banner"),
            body: vvec![RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("banner")) }],
                children: vvec![RsxNode::Text(s("Welcome"))],
                self_closing: false,
            }],
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains(".body_nodes("));
        assert!(code.contains("if show_banner {"));
        assert!(code.contains("__c.push("));
        assert!(code.contains("div().class(\"banner\").text(\"Welcome\").into_node()"));
    }

    #[test]
    fn test_codegen_ternary_in_html_toplevel() {
        let nodes = vvec![RsxNode::Ternary {
            condition: s("dark"),
            if_true: vvec![RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("dark")) }],
                children: empty_nodes(),
                self_closing: false,
            }],
            if_false: vvec![RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("light")) }],
                children: empty_nodes(),
                self_closing: false,
            }],
        }];
        let code = generate_html_fn(&nodes);
        assert!(code.contains(".body_node("));
        assert!(code.contains("if dark {"));
        assert!(code.contains("div().class(\"dark\").into_node()"));
        assert!(code.contains("} else {"));
        assert!(code.contains("div().class(\"light\").into_node()"));
    }

    // ── Stylesheet codegen tests ──

    #[test]
    fn test_codegen_stylesheet_in_html_fn() {
        let nodes = vvec![
            RsxNode::Element {
                tag: s("Stylesheet"),
                attrs: vvec![RsxAttr { name: s("href"), value: RsxAttrValue::Literal(s("/styles/app.css")) }],
                children: empty_nodes(),
                self_closing: true,
            },
            RsxNode::Element {
                tag: s("div"),
                attrs: empty_attrs(),
                children: vvec![RsxNode::Text(s("hello"))],
                self_closing: false,
            },
        ];
        let code = generate_html_fn(&nodes);
        assert!(code.contains(".stylesheet(\"/styles/app.css\")"));
        assert!(code.contains("div().text(\"hello\").into_node()"));
    }

    #[test]
    fn test_codegen_stylesheet_in_styled_fn() {
        let nodes = vvec![
            RsxNode::Element {
                tag: s("Stylesheet"),
                attrs: vvec![RsxAttr { name: s("href"), value: RsxAttrValue::Literal(s("/fonts/inter.css")) }],
                children: empty_nodes(),
                self_closing: true,
            },
            RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("flex")) }],
                children: empty_nodes(),
                self_closing: false,
            },
        ];
        let css = ".flex{display:flex;}";
        let code = generate_html_fn_styled(&nodes, css, None);
        assert!(code.contains(".stylesheet(\"/fonts/inter.css\")"));
        assert!(code.contains(".inline_style(\".flex{display:flex;}\")"));
        assert!(code.contains("div().class(\"flex\").into_node()"));
    }
}
