use super::config::FormatConfig;
use super::tokenizer::{Token, TokenKind};
use crate::core::plugins::protocol::JsonOut;
use crate::core::volkiwithstds::collections::ToString;
use crate::core::volkiwithstds::collections::Vec;
use crate::core::volkiwithstds::collections::json::JsonValue;
use crate::vvec;

pub fn token_kind_str(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::StringLiteral => "StringLiteral",
        TokenKind::TemplateLiteral => "TemplateLiteral",
        TokenKind::TemplateHead => "TemplateHead",
        TokenKind::TemplateMiddle => "TemplateMiddle",
        TokenKind::TemplateTail => "TemplateTail",
        TokenKind::NumericLiteral => "NumericLiteral",
        TokenKind::RegexLiteral => "RegexLiteral",
        TokenKind::Identifier => "Identifier",
        TokenKind::OpenParen => "OpenParen",
        TokenKind::CloseParen => "CloseParen",
        TokenKind::OpenBrace => "OpenBrace",
        TokenKind::CloseBrace => "CloseBrace",
        TokenKind::OpenBracket => "OpenBracket",
        TokenKind::CloseBracket => "CloseBracket",
        TokenKind::Semicolon => "Semicolon",
        TokenKind::Comma => "Comma",
        TokenKind::Dot => "Dot",
        TokenKind::Colon => "Colon",
        TokenKind::QuestionMark => "QuestionMark",
        TokenKind::Arrow => "Arrow",
        TokenKind::Spread => "Spread",
        TokenKind::Operator => "Operator",
        TokenKind::Assignment => "Assignment",
        TokenKind::LineComment => "LineComment",
        TokenKind::BlockComment => "BlockComment",
        TokenKind::Whitespace => "Whitespace",
        TokenKind::Newline => "Newline",
        TokenKind::Eof => "Eof",
    }
}

pub fn token_kind_from_str(s: &str) -> Option<TokenKind> {
    match s {
        "StringLiteral" => Some(TokenKind::StringLiteral),
        "TemplateLiteral" => Some(TokenKind::TemplateLiteral),
        "TemplateHead" => Some(TokenKind::TemplateHead),
        "TemplateMiddle" => Some(TokenKind::TemplateMiddle),
        "TemplateTail" => Some(TokenKind::TemplateTail),
        "NumericLiteral" => Some(TokenKind::NumericLiteral),
        "RegexLiteral" => Some(TokenKind::RegexLiteral),
        "Identifier" => Some(TokenKind::Identifier),
        "OpenParen" => Some(TokenKind::OpenParen),
        "CloseParen" => Some(TokenKind::CloseParen),
        "OpenBrace" => Some(TokenKind::OpenBrace),
        "CloseBrace" => Some(TokenKind::CloseBrace),
        "OpenBracket" => Some(TokenKind::OpenBracket),
        "CloseBracket" => Some(TokenKind::CloseBracket),
        "Semicolon" => Some(TokenKind::Semicolon),
        "Comma" => Some(TokenKind::Comma),
        "Dot" => Some(TokenKind::Dot),
        "Colon" => Some(TokenKind::Colon),
        "QuestionMark" => Some(TokenKind::QuestionMark),
        "Arrow" => Some(TokenKind::Arrow),
        "Spread" => Some(TokenKind::Spread),
        "Operator" => Some(TokenKind::Operator),
        "Assignment" => Some(TokenKind::Assignment),
        "LineComment" => Some(TokenKind::LineComment),
        "BlockComment" => Some(TokenKind::BlockComment),
        "Whitespace" => Some(TokenKind::Whitespace),
        "Newline" => Some(TokenKind::Newline),
        "Eof" => Some(TokenKind::Eof),
        _ => None,
    }
}

pub fn tokens_to_json(tokens: &[Token]) -> JsonOut {
    JsonOut::Array(
        tokens
            .iter()
            .map(|t| {
                JsonOut::Object(vvec![
                    ("kind".into(), JsonOut::Str(token_kind_str(&t.kind).into())),
                    ("text".into(), JsonOut::Str(t.text.clone())),
                    ("line".into(), JsonOut::Int(t.line as i64)),
                    ("col".into(), JsonOut::Int(t.col as i64)),
                ])
            })
            .collect(),
    )
}

pub fn config_to_json(config: &FormatConfig) -> JsonOut {
    JsonOut::Object(vvec![
        (
            "print_width".into(),
            JsonOut::Int(config.print_width as i64)
        ),
        ("tab_width".into(), JsonOut::Int(config.tab_width as i64)),
        ("use_tabs".into(), JsonOut::Bool(config.use_tabs)),
        ("semi".into(), JsonOut::Bool(config.semi)),
        ("single_quote".into(), JsonOut::Bool(config.single_quote)),
        (
            "bracket_spacing".into(),
            JsonOut::Bool(config.bracket_spacing)
        ),
    ])
}

pub fn tokens_from_json(value: &JsonValue) -> Option<Vec<Token>> {
    let arr = value.as_array()?;
    let mut tokens = Vec::with_capacity(arr.len());
    for item in arr {
        let obj = item.as_object()?;
        let kind_str = obj.get("kind")?.as_str()?;
        let kind = token_kind_from_str(kind_str)?;
        let text = obj.get("text")?.as_str()?.to_vstring();
        tokens.push(Token {
            kind,
            text,
            line: 0,
            col: 0,
        });
    }
    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_token_kinds() {
        let kinds = [
            TokenKind::StringLiteral,
            TokenKind::TemplateLiteral,
            TokenKind::TemplateHead,
            TokenKind::TemplateMiddle,
            TokenKind::TemplateTail,
            TokenKind::NumericLiteral,
            TokenKind::RegexLiteral,
            TokenKind::Identifier,
            TokenKind::OpenParen,
            TokenKind::CloseParen,
            TokenKind::OpenBrace,
            TokenKind::CloseBrace,
            TokenKind::OpenBracket,
            TokenKind::CloseBracket,
            TokenKind::Semicolon,
            TokenKind::Comma,
            TokenKind::Dot,
            TokenKind::Colon,
            TokenKind::QuestionMark,
            TokenKind::Arrow,
            TokenKind::Spread,
            TokenKind::Operator,
            TokenKind::Assignment,
            TokenKind::LineComment,
            TokenKind::BlockComment,
            TokenKind::Whitespace,
            TokenKind::Newline,
            TokenKind::Eof,
        ];
        for kind in &kinds {
            let s = token_kind_str(kind);
            let back = token_kind_from_str(s).unwrap();
            assert_eq!(&back, kind);
        }
    }

    #[test]
    fn tokens_to_json_and_back() {
        let tokens = vvec![
            Token {
                kind: TokenKind::Identifier,
                text: "const".into(),
                line: 1,
                col: 1
            },
            Token {
                kind: TokenKind::Whitespace,
                text: " ".into(),
                line: 1,
                col: 6
            },
        ];
        let json_out = tokens_to_json(&tokens);
        let serialized = json_out.serialize();

        use crate::core::volkiwithstds::collections::json::extract_top_level;
        let wrapped = crate::vformat!(r#"{{"tokens":{serialized}}}"#);
        let map = extract_top_level(&wrapped);
        let parsed = tokens_from_json(map.get("tokens").unwrap()).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].kind, TokenKind::Identifier);
        assert_eq!(parsed[0].text, "const");
        assert_eq!(parsed[1].kind, TokenKind::Whitespace);
    }

    #[test]
    fn unknown_kind_returns_none() {
        assert!(token_kind_from_str("FooBar").is_none());
    }
}
