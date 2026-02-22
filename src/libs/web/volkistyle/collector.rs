//! Class collector — walks RsxNode AST and extracts all class attribute values.

use crate::core::volkiwithstds::collections::{String, Vec};
use crate::libs::web::compiler::parser::{RsxAttrValue, RsxNode};

/// Walk all nodes and collect individual class names from `class` attributes.
/// Class values are split on whitespace so `"flex p-4"` yields `["flex", "p-4"]`.
pub fn collect_classes(nodes: &[RsxNode]) -> Vec<String> {
    let mut classes = Vec::new();
    for node in nodes {
        collect_from_node(node, &mut classes);
    }
    classes
}

fn collect_from_node(node: &RsxNode, out: &mut Vec<String>) {
    match node {
        RsxNode::Element { attrs, children, .. } => {
            for attr in attrs.iter() {
                if attr.name.as_str() == "class" {
                    if let RsxAttrValue::Literal(v) = &attr.value {
                        for part in v.as_str().split_whitespace() {
                            out.push(String::from(part));
                        }
                    }
                }
            }
            for child in children.iter() {
                collect_from_node(child, out);
            }
        }
        RsxNode::CondAnd { body, .. } => {
            for node in body.iter() {
                collect_from_node(node, out);
            }
        }
        RsxNode::Ternary { if_true, if_false, .. } => {
            for node in if_true.iter() {
                collect_from_node(node, out);
            }
            for node in if_false.iter() {
                collect_from_node(node, out);
            }
        }
        RsxNode::Text(_) | RsxNode::Expr(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::web::compiler::parser::{RsxAttr, RsxAttrValue};
    use crate::vvec;

    fn s(v: &str) -> String {
        String::from(v)
    }

    fn empty_nodes() -> Vec<RsxNode> {
        Vec::new()
    }

    #[test]
    fn test_collect_single_class() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("flex")) }],
            children: empty_nodes(),
            self_closing: false,
        }];
        let classes = collect_classes(&nodes);
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].as_str(), "flex");
    }

    #[test]
    fn test_collect_multiple_classes() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("flex items-center p-4")) }],
            children: empty_nodes(),
            self_closing: false,
        }];
        let classes = collect_classes(&nodes);
        assert_eq!(classes.len(), 3);
        assert_eq!(classes[0].as_str(), "flex");
        assert_eq!(classes[1].as_str(), "items-center");
        assert_eq!(classes[2].as_str(), "p-4");
    }

    #[test]
    fn test_collect_from_nested() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("outer")) }],
            children: vvec![RsxNode::Element {
                tag: s("span"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("inner text-sm")) }],
                children: empty_nodes(),
                self_closing: false,
            }],
            self_closing: false,
        }];
        let classes = collect_classes(&nodes);
        assert_eq!(classes.len(), 3);
        assert_eq!(classes[0].as_str(), "outer");
        assert_eq!(classes[1].as_str(), "inner");
        assert_eq!(classes[2].as_str(), "text-sm");
    }

    #[test]
    fn test_collect_ignores_non_class_attrs() {
        let nodes = vvec![RsxNode::Element {
            tag: s("div"),
            attrs: vvec![
                RsxAttr { name: s("id"), value: RsxAttrValue::Literal(s("main")) },
                RsxAttr { name: s("data-x"), value: RsxAttrValue::Literal(s("y")) },
            ],
            children: empty_nodes(),
            self_closing: false,
        }];
        let classes = collect_classes(&nodes);
        assert!(classes.is_empty());
    }

    #[test]
    fn test_collect_from_text_and_expr_nodes() {
        let nodes = vvec![
            RsxNode::Text(s("hello")),
            RsxNode::Expr(s("content()")),
        ];
        let classes = collect_classes(&nodes);
        assert!(classes.is_empty());
    }

    #[test]
    fn test_collect_from_multiple_elements() {
        let nodes = vvec![
            RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("flex")) }],
                children: empty_nodes(),
                self_closing: false,
            },
            RsxNode::Element {
                tag: s("span"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("grid")) }],
                children: empty_nodes(),
                self_closing: false,
            },
        ];
        let classes = collect_classes(&nodes);
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0].as_str(), "flex");
        assert_eq!(classes[1].as_str(), "grid");
    }

    // ── Conditional rendering collector tests ──

    #[test]
    fn test_collect_from_cond_and() {
        let nodes = vvec![RsxNode::CondAnd {
            condition: s("show"),
            body: vvec![RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("hidden-panel")) }],
                children: empty_nodes(),
                self_closing: false,
            }],
        }];
        let classes = collect_classes(&nodes);
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].as_str(), "hidden-panel");
    }

    #[test]
    fn test_collect_from_ternary_both_branches() {
        let nodes = vvec![RsxNode::Ternary {
            condition: s("dark"),
            if_true: vvec![RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("bg-dark text-white")) }],
                children: empty_nodes(),
                self_closing: false,
            }],
            if_false: vvec![RsxNode::Element {
                tag: s("div"),
                attrs: vvec![RsxAttr { name: s("class"), value: RsxAttrValue::Literal(s("bg-light text-black")) }],
                children: empty_nodes(),
                self_closing: false,
            }],
        }];
        let classes = collect_classes(&nodes);
        assert_eq!(classes.len(), 4);
        assert_eq!(classes[0].as_str(), "bg-dark");
        assert_eq!(classes[1].as_str(), "text-white");
        assert_eq!(classes[2].as_str(), "bg-light");
        assert_eq!(classes[3].as_str(), "text-black");
    }
}
