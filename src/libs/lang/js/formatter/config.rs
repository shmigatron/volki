#[derive(Debug, Clone, PartialEq)]
pub enum QuoteProps {
    AsNeeded,
    Consistent,
    Preserve,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrailingComma {
    All,
    Es5,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrowParens {
    Always,
    Avoid,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EndOfLine {
    Lf,
    Crlf,
    Cr,
    Auto,
}

impl EndOfLine {
    pub fn as_str(&self) -> &str {
        match self {
            EndOfLine::Lf => "\n",
            EndOfLine::Crlf => "\r\n",
            EndOfLine::Cr => "\r",
            EndOfLine::Auto => "\n",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub print_width: usize,
    pub tab_width: usize,
    pub use_tabs: bool,
    pub semi: bool,
    pub single_quote: bool,
    pub quote_props: QuoteProps,
    pub jsx_single_quote: bool,
    pub trailing_comma: TrailingComma,
    pub bracket_spacing: bool,
    pub bracket_same_line: bool,
    pub arrow_parens: ArrowParens,
    pub end_of_line: EndOfLine,
    pub single_attribute_per_line: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            print_width: 80,
            tab_width: 2,
            use_tabs: false,
            semi: true,
            single_quote: false,
            quote_props: QuoteProps::AsNeeded,
            jsx_single_quote: false,
            trailing_comma: TrailingComma::All,
            bracket_spacing: true,
            bracket_same_line: false,
            arrow_parens: ArrowParens::Always,
            end_of_line: EndOfLine::Lf,
            single_attribute_per_line: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_prettier_v3() {
        let c = FormatConfig::default();
        assert_eq!(c.print_width, 80);
        assert_eq!(c.tab_width, 2);
        assert!(!c.use_tabs);
        assert!(c.semi);
        assert!(!c.single_quote);
        assert_eq!(c.quote_props, QuoteProps::AsNeeded);
        assert!(!c.jsx_single_quote);
        assert_eq!(c.trailing_comma, TrailingComma::All);
        assert!(c.bracket_spacing);
        assert!(!c.bracket_same_line);
        assert_eq!(c.arrow_parens, ArrowParens::Always);
        assert_eq!(c.end_of_line, EndOfLine::Lf);
        assert!(!c.single_attribute_per_line);
    }

    #[test]
    fn end_of_line_str() {
        assert_eq!(EndOfLine::Lf.as_str(), "\n");
        assert_eq!(EndOfLine::Crlf.as_str(), "\r\n");
        assert_eq!(EndOfLine::Cr.as_str(), "\r");
        assert_eq!(EndOfLine::Auto.as_str(), "\n");
    }
}
