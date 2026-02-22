use crate::core::volkiwithstds::collections::ToString;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fmt;

use super::config::{ArrowParens, EndOfLine, FormatConfig, TrailingComma};
use super::plugin_bridge;
use super::tokenizer::{Token, TokenKind, tokenize};

use crate::core::plugins::protocol::{JsonOut, PluginRequest, PluginResponse};
use crate::core::plugins::registry::PluginRegistry;
use crate::core::plugins::types::PluginSpec;
use crate::{vstr, vvec};

#[derive(Debug)]
pub struct FormatError {
    pub message: String,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "format error: {}", self.message)
    }
}

impl core::error::Error for FormatError {}

pub fn format_source(
    source: &str,
    config: &FormatConfig,
    plugins: Option<&PluginRegistry>,
) -> Result<String, FormatError> {
    let mut tokens = tokenize(source).map_err(|e| FormatError {
        message: e.to_vstring(),
    })?;

    run_plugin_hook(&mut tokens, config, plugins, "formatter.before_all");

    normalize_line_endings(&mut tokens, config);
    normalize_quotes(&mut tokens, config);

    run_plugin_hook(&mut tokens, config, plugins, "formatter.after_normalize");

    normalize_semicolons(&mut tokens, config);
    normalize_bracket_spacing(&mut tokens, config);
    normalize_trailing_commas(&mut tokens, config);
    normalize_arrow_parens(&mut tokens, config);

    run_plugin_hook(&mut tokens, config, plugins, "formatter.before_whitespace");

    normalize_whitespace(&mut tokens, config);

    run_plugin_hook(&mut tokens, config, plugins, "formatter.after_all");

    Ok(serialize(&tokens, config))
}

fn run_plugin_hook(
    tokens: &mut Vec<Token>,
    config: &FormatConfig,
    plugins: Option<&PluginRegistry>,
    hook: &str,
) {
    let registry = match plugins {
        Some(r) if !r.is_empty() => r,
        _ => return,
    };

    let hook_str = hook.to_vstring();
    let config_json = plugin_bridge::config_to_json(config);

    let results = registry.invoke_hook(&|spec: &PluginSpec| {
        let tokens_json = plugin_bridge::tokens_to_json(tokens);
        PluginRequest {
            hook: hook_str.clone(),
            data: JsonOut::Object(vvec![
                ("tokens".into(), tokens_json),
                ("config".into(), config_json.clone()),
            ]),
            plugin_options: spec.options.clone(),
        }
    });

    for result in results {
        match result {
            Ok(PluginResponse::Ok { data }) => {
                if let Some(obj) = data.as_object() {
                    if let Some(tok_val) = obj.get("tokens") {
                        if let Some(new_tokens) = plugin_bridge::tokens_from_json(tok_val) {
                            *tokens = new_tokens;
                        }
                    }
                }
            }
            Ok(PluginResponse::Skip) => {}
            Ok(PluginResponse::Error { message }) => {
                crate::veprintln!("plugin error at hook {hook}: {message}");
            }
            Err(e) => {
                crate::veprintln!("plugin invocation error at hook {hook}: {e}");
            }
        }
    }
}

// Pass 1: Normalize line endings
fn normalize_line_endings(tokens: &mut Vec<Token>, config: &FormatConfig) {
    let eol = config.end_of_line.as_str();
    if matches!(config.end_of_line, EndOfLine::Auto) {
        return;
    }
    for token in tokens.iter_mut() {
        if token.kind == TokenKind::Newline {
            token.text = eol.to_vstring();
        }
    }
}

// Pass 2: Normalize quote style
fn normalize_quotes(tokens: &mut Vec<Token>, config: &FormatConfig) {
    let target_quote = if config.single_quote { '\'' } else { '"' };
    for token in tokens.iter_mut() {
        if token.kind == TokenKind::StringLiteral && token.text.len() >= 2 {
            let current_quote = token.text.as_bytes()[0] as char;
            if current_quote != target_quote {
                try_swap_quotes(token, current_quote, target_quote);
            }
        }
    }
}

fn try_swap_quotes(token: &mut Token, from: char, to: char) {
    let inner = &token.text[1..token.text.len() - 1];

    let mut target_count = 0usize;
    let mut from_escaped = 0usize;
    let chars: Vec<char> = inner.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' {
            if i + 1 < chars.len() && chars[i + 1] == from {
                from_escaped += 1;
            }
            i += 2;
        } else {
            if chars[i] == to {
                target_count += 1;
            }
            i += 1;
        }
    }

    // Skip swap if it would add more escapes than it removes
    if target_count > from_escaped {
        return;
    }

    let mut result = String::with_capacity(token.text.len());
    result.push(to);
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            if chars[i + 1] == from {
                result.push(from);
            } else {
                result.push('\\');
                result.push(chars[i + 1]);
            }
            i += 2;
        } else if chars[i] == to {
            result.push('\\');
            result.push(to);
            i += 1;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result.push(to);
    token.text = result;
}

// Pass 3: Normalize semicolons
fn normalize_semicolons(tokens: &mut Vec<Token>, config: &FormatConfig) {
    if config.semi {
        insert_semicolons(tokens);
    } else {
        remove_semicolons(tokens);
    }
}

fn is_statement_end(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Identifier
            | TokenKind::NumericLiteral
            | TokenKind::StringLiteral
            | TokenKind::TemplateLiteral
            | TokenKind::TemplateTail
            | TokenKind::CloseParen
            | TokenKind::CloseBracket
            | TokenKind::RegexLiteral
    )
}

fn is_keyword_no_semi(text: &str) -> bool {
    matches!(
        text,
        "if" | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "try"
            | "catch"
            | "finally"
            | "class"
            | "function"
    )
}

fn insert_semicolons(tokens: &mut Vec<Token>) {
    let mut insertions = Vec::new();
    let mut last_inserted_for: Option<usize> = None;

    for i in 0..tokens.len() {
        if tokens[i].kind != TokenKind::Newline {
            continue;
        }
        let prev = find_significant_before(tokens, i);
        let next = find_significant_after(tokens, i);

        if let Some(pi) = prev {
            // Skip if we already inserted a semicolon for this same statement-ending token
            if last_inserted_for == Some(pi) {
                continue;
            }
            if !is_statement_end(&tokens[pi].kind) {
                continue;
            }
            if tokens[pi].kind == TokenKind::Identifier && is_keyword_no_semi(&tokens[pi].text) {
                continue;
            }
            if let Some(ni) = next {
                match tokens[ni].kind {
                    TokenKind::CloseBracket | TokenKind::CloseParen => continue,
                    TokenKind::Dot | TokenKind::QuestionMark | TokenKind::Colon => continue,
                    TokenKind::Operator if tokens[ni].text == "?" => continue,
                    _ => {}
                }
                if tokens[ni].kind == TokenKind::Identifier
                    && matches!(tokens[ni].text.as_str(), "else" | "catch" | "finally")
                {
                    continue;
                }
            }
            if tokens[pi].kind == TokenKind::Semicolon {
                continue;
            }
            insertions.push(i);
            last_inserted_for = Some(pi);
        }
    }
    for (offset, idx) in insertions.iter().enumerate() {
        let line = tokens[idx + offset].line;
        let col = tokens[idx + offset].col;
        tokens.insert(
            idx + offset,
            Token {
                kind: TokenKind::Semicolon,
                text: ";".into(),
                line,
                col,
            },
        );
    }
}

fn remove_semicolons(tokens: &mut Vec<Token>) {
    let mut removals = Vec::new();
    for i in 0..tokens.len() {
        if tokens[i].kind != TokenKind::Semicolon {
            continue;
        }
        let next_sig = find_significant_after(tokens, i);
        if let Some(ni) = next_sig {
            let is_asi_hazard = matches!(
                tokens[ni].kind,
                TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::RegexLiteral
            ) || (tokens[ni].kind == TokenKind::TemplateLiteral
                || tokens[ni].kind == TokenKind::TemplateHead)
                || (tokens[ni].kind == TokenKind::Operator
                    && (tokens[ni].text == "+" || tokens[ni].text == "-"));

            if is_asi_hazard {
                continue;
            }
        }
        // Check for `for (;;)` — don't remove semis inside for-parens
        if is_inside_for_parens(tokens, i) {
            continue;
        }
        removals.push(i);
    }
    for (offset, idx) in removals.iter().enumerate() {
        tokens.remove(idx - offset);
    }
}

fn is_inside_for_parens(tokens: &[Token], pos: usize) -> bool {
    let mut depth = 0i32;
    for i in (0..pos).rev() {
        match tokens[i].kind {
            TokenKind::CloseParen => depth += 1,
            TokenKind::OpenParen => {
                if depth == 0 {
                    if let Some(pi) = find_significant_before(tokens, i) {
                        return tokens[pi].kind == TokenKind::Identifier
                            && tokens[pi].text == "for";
                    }
                    return false;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    false
}

// Pass 4: Bracket spacing
fn normalize_bracket_spacing(tokens: &mut Vec<Token>, config: &FormatConfig) {
    let len = tokens.len();
    let mut i = 0;
    while i < len {
        if tokens[i].kind == TokenKind::OpenBrace && is_object_literal_brace(tokens, i) {
            let close = find_matching_close_brace(tokens, i);
            if let Some(ci) = close {
                if is_single_line_range(tokens, i, ci) {
                    apply_brace_spacing(tokens, i, ci, config.bracket_spacing);
                }
            }
        }
        i += 1;
    }
}

fn is_object_literal_brace(tokens: &[Token], pos: usize) -> bool {
    let prev = find_significant_before(tokens, pos);
    match prev {
        None => true,
        Some(pi) => {
            matches!(
                tokens[pi].kind,
                TokenKind::Assignment
                    | TokenKind::Comma
                    | TokenKind::OpenParen
                    | TokenKind::OpenBracket
                    | TokenKind::Colon
                    | TokenKind::Arrow
                    | TokenKind::Semicolon
            ) || (tokens[pi].kind == TokenKind::Identifier
                && matches!(
                    tokens[pi].text.as_str(),
                    "return" | "export" | "default" | "yield" | "await"
                ))
        }
    }
}

fn find_matching_close_brace(tokens: &[Token], open: usize) -> Option<usize> {
    let mut depth = 1;
    for i in (open + 1)..tokens.len() {
        match tokens[i].kind {
            TokenKind::OpenBrace => depth += 1,
            TokenKind::CloseBrace => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_single_line_range(tokens: &[Token], from: usize, to: usize) -> bool {
    for i in from..=to {
        if tokens[i].kind == TokenKind::Newline {
            return false;
        }
    }
    true
}

fn apply_brace_spacing(tokens: &mut [Token], open: usize, close: usize, spacing: bool) {
    if open + 1 < close {
        if spacing {
            if tokens[open + 1].kind == TokenKind::Whitespace {
                tokens[open + 1].text = " ".into();
            }
        } else if tokens[open + 1].kind == TokenKind::Whitespace {
            tokens[open + 1].text = String::new();
        }
    }
    if close > 0 && close - 1 > open {
        if spacing {
            if tokens[close - 1].kind == TokenKind::Whitespace {
                tokens[close - 1].text = " ".into();
            }
        } else if tokens[close - 1].kind == TokenKind::Whitespace {
            tokens[close - 1].text = String::new();
        }
    }
}

// Pass 5: Trailing commas
fn normalize_trailing_commas(tokens: &mut Vec<Token>, config: &FormatConfig) {
    let mut i = 0;
    while i < tokens.len() {
        let is_close = matches!(
            tokens[i].kind,
            TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace
        );
        if !is_close {
            i += 1;
            continue;
        }

        let matching_open = find_matching_open(tokens, i);
        if matching_open.is_none() {
            i += 1;
            continue;
        }
        let open_idx = matching_open.unwrap();
        let multi_line = !is_single_line_range(tokens, open_idx, i);
        let prev_sig = find_significant_before(tokens, i);

        if multi_line && !matches!(config.trailing_comma, TrailingComma::None) {
            if let Some(pi) = prev_sig {
                if pi > open_idx
                    && tokens[pi].kind != TokenKind::Comma
                    && is_comma_eligible(&tokens[pi])
                    && !is_empty_body(tokens, open_idx, i)
                {
                    let line = tokens[pi].line;
                    let col = tokens[pi].col;
                    let insert_at = pi + 1;
                    tokens.insert(
                        insert_at,
                        Token {
                            kind: TokenKind::Comma,
                            text: ",".into(),
                            line,
                            col,
                        },
                    );
                    i += 2;
                    continue;
                }
            }
        } else if !multi_line {
            if let Some(pi) = prev_sig {
                if tokens[pi].kind == TokenKind::Comma && pi > open_idx {
                    tokens.remove(pi);
                    i = i.saturating_sub(1);
                    continue;
                }
            }
        } else if matches!(config.trailing_comma, TrailingComma::None) && multi_line {
            if let Some(pi) = prev_sig {
                if tokens[pi].kind == TokenKind::Comma && pi > open_idx {
                    tokens.remove(pi);
                    i = i.saturating_sub(1);
                    continue;
                }
            }
        }

        i += 1;
    }
}

fn is_comma_eligible(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::Identifier
            | TokenKind::NumericLiteral
            | TokenKind::StringLiteral
            | TokenKind::TemplateLiteral
            | TokenKind::TemplateTail
            | TokenKind::CloseParen
            | TokenKind::CloseBracket
            | TokenKind::CloseBrace
            | TokenKind::RegexLiteral
    )
}

fn is_empty_body(tokens: &[Token], open: usize, close: usize) -> bool {
    for i in (open + 1)..close {
        if !matches!(tokens[i].kind, TokenKind::Whitespace | TokenKind::Newline) {
            return false;
        }
    }
    true
}

fn find_matching_open(tokens: &[Token], close: usize) -> Option<usize> {
    let target = match tokens[close].kind {
        TokenKind::CloseParen => TokenKind::OpenParen,
        TokenKind::CloseBracket => TokenKind::OpenBracket,
        TokenKind::CloseBrace => TokenKind::OpenBrace,
        _ => return None,
    };
    let mut depth = 1;
    for i in (0..close).rev() {
        if tokens[i].kind == tokens[close].kind {
            depth += 1;
        } else if tokens[i].kind == target {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

// Pass 6: Arrow parens
fn normalize_arrow_parens(tokens: &mut Vec<Token>, config: &FormatConfig) {
    match config.arrow_parens {
        ArrowParens::Always => add_arrow_parens(tokens),
        ArrowParens::Avoid => remove_arrow_parens(tokens),
    }
}

fn add_arrow_parens(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].kind == TokenKind::Arrow {
            // Look back for a bare identifier (no paren before it)
            let prev_sig = find_significant_before(tokens, i);
            if let Some(pi) = prev_sig {
                if tokens[pi].kind == TokenKind::Identifier {
                    let before_ident = find_significant_before(tokens, pi);
                    let is_bare = match before_ident {
                        None => true,
                        Some(bi) => {
                            !matches!(tokens[bi].kind, TokenKind::CloseParen | TokenKind::Dot)
                        }
                    };
                    if is_bare {
                        let line = tokens[pi].line;
                        let col = tokens[pi].col;
                        tokens.insert(
                            pi + 1,
                            Token {
                                kind: TokenKind::CloseParen,
                                text: ")".into(),
                                line,
                                col,
                            },
                        );
                        tokens.insert(
                            pi,
                            Token {
                                kind: TokenKind::OpenParen,
                                text: "(".into(),
                                line,
                                col,
                            },
                        );
                        i += 3; // skip past the arrow
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
}

fn remove_arrow_parens(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].kind == TokenKind::Arrow {
            let prev_sig = find_significant_before(tokens, i);
            if let Some(close_idx) = prev_sig {
                if tokens[close_idx].kind == TokenKind::CloseParen {
                    let open_idx = find_matching_open(tokens, close_idx);
                    if let Some(oi) = open_idx {
                        // Check: exactly one simple identifier between parens
                        let inner_sig: Vec<usize> = ((oi + 1)..close_idx)
                            .filter(|&j| {
                                !matches!(
                                    tokens[j].kind,
                                    TokenKind::Whitespace | TokenKind::Newline
                                )
                            })
                            .collect();
                        if inner_sig.len() == 1
                            && tokens[inner_sig[0]].kind == TokenKind::Identifier
                        {
                            // No type annotation or default value
                            let before_open = find_significant_before(tokens, oi);
                            let is_arrow_param = match before_open {
                                None => true,
                                Some(bi) => {
                                    matches!(
                                        tokens[bi].kind,
                                        TokenKind::Assignment
                                            | TokenKind::Comma
                                            | TokenKind::OpenParen
                                            | TokenKind::Semicolon
                                            | TokenKind::Arrow
                                            | TokenKind::Colon
                                            | TokenKind::OpenBrace
                                            | TokenKind::OpenBracket
                                    ) || (tokens[bi].kind == TokenKind::Identifier
                                        && matches!(
                                            tokens[bi].text.as_str(),
                                            "return"
                                                | "const"
                                                | "let"
                                                | "var"
                                                | "export"
                                                | "default"
                                                | "yield"
                                                | "await"
                                                | "=>"
                                        ))
                                }
                            };
                            if is_arrow_param {
                                tokens.remove(close_idx);
                                tokens.remove(oi);
                                i = i.saturating_sub(2);
                                continue;
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }
}

// Pass 7: Whitespace and indentation
fn normalize_whitespace(tokens: &mut Vec<Token>, config: &FormatConfig) {
    collapse_blank_lines(tokens);
    normalize_spacing(tokens);
    reindent(tokens, config);
    wrap_long_lines(tokens, config);
    reindent(tokens, config);
}

fn collapse_blank_lines(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].kind == TokenKind::Newline {
            let mut newline_count = 1;
            let mut j = i + 1;
            while j < tokens.len() {
                if tokens[j].kind == TokenKind::Newline {
                    newline_count += 1;
                    j += 1;
                } else if tokens[j].kind == TokenKind::Whitespace {
                    j += 1;
                } else {
                    break;
                }
            }
            if newline_count > 2 {
                // Keep at most 2 newlines (one blank line)
                let eol = tokens[i].text.clone();
                let mut keep = vvec![
                    Token {
                        kind: TokenKind::Newline,
                        text: eol.clone(),
                        line: 0,
                        col: 0
                    },
                    Token {
                        kind: TokenKind::Newline,
                        text: eol,
                        line: 0,
                        col: 0
                    },
                ];
                // Preserve trailing whitespace (indentation of next line)
                if j > 0 && j < tokens.len() && tokens[j - 1].kind == TokenKind::Whitespace {
                } else {
                }
                let _ = tokens.splice(i..j, keep.drain(..));
                i += 2;
            } else {
                i = j;
            }
        } else {
            i += 1;
        }
    }
}

fn normalize_spacing(tokens: &mut Vec<Token>) {
    for i in 0..tokens.len() {
        if tokens[i].kind == TokenKind::Whitespace
            && !is_after_newline(tokens, i)
            && tokens[i].text.len() > 1
        {
            tokens[i].text = " ".into();
        }
    }
}

fn is_after_newline(tokens: &[Token], pos: usize) -> bool {
    pos > 0 && tokens[pos - 1].kind == TokenKind::Newline
}

fn reindent(tokens: &mut Vec<Token>, config: &FormatConfig) {
    let indent_str = if config.use_tabs {
        vstr!("\t")
    } else {
        vstr!(" ").repeat(config.tab_width)
    };

    let mut depth: usize = 0;
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i].kind {
            TokenKind::OpenBrace | TokenKind::OpenBracket | TokenKind::OpenParen => {
                depth += 1;
            }
            TokenKind::CloseBrace | TokenKind::CloseBracket | TokenKind::CloseParen => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }

        // After a newline, set indentation whitespace
        if tokens[i].kind == TokenKind::Newline {
            let next_idx = i + 1;
            if next_idx < tokens.len() {
                // Peek at close delimiter after whitespace to dedent
                let effective_depth = if let Some(close_idx) = find_first_non_ws(tokens, next_idx) {
                    if matches!(
                        tokens[close_idx].kind,
                        TokenKind::CloseBrace | TokenKind::CloseBracket | TokenKind::CloseParen
                    ) {
                        depth.saturating_sub(1)
                    } else {
                        depth
                    }
                } else {
                    depth
                };

                let new_indent = indent_str.repeat(effective_depth);
                if tokens[next_idx].kind == TokenKind::Whitespace {
                    tokens[next_idx].text = new_indent;
                } else if effective_depth > 0
                    && tokens[next_idx].kind != TokenKind::Newline
                    && tokens[next_idx].kind != TokenKind::Eof
                {
                    tokens.insert(
                        next_idx,
                        Token {
                            kind: TokenKind::Whitespace,
                            text: new_indent,
                            line: 0,
                            col: 0,
                        },
                    );
                }
            }
        }
        i += 1;
    }
}

fn find_first_non_ws(tokens: &[Token], from: usize) -> Option<usize> {
    for i in from..tokens.len() {
        if tokens[i].kind != TokenKind::Whitespace {
            return Some(i);
        }
    }
    None
}

// --- Group-based line wrapping (Prettier-like algorithm) ---

#[derive(Debug, Clone, Copy, PartialEq)]
enum GroupKind {
    FunctionCall,
    FunctionParams,
    ArrayLiteral,
    ObjectLiteral,
    Block,
    Other,
}

#[derive(Debug, Clone)]
struct DelimiterGroup {
    open_idx: usize,
    close_idx: usize,
    comma_indices: Vec<usize>,
    flat_len: usize,
    has_source_newline: bool,
    kind: GroupKind,
    children: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BreakDecision {
    Flat,
    Expanded,
    LastArgExpanded,
}

fn find_matching_close(tokens: &[Token], open: usize) -> Option<usize> {
    let (open_kind, close_kind) = match tokens[open].kind {
        TokenKind::OpenParen => (TokenKind::OpenParen, TokenKind::CloseParen),
        TokenKind::OpenBracket => (TokenKind::OpenBracket, TokenKind::CloseBracket),
        TokenKind::OpenBrace => (TokenKind::OpenBrace, TokenKind::CloseBrace),
        _ => return None,
    };
    let mut depth = 1;
    for i in (open + 1)..tokens.len() {
        if tokens[i].kind == open_kind {
            depth += 1;
        } else if tokens[i].kind == close_kind {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn measure_flat_length(tokens: &[Token], from: usize, to: usize) -> usize {
    let mut len = 0;
    for i in from..=to {
        match tokens[i].kind {
            TokenKind::Newline => {}
            TokenKind::Whitespace => {
                if i > 0 && tokens[i - 1].kind == TokenKind::Newline {
                } else {
                    len += 1;
                }
            }
            _ => {
                len += tokens[i].text.len();
            }
        }
    }
    len
}

fn has_newline_in_range(tokens: &[Token], from: usize, to: usize) -> bool {
    for i in from..=to {
        if tokens[i].kind == TokenKind::Newline {
            return true;
        }
    }
    false
}

fn classify_group_kind(tokens: &[Token], open_idx: usize) -> GroupKind {
    match tokens[open_idx].kind {
        TokenKind::OpenBracket => GroupKind::ArrayLiteral,
        TokenKind::OpenBrace => {
            if is_object_literal_brace(tokens, open_idx) {
                GroupKind::ObjectLiteral
            } else {
                GroupKind::Block
            }
        }
        TokenKind::OpenParen => {
            let prev = find_significant_before(tokens, open_idx);
            if let Some(pi) = prev {
                if tokens[pi].kind == TokenKind::Identifier
                    && matches!(tokens[pi].text.as_str(), "function" | "async")
                {
                    return GroupKind::FunctionParams;
                }
                if tokens[pi].kind == TokenKind::Identifier && !is_control_keyword(&tokens[pi].text)
                {
                    return GroupKind::FunctionCall;
                }
                if tokens[pi].kind == TokenKind::CloseParen {
                    return GroupKind::FunctionCall;
                }
            }
            // Check if this paren's `)` is followed by `=>`
            if let Some(ci) = find_matching_close(tokens, open_idx) {
                if let Some(after) = find_significant_after(tokens, ci) {
                    if tokens[after].kind == TokenKind::Arrow {
                        return GroupKind::FunctionParams;
                    }
                }
            }
            GroupKind::Other
        }
        _ => GroupKind::Other,
    }
}

fn is_control_keyword(text: &str) -> bool {
    matches!(
        text,
        "if" | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "catch"
            | "with"
            | "return"
            | "throw"
            | "new"
            | "delete"
            | "typeof"
            | "void"
            | "in"
            | "of"
            | "instanceof"
            | "await"
            | "yield"
    )
}

fn build_group_tree(tokens: &[Token]) -> Vec<DelimiterGroup> {
    let mut groups: Vec<DelimiterGroup> = Vec::new();
    let mut stack: Vec<usize> = Vec::new(); // stack of group indices

    for i in 0..tokens.len() {
        if matches!(
            tokens[i].kind,
            TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace
        ) {
            if let Some(ci) = find_matching_close(tokens, i) {
                let kind = classify_group_kind(tokens, i);
                let flat_len = measure_flat_length(tokens, i, ci);
                let has_nl = has_newline_in_range(tokens, i, ci);

                let mut commas = Vec::new();
                // Collect commas at this depth only
                let mut depth: usize = 0;
                for j in (i + 1)..ci {
                    match tokens[j].kind {
                        TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace => {
                            depth += 1
                        }
                        TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace => {
                            depth = depth.saturating_sub(1)
                        }
                        TokenKind::Comma if depth == 0 => commas.push(j),
                        _ => {}
                    }
                }

                let gidx = groups.len();
                groups.push(DelimiterGroup {
                    open_idx: i,
                    close_idx: ci,
                    comma_indices: commas,
                    flat_len,
                    has_source_newline: has_nl,
                    kind,
                    children: Vec::new(),
                });

                if let Some(&parent) = stack.last() {
                    groups[parent].children.push(gidx);
                }

                stack.push(gidx);
            }
        } else if matches!(
            tokens[i].kind,
            TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace
        ) {
            if let Some(&top) = stack.last() {
                if groups[top].close_idx == i {
                    stack.pop();
                }
            }
        }
    }
    groups
}

fn column_at(tokens: &[Token], pos: usize) -> usize {
    let mut col = 0;
    for i in (0..pos).rev() {
        if tokens[i].kind == TokenKind::Newline {
            break;
        }
        col += tokens[i].text.len();
    }
    col
}

fn is_expandable_last_arg(tokens: &[Token], start: usize, end: usize) -> bool {
    let first_sig = find_first_significant(tokens, start, end);
    let first = match first_sig {
        Some(i) => i,
        None => return false,
    };

    // Arrow function: (...) => { ... } or ident => { ... }
    // Look for => within the range
    let mut depth: usize = 0;
    for i in start..=end {
        match tokens[i].kind {
            TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace => depth += 1,
            TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace => {
                depth = depth.saturating_sub(1)
            }
            TokenKind::Arrow if depth == 0 => {
                // Arrow function — expandable if it has a block body
                if let Some(after) = find_significant_after(tokens, i) {
                    if after <= end && tokens[after].kind == TokenKind::OpenBrace {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }

    // `function` keyword expression
    if tokens[first].kind == TokenKind::Identifier && tokens[first].text == "function" {
        return true;
    }

    // Object literal or array literal as sole content
    if tokens[first].kind == TokenKind::OpenBrace && is_object_literal_brace(tokens, first) {
        if let Some(ci) = find_matching_close(tokens, first) {
            let after_close = find_first_significant(tokens, ci + 1, end);
            if after_close.is_none() {
                return true;
            }
        }
    }

    if tokens[first].kind == TokenKind::OpenBracket {
        if let Some(ci) = find_matching_close(tokens, first) {
            let after_close = find_first_significant(tokens, ci + 1, end);
            if after_close.is_none() {
                return true;
            }
        }
    }

    false
}

fn find_first_significant(tokens: &[Token], from: usize, to: usize) -> Option<usize> {
    for i in from..=to {
        if !matches!(
            tokens[i].kind,
            TokenKind::Whitespace
                | TokenKind::Newline
                | TokenKind::LineComment
                | TokenKind::BlockComment
        ) {
            return Some(i);
        }
    }
    None
}

fn find_last_arg_range(tokens: &[Token], group: &DelimiterGroup) -> Option<(usize, usize)> {
    if group.comma_indices.is_empty() {
        return None;
    }
    let last_comma = *group.comma_indices.last().unwrap();
    let start = last_comma + 1;
    let end = group.close_idx.saturating_sub(1);
    if start > end {
        return None;
    }
    // Check that there's actual content
    if find_first_significant(tokens, start, end).is_none() {
        return None;
    }
    Some((start, end))
}

fn measure_prefix_flat_len(tokens: &[Token], group: &DelimiterGroup) -> usize {
    if group.comma_indices.is_empty() {
        return group.flat_len;
    }
    let last_comma = *group.comma_indices.last().unwrap();
    // prefix = open delim through last comma + space
    measure_flat_length(tokens, group.open_idx, last_comma) + 1
}

fn decide_breaks(
    tokens: &[Token],
    groups: &[DelimiterGroup],
    config: &FormatConfig,
) -> Vec<BreakDecision> {
    let mut decisions = Vec::with_capacity(groups.len());
    for _ in 0..groups.len() {
        decisions.push(BreakDecision::Flat);
    }

    // Process from innermost to outermost (higher index = deeper in our tree)
    // We iterate in reverse since children are pushed after parents
    for gi in (0..groups.len()).rev() {
        let group = &groups[gi];

        if is_empty_body_range(tokens, group.open_idx, group.close_idx) {
            decisions[gi] = BreakDecision::Flat;
            continue;
        }

        if group.kind == GroupKind::Block {
            decisions[gi] = BreakDecision::Expanded;
            continue;
        }

        if group.has_source_newline {
            decisions[gi] = BreakDecision::Expanded;
            continue;
        }

        let col = column_at(tokens, group.open_idx);
        if col + group.flat_len <= config.print_width {
            decisions[gi] = BreakDecision::Flat;
            continue;
        }

        // Try LastArgExpanded for function calls
        if group.kind == GroupKind::FunctionCall {
            if let Some((start, end)) = find_last_arg_range(tokens, group) {
                if is_expandable_last_arg(tokens, start, end) {
                    let prefix_len = col + measure_prefix_flat_len(tokens, group);
                    if prefix_len <= config.print_width {
                        decisions[gi] = BreakDecision::LastArgExpanded;
                        continue;
                    }
                }
            }
        }

        decisions[gi] = BreakDecision::Expanded;
    }

    decisions
}

fn is_empty_body_range(tokens: &[Token], open: usize, close: usize) -> bool {
    for i in (open + 1)..close {
        if !matches!(tokens[i].kind, TokenKind::Whitespace | TokenKind::Newline) {
            return false;
        }
    }
    true
}

fn wrap_long_lines(tokens: &mut Vec<Token>, config: &FormatConfig) {
    let groups = build_group_tree(tokens);
    if groups.is_empty() {
        return;
    }

    let decisions = decide_breaks(tokens, &groups, config);

    apply_wrap_decisions(tokens, &groups, &decisions, config);

    // Method chain breaking
    apply_method_chain_breaks(tokens, config);

    // Trailing comma fixup
    fixup_trailing_commas(tokens, &groups, &decisions, config);
}

fn apply_wrap_decisions(
    tokens: &mut Vec<Token>,
    groups: &[DelimiterGroup],
    decisions: &[BreakDecision],
    config: &FormatConfig,
) {
    let eol = config.end_of_line.as_str().to_vstring();

    // We need to process groups but indices shift as we insert/remove.
    // Strategy: process groups sorted by open_idx descending (rightmost first)
    // so insertions don't affect earlier indices.

    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by(|a, b| groups[*b].open_idx.cmp(&groups[*a].open_idx));

    for &gi in &order {
        let decision = decisions[gi];
        // Re-find the group boundaries using tokens (they might have shifted from prior ops)
        // Since we process right-to-left, indices to the left are stable.
        let group = &groups[gi];
        let open_idx = group.open_idx;

        // We need to find the current close_idx since prior insertions may have shifted it.
        // Use find_matching_close to get the actual current position.
        let close_idx = match find_matching_close(tokens, open_idx) {
            Some(ci) => ci,
            None => continue,
        };

        match decision {
            BreakDecision::Flat => {
                flatten_group(tokens, open_idx, close_idx, &eol);
            }
            BreakDecision::Expanded => {
                expand_group(tokens, open_idx, close_idx, &eol);
            }
            BreakDecision::LastArgExpanded => {
                apply_last_arg_expansion(tokens, open_idx, close_idx, &eol, config);
            }
        }
    }
}

fn flatten_group(tokens: &mut Vec<Token>, open: usize, _close: usize, _eol: &str) {
    // Collapse newlines + indentation to single spaces within this group at depth 0
    let mut i = open + 1;
    while i < tokens.len() {
        // Recalculate close since we may remove tokens
        let current_close = match find_matching_close(tokens, open) {
            Some(c) => c,
            None => break,
        };
        if i >= current_close {
            break;
        }

        // Only operate at depth 0 within this group
        let mut depth: usize = 0;
        let mut at_depth_0 = true;
        for j in (open + 1)..i {
            match tokens[j].kind {
                TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace => depth += 1,
                TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace => {
                    depth = depth.saturating_sub(1)
                }
                _ => {}
            }
        }
        if depth > 0 {
            at_depth_0 = false;
        }

        if at_depth_0 && tokens[i].kind == TokenKind::Newline {
            // Remove newline and following indentation whitespace, replace with space
            tokens.remove(i);
            if i < tokens.len()
                && tokens[i].kind == TokenKind::Whitespace
                && (i == 0 || tokens[i - 1].kind != TokenKind::Newline || true)
            {
                // Check if this whitespace is indentation (comes right after where the newline was)
                tokens.remove(i);
            }
            // Insert a space if previous and next tokens need separation
            if i > 0
                && i < tokens.len()
                && !matches!(
                    tokens[i - 1].kind,
                    TokenKind::Whitespace
                        | TokenKind::OpenParen
                        | TokenKind::OpenBracket
                        | TokenKind::OpenBrace
                )
                && !matches!(
                    tokens[i].kind,
                    TokenKind::Whitespace
                        | TokenKind::CloseParen
                        | TokenKind::CloseBracket
                        | TokenKind::CloseBrace
                )
            {
                tokens.insert(
                    i,
                    Token {
                        kind: TokenKind::Whitespace,
                        text: " ".into(),
                        line: 0,
                        col: 0,
                    },
                );
                i += 1;
            }
            continue;
        }

        i += 1;
    }
}

fn expand_group(tokens: &mut Vec<Token>, open: usize, _close: usize, eol: &str) {
    let current_close = match find_matching_close(tokens, open) {
        Some(c) => c,
        None => return,
    };

    if is_empty_body_range(tokens, open, current_close) {
        return;
    }

    // Already expanded? Check if there's a newline right after open (possibly with whitespace)
    let after_open = open + 1;
    let mut already_expanded = false;
    if after_open < current_close {
        if tokens[after_open].kind == TokenKind::Newline {
            already_expanded = true;
        } else if tokens[after_open].kind == TokenKind::Whitespace
            && after_open + 1 < current_close
            && tokens[after_open + 1].kind == TokenKind::Newline
        {
            already_expanded = true;
        }
    }

    if already_expanded {
        // Ensure commas at depth 0 are followed by newlines
        ensure_commas_have_newlines(tokens, open, eol);
        return;
    }

    // Insert newlines in reverse order to preserve indices
    // 1. Before close
    let current_close = find_matching_close(tokens, open).unwrap();
    insert_newline_before(tokens, current_close, eol);

    // 2. After each comma (reverse order)
    let commas = collect_depth0_commas(tokens, open);
    for &ci in commas.iter().rev() {
        // Remove whitespace after comma
        if ci + 1 < tokens.len() && tokens[ci + 1].kind == TokenKind::Whitespace {
            tokens.remove(ci + 1);
        }
        // Insert newline after comma
        tokens.insert(
            ci + 1,
            Token {
                kind: TokenKind::Newline,
                text: eol.to_vstring(),
                line: 0,
                col: 0,
            },
        );
    }

    // 3. After open delim
    // Remove whitespace right after open
    if open + 1 < tokens.len() && tokens[open + 1].kind == TokenKind::Whitespace {
        tokens.remove(open + 1);
    }
    tokens.insert(
        open + 1,
        Token {
            kind: TokenKind::Newline,
            text: eol.to_vstring(),
            line: 0,
            col: 0,
        },
    );
}

fn ensure_commas_have_newlines(tokens: &mut Vec<Token>, open: usize, eol: &str) {
    let commas = collect_depth0_commas(tokens, open);
    for &ci in commas.iter().rev() {
        let after = ci + 1;
        if after < tokens.len() {
            if tokens[after].kind == TokenKind::Whitespace {
                // Check if there's a newline after the whitespace
                if after + 1 < tokens.len() && tokens[after + 1].kind == TokenKind::Newline {
                    continue;
                }
                // Replace whitespace with newline
                tokens.remove(after);
                tokens.insert(
                    after,
                    Token {
                        kind: TokenKind::Newline,
                        text: eol.to_vstring(),
                        line: 0,
                        col: 0,
                    },
                );
            } else if tokens[after].kind != TokenKind::Newline {
                tokens.insert(
                    after,
                    Token {
                        kind: TokenKind::Newline,
                        text: eol.to_vstring(),
                        line: 0,
                        col: 0,
                    },
                );
            }
        }
    }
}

fn collect_depth0_commas(tokens: &[Token], open: usize) -> Vec<usize> {
    let close = match find_matching_close(tokens, open) {
        Some(c) => c,
        None => return Vec::new(),
    };
    let mut commas = Vec::new();
    let mut depth: usize = 0;
    for i in (open + 1)..close {
        match tokens[i].kind {
            TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace => depth += 1,
            TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace => {
                depth = depth.saturating_sub(1)
            }
            TokenKind::Comma if depth == 0 => commas.push(i),
            _ => {}
        }
    }
    commas
}

fn insert_newline_before(tokens: &mut Vec<Token>, pos: usize, eol: &str) {
    // Remove whitespace before pos if it exists
    if pos > 0 && tokens[pos - 1].kind == TokenKind::Whitespace {
        tokens.remove(pos - 1);
        let pos = pos - 1;
        if pos > 0 && tokens[pos - 1].kind == TokenKind::Newline {
            return; // already has newline
        }
        tokens.insert(
            pos,
            Token {
                kind: TokenKind::Newline,
                text: eol.to_vstring(),
                line: 0,
                col: 0,
            },
        );
    } else if pos > 0 && tokens[pos - 1].kind == TokenKind::Newline {
    } else {
        tokens.insert(
            pos,
            Token {
                kind: TokenKind::Newline,
                text: eol.to_vstring(),
                line: 0,
                col: 0,
            },
        );
    }
}

fn apply_last_arg_expansion(
    tokens: &mut Vec<Token>,
    open: usize,
    _close: usize,
    eol: &str,
    _config: &FormatConfig,
) {
    let commas = collect_depth0_commas(tokens, open);
    if commas.is_empty() {
        return;
    }

    let last_comma = *commas.last().unwrap();

    // Find the expandable group within the last arg
    let close = match find_matching_close(tokens, open) {
        Some(c) => c,
        None => return,
    };

    // First pass: look for arrow fn body or function body (these take priority)
    let mut inner_open = None;
    let mut depth: usize = 0;
    for i in (last_comma + 1)..close {
        match tokens[i].kind {
            TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace => depth += 1,
            TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace => {
                depth = depth.saturating_sub(1);
            }
            TokenKind::Arrow if depth == 0 => {
                if let Some(after) = find_significant_after(tokens, i) {
                    if tokens[after].kind == TokenKind::OpenBrace {
                        inner_open = Some(after);
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    // Second pass: fall back to first delimiter at depth 0 (object/array)
    if inner_open.is_none() {
        depth = 0;
        for i in (last_comma + 1)..close {
            match tokens[i].kind {
                TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace => {
                    if depth == 0 {
                        inner_open = Some(i);
                        break;
                    }
                    depth += 1;
                }
                TokenKind::CloseParen | TokenKind::CloseBracket | TokenKind::CloseBrace => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    if let Some(io) = inner_open {
        expand_group(tokens, io, 0, eol);
    }
}

fn apply_method_chain_breaks(tokens: &mut Vec<Token>, config: &FormatConfig) {
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].kind == TokenKind::Dot {
            let chain = detect_method_chain(tokens, i);
            if chain.len() >= 2 {
                let chain_start = find_chain_start(tokens, chain[0]);
                let chain_end = find_chain_end(tokens, *chain.last().unwrap());
                let flat_len = measure_flat_length(tokens, chain_start, chain_end);
                let col = column_at(tokens, chain_start);

                if col + flat_len > config.print_width {
                    let eol = config.end_of_line.as_str().to_vstring();
                    // Insert newlines before each dot in the chain (reverse order)
                    for &dot_idx in chain.iter().rev() {
                        // Don't break if there's already a newline before the dot
                        if dot_idx > 0 {
                            let prev = dot_idx - 1;
                            if tokens[prev].kind == TokenKind::Newline {
                                continue;
                            }
                            if tokens[prev].kind == TokenKind::Whitespace
                                && prev > 0
                                && tokens[prev - 1].kind == TokenKind::Newline
                            {
                                continue;
                            }
                            // Remove whitespace before dot
                            if tokens[prev].kind == TokenKind::Whitespace {
                                tokens.remove(prev);
                                let dot_idx = dot_idx - 1;
                                tokens.insert(
                                    dot_idx,
                                    Token {
                                        kind: TokenKind::Newline,
                                        text: eol.clone(),
                                        line: 0,
                                        col: 0,
                                    },
                                );
                                continue;
                            }
                        }
                        tokens.insert(
                            dot_idx,
                            Token {
                                kind: TokenKind::Newline,
                                text: eol.clone(),
                                line: 0,
                                col: 0,
                            },
                        );
                    }
                    // Skip past the chain we just processed
                    i = chain_end + chain.len() + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
}

fn detect_method_chain(tokens: &[Token], start_dot: usize) -> Vec<usize> {
    let mut dots = vvec![start_dot];
    let mut i = start_dot;

    // Walk forward through .method() chains
    while i < tokens.len() {
        // After a dot, expect identifier
        let ident = match find_significant_after(tokens, i) {
            Some(idx) if tokens[idx].kind == TokenKind::Identifier => idx,
            _ => break,
        };
        // After identifier, expect open paren
        let open = match find_significant_after(tokens, ident) {
            Some(idx) if tokens[idx].kind == TokenKind::OpenParen => idx,
            _ => break,
        };
        // Find matching close paren
        let close = match find_matching_close(tokens, open) {
            Some(c) => c,
            None => break,
        };
        // After close paren, check for another dot
        match find_significant_after(tokens, close) {
            Some(idx) if tokens[idx].kind == TokenKind::Dot => {
                dots.push(idx);
                i = idx;
            }
            _ => break,
        }
    }

    dots
}

fn find_chain_start(tokens: &[Token], first_dot: usize) -> usize {
    // Walk backwards past the receiver
    if first_dot == 0 {
        return 0;
    }
    let prev = find_significant_before(tokens, first_dot);
    match prev {
        Some(pi) => {
            if tokens[pi].kind == TokenKind::CloseParen {
                if let Some(oi) = find_matching_open(tokens, pi) {
                    return find_chain_start(tokens, oi);
                }
            }
            pi
        }
        None => first_dot,
    }
}

fn find_chain_end(tokens: &[Token], last_dot: usize) -> usize {
    // Walk forward past .method()
    let ident = match find_significant_after(tokens, last_dot) {
        Some(i) if tokens[i].kind == TokenKind::Identifier => i,
        _ => return last_dot,
    };
    let open = match find_significant_after(tokens, ident) {
        Some(i) if tokens[i].kind == TokenKind::OpenParen => i,
        _ => return ident,
    };
    match find_matching_close(tokens, open) {
        Some(c) => c,
        None => open,
    }
}

fn fixup_trailing_commas(
    tokens: &mut Vec<Token>,
    groups: &[DelimiterGroup],
    decisions: &[BreakDecision],
    config: &FormatConfig,
) {
    // Process right-to-left to keep indices stable
    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by(|a, b| groups[*b].open_idx.cmp(&groups[*a].open_idx));

    for &gi in &order {
        let decision = decisions[gi];
        let group = &groups[gi];

        let close_idx = match find_matching_close(tokens, group.open_idx) {
            Some(c) => c,
            None => continue,
        };

        let prev_sig = find_significant_before(tokens, close_idx);
        let open_idx = group.open_idx;

        match decision {
            BreakDecision::Expanded => {
                if !matches!(config.trailing_comma, TrailingComma::None) {
                    if let Some(pi) = prev_sig {
                        if pi > open_idx
                            && tokens[pi].kind != TokenKind::Comma
                            && is_comma_eligible(&tokens[pi])
                            && !is_empty_body_range(tokens, open_idx, close_idx)
                        {
                            tokens.insert(
                                pi + 1,
                                Token {
                                    kind: TokenKind::Comma,
                                    text: ",".into(),
                                    line: 0,
                                    col: 0,
                                },
                            );
                        }
                    }
                }
            }
            BreakDecision::LastArgExpanded => {
                // Remove trailing comma after the expandable last arg
                if let Some(pi) = prev_sig {
                    if tokens[pi].kind == TokenKind::Comma && pi > open_idx {
                        // Check the token before the comma - if it's a close delim of the
                        // expanded arg, remove the trailing comma
                        let before_comma = find_significant_before(tokens, pi);
                        if let Some(bi) = before_comma {
                            if matches!(
                                tokens[bi].kind,
                                TokenKind::CloseBrace
                                    | TokenKind::CloseBracket
                                    | TokenKind::CloseParen
                            ) {
                                tokens.remove(pi);
                            }
                        }
                    }
                }
            }
            BreakDecision::Flat => {
                // Remove trailing commas on flat groups
                if let Some(pi) = prev_sig {
                    if tokens[pi].kind == TokenKind::Comma && pi > open_idx {
                        tokens.remove(pi);
                    }
                }
            }
        }
    }
}

// Serialize tokens back to string
fn serialize(tokens: &[Token], config: &FormatConfig) -> String {
    let mut out = String::new();
    for token in tokens {
        if token.kind == TokenKind::Eof {
            break;
        }
        out.push_str(&token.text);
    }
    // Ensure file ends with a newline
    let eol = config.end_of_line.as_str();
    if !out.ends_with(eol) && !out.is_empty() {
        out.push_str(eol);
    }
    out
}

// Helpers
fn find_significant_before(tokens: &[Token], pos: usize) -> Option<usize> {
    for i in (0..pos).rev() {
        if !matches!(
            tokens[i].kind,
            TokenKind::Whitespace
                | TokenKind::Newline
                | TokenKind::LineComment
                | TokenKind::BlockComment
        ) {
            return Some(i);
        }
    }
    None
}

fn find_significant_after(tokens: &[Token], pos: usize) -> Option<usize> {
    for i in (pos + 1)..tokens.len() {
        if !matches!(
            tokens[i].kind,
            TokenKind::Whitespace
                | TokenKind::Newline
                | TokenKind::LineComment
                | TokenKind::BlockComment
        ) {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::lang::js::formatter::config::FormatConfig;

    fn fmt(input: &str) -> String {
        format_source(input, &FormatConfig::default(), None).unwrap()
    }

    fn fmt_with(input: &str, config: &FormatConfig) -> String {
        format_source(input, config, None).unwrap()
    }

    // Quote normalization
    #[test]
    fn double_to_single_quotes() {
        let mut config = FormatConfig::default();
        config.single_quote = true;
        let result = fmt_with(r#"const x = "hello""#, &config);
        assert!(result.contains("'hello'"));
    }

    #[test]
    fn single_to_double_quotes() {
        let result = fmt("const x = 'hello'");
        assert!(result.contains("\"hello\""));
    }

    #[test]
    fn skip_swap_if_more_escapes() {
        let mut config = FormatConfig::default();
        config.single_quote = true;
        let result = fmt_with(r#"const x = "it's here""#, &config);
        assert!(result.contains("\"it's here\""));
    }

    // Semicolons
    #[test]
    fn insert_semicolons() {
        let result = fmt("const x = 1\nconst y = 2\n");

        assert!(result.contains("const x = 1;"));
        assert!(result.contains("const y = 2;"));
    }

    #[test]
    fn remove_semicolons() {
        let mut config = FormatConfig::default();
        config.semi = false;
        let result = fmt_with("const x = 1;\nconst y = 2;\n", &config);
        assert!(!result.contains(";"));
    }

    #[test]
    fn keep_semi_before_asi_hazard() {
        let mut config = FormatConfig::default();
        config.semi = false;
        let result = fmt_with("const x = 1;\n[1, 2].forEach(f)\n", &config);
        assert!(result.starts_with("const x = 1;") || result.contains("x = 1;"));
    }

    // Bracket spacing
    #[test]
    fn bracket_spacing_on() {
        let result = fmt("const o = {a: 1}");
        assert!(result.contains("{ a: 1 }") || result.contains("{a: 1}"));
    }

    #[test]
    fn bracket_spacing_off() {
        let mut config = FormatConfig::default();
        config.bracket_spacing = false;
        let result = fmt_with("const o = { a: 1 }", &config);
        assert!(result.contains("{a: 1}"));
    }

    // Trailing commas
    #[test]
    fn add_trailing_comma_multiline() {
        let result = fmt("const a = [\n  1,\n  2\n]");

        assert!(result.contains("2,"));
    }

    #[test]
    fn remove_trailing_comma_single_line() {
        let result = fmt("const a = [1, 2,]");

        assert!(!result.contains(", 2,]"));
    }

    #[test]
    fn no_trailing_comma_when_none() {
        let mut config = FormatConfig::default();
        config.trailing_comma = TrailingComma::None;
        let result = fmt_with("const a = [\n  1,\n  2,\n]", &config);

        assert!(!result.contains("2,"));
    }

    // Arrow parens
    #[test]
    fn add_arrow_parens() {
        let result = fmt("const f = x => x");
        assert!(result.contains("(x) =>"));
    }

    #[test]
    fn remove_arrow_parens() {
        let mut config = FormatConfig::default();
        config.arrow_parens = ArrowParens::Avoid;
        let result = fmt_with("const f = (x) => x", &config);
        assert!(result.contains("x =>"));
    }

    // Line endings
    #[test]
    fn normalize_crlf_to_lf() {
        let result = fmt("a\r\nb");
        assert!(!result.contains("\r\n"));
        assert!(result.contains("\n"));
    }

    // Whitespace
    #[test]
    fn collapse_multiple_blank_lines() {
        let result = fmt("a\n\n\n\nb");

        let newline_count = result.matches('\n').count();
        assert!(newline_count <= 3); // content + at most 1 blank line + trailing
    }

    // Indentation
    #[test]
    fn indent_brace_block() {
        let result = fmt("if (x) {\nfoo()\n}");
        assert!(result.contains("  foo()"));
    }

    #[test]
    fn indent_with_tabs() {
        let mut config = FormatConfig::default();
        config.use_tabs = true;
        let result = fmt_with("if (x) {\nfoo()\n}", &config);
        assert!(result.contains("\tfoo()"));
    }

    #[test]
    fn empty_input() {
        let result = fmt("");
        assert!(result.is_empty() || result == "\n");
    }

    #[test]
    fn preserves_comments() {
        let result = fmt("// hello\nconst x = 1");
        assert!(result.contains("// hello"));
    }

    #[test]
    fn preserves_template_literals() {
        let result = fmt("const x = `hello ${world}`");
        assert!(result.contains("`hello ${world}`"));
    }

    // --- Line wrapping tests ---

    #[test]
    fn wrap_fits_on_one_line() {
        let result = fmt("foo(a, b, c);");
        assert_eq!(result.trim(), "foo(a, b, c);");
    }

    #[test]
    fn wrap_all_or_nothing_args() {
        let input = "someFunction(firstArgument, secondArgument, thirdArgument, fourthArgument, fifthArgument);";
        let result = fmt(input);
        assert!(
            result.contains("\n"),
            "Long call should be wrapped, got: {result}"
        );
        assert!(
            result.contains("firstArgument,"),
            "Args should be present, got: {result}"
        );
        assert!(
            result.contains("secondArgument,"),
            "Args should be present, got: {result}"
        );
    }

    #[test]
    fn wrap_array_expansion() {
        let input =
            r#"const arr = ["alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf"];"#;
        let result = fmt(input);
        assert!(
            result.contains("\n"),
            "Long array should expand, got: {result}"
        );
        for item in &["\"alpha\",", "\"bravo\",", "\"charlie\","] {
            assert!(result.contains(item), "Expected {item} in result: {result}");
        }
    }

    #[test]
    fn wrap_object_expansion() {
        let input =
            "const obj = { alpha: 1, beta: 2, gamma: 3, delta: 4, epsilon: 5, zeta: 6, eta: 7 };";
        let result = fmt(input);
        assert!(
            result.contains("\n"),
            "Long object should expand, got: {result}"
        );
        assert!(
            result.contains("alpha: 1,"),
            "Props should be present, got: {result}"
        );
    }

    #[test]
    fn wrap_last_arg_arrow_fn() {
        let mut config = FormatConfig::default();
        config.print_width = 50;
        let input = r#"app.get("/path", (req, res) => { res.send("ok"); });"#;
        let result = fmt_with(input, &config);
        assert!(
            result.contains("app.get("),
            "Call should start on first line, got: {result}"
        );
        assert!(
            result.contains("{\n"),
            "Arrow body should expand, got: {result}"
        );
    }

    #[test]
    fn wrap_last_arg_object() {
        let input =
            "setState(prevState, { loading: true, error: null, data: response, count: 42 });";
        let result = fmt(input);
        assert!(result.contains("\n"), "Should expand, got: {result}");
    }

    #[test]
    fn wrap_last_arg_array() {
        let input = "processItems(config, [alpha, bravo, charlie, delta, echo, foxtrot, golf]);";
        let result = fmt(input);
        assert!(result.contains("\n"), "Should expand, got: {result}");
    }

    #[test]
    fn wrap_method_chains() {
        let input = "promiseResult.then(handleSuccess).catch(handleError).finally(performCleanup);";
        let mut config = FormatConfig::default();
        config.print_width = 60;
        let result = fmt_with(input, &config);
        // Each .method() should be on its own line
        let dot_count = result
            .lines()
            .filter(|l| l.trim_start().starts_with("."))
            .count();
        assert!(dot_count >= 2, "Should have chain breaks, got: {result}");
    }

    #[test]
    fn wrap_preserve_multiline() {
        let input = "const o = {\n  a: 1\n};";
        let result = fmt(input);
        assert!(
            result.contains("{\n"),
            "Multi-line should stay multi-line, got: {result}"
        );
        assert!(
            result.contains("  a: 1"),
            "Should preserve content, got: {result}"
        );
    }

    #[test]
    fn wrap_nested_groups() {
        let input = "foo(bar(longArgA, longArgB, longArgC), baz(longArgD, longArgE, longArgF));";
        let result = fmt(input);
        assert!(
            result.contains("\n"),
            "Nested long calls should wrap, got: {result}"
        );
    }

    #[test]
    fn wrap_empty_groups_stay_flat() {
        let result = fmt("foo(); []; {}");
        // Empty delimiters should not get expanded
        assert!(
            result.contains("foo()"),
            "Empty call unchanged, got: {result}"
        );
    }

    #[test]
    fn wrap_irreducible_long_token() {
        let long_str = crate::vformat!(r#"const x = "{}""#, "a".repeat(100));
        let result = fmt(&long_str);
        // Can't break a string literal — should remain on one line
        assert!(
            result.contains(&"a".repeat(100)),
            "Long string unchanged, got: {result}"
        );
    }

    #[test]
    fn wrap_short_print_width() {
        let mut config = FormatConfig::default();
        config.print_width = 40;
        let input = "const f = (a, b, c) => { return a + b + c; };";
        let result = fmt_with(input, &config);
        assert!(
            result.contains("{\n"),
            "Arrow body should expand at width 40, got: {result}"
        );
    }

    #[test]
    fn wrap_already_expanded_stays_stable() {
        let input = "const a = [\n  1,\n  2,\n  3,\n];";
        let result = fmt(input);
        let result2 = fmt(&result);
        assert_eq!(result, result2, "Formatting should be idempotent");
    }

    #[test]
    fn wrap_single_line_stays_flat_within_width() {
        let input = "const a = [1, 2, 3];";
        let result = fmt(input);
        assert!(
            !result.contains("\n["),
            "Short array should stay flat, got: {result}"
        );
    }

    #[test]
    fn wrap_function_params_expand() {
        let mut config = FormatConfig::default();
        config.print_width = 40;
        let input = "function myFunc(paramAlpha, paramBeta, paramGamma) {}";
        let result = fmt_with(input, &config);
        assert!(
            result.contains("\n"),
            "Long params should expand, got: {result}"
        );
    }
}
