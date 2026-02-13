use crate::libs::lang::js::formatter::tokenizer::{Token, TokenKind, tokenize};

use super::types::{Export, ExportKind, Import, ImportedSymbols};

pub fn parse_imports_exports(source: &str) -> (Vec<Import>, Vec<Export>) {
    let tokens = match tokenize(source) {
        Ok(t) => t,
        Err(_) => return (vec![], vec![]),
    };

    let significant: Vec<&Token> = tokens
        .iter()
        .filter(|t| {
            !matches!(
                t.kind,
                TokenKind::Whitespace
                    | TokenKind::Newline
                    | TokenKind::LineComment
                    | TokenKind::BlockComment
            )
        })
        .collect();

    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut i = 0;

    while i < significant.len() {
        let tok = significant[i];
        if tok.kind == TokenKind::Identifier && tok.text == "import" {
            if let Some((imp, next)) = parse_import(&significant, i) {
                imports.push(imp);
                i = next;
                continue;
            }
        } else if tok.kind == TokenKind::Identifier && tok.text == "export" {
            let (mut exps, next) = parse_export(&significant, i);
            exports.append(&mut exps);
            i = next;
            continue;
        } else if tok.kind == TokenKind::Identifier && tok.text == "require" {
            if let Some((imp, next)) = parse_require(&significant, i) {
                imports.push(imp);
                i = next;
                continue;
            }
        }
        i += 1;
    }

    (imports, exports)
}

fn parse_import(tokens: &[&Token], start: usize) -> Option<(Import, usize)> {
    let line = tokens[start].line;
    let mut i = start + 1;
    if i >= tokens.len() {
        return None;
    }

    // import "side-effect"
    if tokens[i].kind == TokenKind::StringLiteral {
        let source = unquote(&tokens[i].text);
        return Some((
            Import {
                source,
                symbols: ImportedSymbols::SideEffect,
                line,
            },
            i + 1,
        ));
    }

    // import * as name from "source"
    if tokens[i].kind == TokenKind::Operator && tokens[i].text == "*" {
        i += 1;
        if i < tokens.len() && tokens[i].kind == TokenKind::Identifier && tokens[i].text == "as" {
            i += 1;
            if i < tokens.len() && tokens[i].kind == TokenKind::Identifier {
                let name = tokens[i].text.clone();
                i += 1;
                if let Some((source, next)) = consume_from(tokens, i) {
                    return Some((
                        Import {
                            source,
                            symbols: ImportedSymbols::Namespace(name),
                            line,
                        },
                        next,
                    ));
                }
            }
        }
        return None;
    }

    // import { a, b } from "source"
    if tokens[i].kind == TokenKind::OpenBrace {
        let (names, next) = parse_named_bindings(tokens, i);
        if let Some((source, next)) = consume_from(tokens, next) {
            return Some((
                Import {
                    source,
                    symbols: ImportedSymbols::Named(names),
                    line,
                },
                next,
            ));
        }
        return None;
    }

    // import type { ... } from "source" — TS type imports
    if tokens[i].kind == TokenKind::Identifier && tokens[i].text == "type" {
        i += 1;
        if i < tokens.len() && tokens[i].kind == TokenKind::OpenBrace {
            let (names, next) = parse_named_bindings(tokens, i);
            if let Some((source, next)) = consume_from(tokens, next) {
                return Some((
                    Import {
                        source,
                        symbols: ImportedSymbols::Named(names),
                        line,
                    },
                    next,
                ));
            }
        }
        // import type Foo from "source"
        if i < tokens.len() && tokens[i].kind == TokenKind::Identifier {
            let name = tokens[i].text.clone();
            i += 1;
            if let Some((source, next)) = consume_from(tokens, i) {
                return Some((
                    Import {
                        source,
                        symbols: ImportedSymbols::Default(name),
                        line,
                    },
                    next,
                ));
            }
        }
        return None;
    }

    // import defaultName from "source"
    // import defaultName, { a, b } from "source"
    if tokens[i].kind == TokenKind::Identifier {
        let name = tokens[i].text.clone();
        i += 1;

        // import default, { named } from "source"
        if i < tokens.len() && tokens[i].kind == TokenKind::Comma {
            i += 1;
            if i < tokens.len() && tokens[i].kind == TokenKind::OpenBrace {
                let (mut names, next) = parse_named_bindings(tokens, i);
                names.insert(0, name.clone());
                if let Some((source, next)) = consume_from(tokens, next) {
                    return Some((
                        Import {
                            source,
                            symbols: ImportedSymbols::Named(names),
                            line,
                        },
                        next,
                    ));
                }
            }
            // import default, * as ns from "source"
            if i < tokens.len()
                && tokens[i].kind == TokenKind::Operator
                && tokens[i].text == "*"
            {
                i += 1; // skip *
                if i < tokens.len()
                    && tokens[i].kind == TokenKind::Identifier
                    && tokens[i].text == "as"
                {
                    i += 1; // skip as
                    if i < tokens.len() && tokens[i].kind == TokenKind::Identifier {
                        i += 1; // skip ns name
                    }
                }
                if let Some((source, next)) = consume_from(tokens, i) {
                    return Some((
                        Import {
                            source,
                            symbols: ImportedSymbols::Default(name),
                            line,
                        },
                        next,
                    ));
                }
            }
            return None;
        }

        if let Some((source, next)) = consume_from(tokens, i) {
            return Some((
                Import {
                    source,
                    symbols: ImportedSymbols::Default(name),
                    line,
                },
                next,
            ));
        }
    }

    None
}

fn parse_export(tokens: &[&Token], start: usize) -> (Vec<Export>, usize) {
    let line = tokens[start].line;
    let mut i = start + 1;
    if i >= tokens.len() {
        return (vec![], i);
    }

    // export default
    if tokens[i].kind == TokenKind::Identifier && tokens[i].text == "default" {
        i += 1;
        // Skip to next statement boundary
        while i < tokens.len()
            && tokens[i].kind != TokenKind::Semicolon
            && tokens[i].kind != TokenKind::Eof
        {
            i += 1;
        }
        if i < tokens.len() && tokens[i].kind == TokenKind::Semicolon {
            i += 1;
        }
        return (
            vec![Export {
                name: "default".to_string(),
                kind: ExportKind::Default,
                line,
            }],
            i,
        );
    }

    // export * from "source" (re-export all)
    if tokens[i].kind == TokenKind::Operator && tokens[i].text == "*" {
        i += 1;
        if let Some((source, next)) = consume_from(tokens, i) {
            return (
                vec![Export {
                    name: "*".to_string(),
                    kind: ExportKind::ReexportFrom(source),
                    line,
                }],
                next,
            );
        }
        return (vec![], i);
    }

    // export { a, b } or export { a, b } from "source"
    if tokens[i].kind == TokenKind::OpenBrace {
        let (names, next) = parse_named_bindings(tokens, i);
        // Check for re-export: from "source"
        if let Some((source, next)) = consume_from(tokens, next) {
            let exps = names
                .into_iter()
                .map(|n| Export {
                    name: n,
                    kind: ExportKind::ReexportFrom(source.clone()),
                    line,
                })
                .collect();
            return (exps, next);
        }
        let exps = names
            .into_iter()
            .map(|n| Export {
                name: n,
                kind: ExportKind::Named,
                line,
            })
            .collect();
        return (exps, next);
    }

    // export type { ... } from "source" — TS type re-exports
    if tokens[i].kind == TokenKind::Identifier && tokens[i].text == "type" {
        i += 1;
        if i < tokens.len() && tokens[i].kind == TokenKind::OpenBrace {
            let (names, next) = parse_named_bindings(tokens, i);
            if let Some((source, next)) = consume_from(tokens, next) {
                let exps = names
                    .into_iter()
                    .map(|n| Export {
                        name: n,
                        kind: ExportKind::ReexportFrom(source.clone()),
                        line,
                    })
                    .collect();
                return (exps, next);
            }
            let exps = names
                .into_iter()
                .map(|n| Export {
                    name: n,
                    kind: ExportKind::Named,
                    line,
                })
                .collect();
            return (exps, next);
        }
    }

    // export const/let/var name, export function name, export class name
    // export async function name
    if tokens[i].kind == TokenKind::Identifier {
        let kw = tokens[i].text.as_str();
        match kw {
            "const" | "let" | "var" => {
                i += 1;
                if i < tokens.len() && tokens[i].kind == TokenKind::Identifier {
                    let name = tokens[i].text.clone();
                    i += 1;
                    return (
                        vec![Export {
                            name,
                            kind: ExportKind::Named,
                            line,
                        }],
                        i,
                    );
                }
            }
            "function" | "class" => {
                i += 1;
                // skip * for generator functions
                if i < tokens.len()
                    && tokens[i].kind == TokenKind::Operator
                    && tokens[i].text == "*"
                {
                    i += 1;
                }
                if i < tokens.len() && tokens[i].kind == TokenKind::Identifier {
                    let name = tokens[i].text.clone();
                    i += 1;
                    return (
                        vec![Export {
                            name,
                            kind: ExportKind::Named,
                            line,
                        }],
                        i,
                    );
                }
            }
            "async" => {
                i += 1;
                if i < tokens.len()
                    && tokens[i].kind == TokenKind::Identifier
                    && tokens[i].text == "function"
                {
                    i += 1;
                    // skip * for async generator
                    if i < tokens.len()
                        && tokens[i].kind == TokenKind::Operator
                        && tokens[i].text == "*"
                    {
                        i += 1;
                    }
                    if i < tokens.len() && tokens[i].kind == TokenKind::Identifier {
                        let name = tokens[i].text.clone();
                        i += 1;
                        return (
                            vec![Export {
                                name,
                                kind: ExportKind::Named,
                                line,
                            }],
                            i,
                        );
                    }
                }
            }
            "enum" | "interface" | "abstract" => {
                i += 1;
                // For "abstract", skip the next keyword (e.g., "class")
                if kw == "abstract"
                    && i < tokens.len()
                    && tokens[i].kind == TokenKind::Identifier
                    && tokens[i].text == "class"
                {
                    i += 1;
                }
                if i < tokens.len() && tokens[i].kind == TokenKind::Identifier {
                    let name = tokens[i].text.clone();
                    i += 1;
                    return (
                        vec![Export {
                            name,
                            kind: ExportKind::Named,
                            line,
                        }],
                        i,
                    );
                }
            }
            _ => {}
        }
    }

    (vec![], i)
}

fn parse_require(tokens: &[&Token], start: usize) -> Option<(Import, usize)> {
    let line = tokens[start].line;
    let mut i = start + 1;
    if i >= tokens.len() || tokens[i].kind != TokenKind::OpenParen {
        return None;
    }
    i += 1;
    if i >= tokens.len() || tokens[i].kind != TokenKind::StringLiteral {
        return None;
    }
    let source = unquote(&tokens[i].text);
    i += 1;
    if i < tokens.len() && tokens[i].kind == TokenKind::CloseParen {
        i += 1;
    }
    Some((
        Import {
            source,
            symbols: ImportedSymbols::SideEffect,
            line,
        },
        i,
    ))
}

fn parse_named_bindings(tokens: &[&Token], start: usize) -> (Vec<String>, usize) {
    let mut names = Vec::new();
    let mut i = start + 1; // skip opening brace
    while i < tokens.len() {
        if tokens[i].kind == TokenKind::CloseBrace {
            return (names, i + 1);
        }
        if tokens[i].kind == TokenKind::Comma {
            i += 1;
            continue;
        }
        if tokens[i].kind == TokenKind::Identifier {
            let mut name = tokens[i].text.clone();
            i += 1;
            // Skip inline `type` keyword: import { type Foo } from '...'
            if name == "type"
                && i < tokens.len()
                && tokens[i].kind == TokenKind::Identifier
            {
                name = tokens[i].text.clone();
                i += 1;
            }
            // Handle `as alias`
            if i < tokens.len()
                && tokens[i].kind == TokenKind::Identifier
                && tokens[i].text == "as"
            {
                i += 1; // skip "as"
                if i < tokens.len() && tokens[i].kind == TokenKind::Identifier {
                    // Use the alias as the local name
                    names.push(tokens[i].text.clone());
                    i += 1;
                    continue;
                }
            }
            names.push(name);
        } else {
            i += 1;
        }
    }
    (names, i)
}

fn consume_from(tokens: &[&Token], start: usize) -> Option<(String, usize)> {
    let mut i = start;
    if i < tokens.len() && tokens[i].kind == TokenKind::Identifier && tokens[i].text == "from" {
        i += 1;
        if i < tokens.len() && tokens[i].kind == TokenKind::StringLiteral {
            let source = unquote(&tokens[i].text);
            i += 1;
            // skip optional semicolon
            if i < tokens.len() && tokens[i].kind == TokenKind::Semicolon {
                i += 1;
            }
            return Some((source, i));
        }
    }
    None
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_effect_import() {
        let (imports, _) = parse_imports_exports(r#"import "./styles.css";"#);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "./styles.css");
        assert!(matches!(imports[0].symbols, ImportedSymbols::SideEffect));
    }

    #[test]
    fn default_import() {
        let (imports, _) = parse_imports_exports(r#"import React from "react";"#);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "react");
        assert!(matches!(imports[0].symbols, ImportedSymbols::Default(ref n) if n == "React"));
    }

    #[test]
    fn named_imports() {
        let (imports, _) =
            parse_imports_exports(r#"import { useState, useEffect } from "react";"#);
        assert_eq!(imports.len(), 1);
        if let ImportedSymbols::Named(ref names) = imports[0].symbols {
            assert_eq!(names.len(), 2);
            assert!(names.contains(&"useState".to_string()));
            assert!(names.contains(&"useEffect".to_string()));
        } else {
            panic!("expected Named");
        }
    }

    #[test]
    fn namespace_import() {
        let (imports, _) = parse_imports_exports(r#"import * as path from "path";"#);
        assert_eq!(imports.len(), 1);
        assert!(matches!(imports[0].symbols, ImportedSymbols::Namespace(ref n) if n == "path"));
    }

    #[test]
    fn named_export() {
        let (_, exports) = parse_imports_exports("export const foo = 42;");
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "foo");
        assert!(matches!(exports[0].kind, ExportKind::Named));
    }

    #[test]
    fn default_export() {
        let (_, exports) = parse_imports_exports("export default function main() {}");
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "default");
        assert!(matches!(exports[0].kind, ExportKind::Default));
    }

    #[test]
    fn export_function() {
        let (_, exports) = parse_imports_exports("export function hello() {}");
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "hello");
    }

    #[test]
    fn export_class() {
        let (_, exports) = parse_imports_exports("export class MyClass {}");
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "MyClass");
    }

    #[test]
    fn reexport_all() {
        let (_, exports) = parse_imports_exports(r#"export * from "./utils";"#);
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "*");
        assert!(matches!(exports[0].kind, ExportKind::ReexportFrom(ref s) if s == "./utils"));
    }

    #[test]
    fn reexport_named() {
        let (_, exports) = parse_imports_exports(r#"export { foo, bar } from "./lib";"#);
        assert_eq!(exports.len(), 2);
        assert!(matches!(exports[0].kind, ExportKind::ReexportFrom(ref s) if s == "./lib"));
    }

    #[test]
    fn export_named_list() {
        let (_, exports) = parse_imports_exports("export { foo, bar };");
        assert_eq!(exports.len(), 2);
        assert_eq!(exports[0].name, "foo");
        assert_eq!(exports[1].name, "bar");
    }

    #[test]
    fn require_call() {
        let (imports, _) = parse_imports_exports(r#"const x = require("lodash");"#);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "lodash");
    }

    #[test]
    fn mixed_imports_exports() {
        let code = r#"
            import { useState } from "react";
            import "./global.css";
            export const App = () => {};
            export default App;
        "#;
        let (imports, exports) = parse_imports_exports(code);
        assert_eq!(imports.len(), 2);
        assert_eq!(exports.len(), 2);
    }

    #[test]
    fn named_import_with_alias() {
        let (imports, _) =
            parse_imports_exports(r#"import { default as React } from "react";"#);
        assert_eq!(imports.len(), 1);
        if let ImportedSymbols::Named(ref names) = imports[0].symbols {
            assert_eq!(names, &["React"]);
        } else {
            panic!("expected Named");
        }
    }

    #[test]
    fn export_async_function() {
        let (_, exports) = parse_imports_exports("export async function fetchData() {}");
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "fetchData");
    }

    #[test]
    fn type_import() {
        let (imports, _) =
            parse_imports_exports(r#"import type { Foo } from "./types";"#);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "./types");
    }

    #[test]
    fn empty_input() {
        let (imports, exports) = parse_imports_exports("");
        assert!(imports.is_empty());
        assert!(exports.is_empty());
    }

    #[test]
    fn no_imports_or_exports() {
        let (imports, exports) =
            parse_imports_exports("const x = 1;\nfunction foo() { return x; }");
        assert!(imports.is_empty());
        assert!(exports.is_empty());
    }

    #[test]
    fn unquote_double() {
        assert_eq!(unquote("\"hello\""), "hello");
    }

    #[test]
    fn unquote_single() {
        assert_eq!(unquote("'hello'"), "hello");
    }

    #[test]
    fn unquote_no_quotes() {
        assert_eq!(unquote("hello"), "hello");
    }
}
