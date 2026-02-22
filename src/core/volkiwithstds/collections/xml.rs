use crate::core::volkiwithstds::collections::ToString;
use crate::core::volkiwithstds::collections::{String, Vec};

/// Lightweight XML reader for extracting tag contents and attribute values.
pub struct Xml<'a> {
    data: &'a str,
}

impl<'a> Xml<'a> {
    pub fn new(data: &'a str) -> Self {
        Xml { data }
    }

    /// Extract text content between `<tag>` and `</tag>` pairs.
    pub fn tag_contents(&self, tag: &str) -> Vec<String> {
        let open = crate::vformat!("<{tag}>");
        let close = crate::vformat!("</{tag}>");
        let mut results = Vec::new();
        let mut search_from = 0;

        while let Some(start) = self.data[search_from..].find(open.as_str()) {
            let content_start = search_from + start + open.len();
            if let Some(end) = self.data[content_start..].find(close.as_str()) {
                let content = self.data[content_start..content_start + end]
                    .trim()
                    .to_vstring();
                if !content.is_empty() {
                    results.push(content);
                }
                search_from = content_start + end + close.len();
            } else {
                break;
            }
        }

        results
    }

    /// Extract the first tag content, or `None` if absent.
    pub fn first_tag_content(&self, tag: &str) -> Option<String> {
        self.tag_contents(tag).into_iter().next()
    }

    /// Extract attribute values from tags like `<tag attr="value">` or `<tag attr="value" />`.
    #[allow(dead_code)]
    pub fn tag_attribute(&self, tag: &str, attr: &str) -> Vec<String> {
        let tag_open = crate::vformat!("<{tag} ");
        let mut results = Vec::new();
        let mut search_from = 0;

        while let Some(start) = self.data[search_from..].find(tag_open.as_str()) {
            let tag_start = search_from + start;
            if let Some(end_offset) = self.data[tag_start..].find('>') {
                let tag_content = &self.data[tag_start..tag_start + end_offset + 1];
                if let Some(val) = Self::attr_value(tag_content, attr) {
                    results.push(val);
                }
                search_from = tag_start + end_offset + 1;
            } else {
                break;
            }
        }

        results
    }

    /// Extract a single attribute value from a tag string like `attr="value"` or `attr='value'`.
    pub(crate) fn attr_value(tag_str: &str, attr: &str) -> Option<String> {
        let patterns = [crate::vformat!("{attr}=\""), crate::vformat!("{attr}='")];
        for pattern in &patterns {
            if let Some(start) = tag_str.find(pattern.as_str()) {
                let val_start = start + pattern.len();
                let quote = tag_str.as_bytes()[start + pattern.len() - 1];
                if let Some(end) = tag_str[val_start..].find(quote as char) {
                    return Some(tag_str[val_start..val_start + end].to_vstring());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vvec;

    #[test]
    fn tag_contents_simple() {
        let xml = Xml::new("<name>hello</name>");
        assert_eq!(xml.tag_contents("name"), vvec![crate::vstr!("hello")]);
    }

    #[test]
    fn tag_contents_multiple() {
        let xml = Xml::new("<item>a</item><item>b</item>");
        assert_eq!(
            xml.tag_contents("item"),
            vvec![crate::vstr!("a"), crate::vstr!("b")]
        );
    }

    #[test]
    fn tag_contents_empty_filtered() {
        let xml = Xml::new("<name>  </name>");
        assert!(xml.tag_contents("name").is_empty());
    }

    #[test]
    fn tag_contents_whitespace_trimmed() {
        let xml = Xml::new("<name>  hello  </name>");
        assert_eq!(xml.tag_contents("name"), vvec![crate::vstr!("hello")]);
    }

    #[test]
    fn tag_contents_no_match() {
        let xml = Xml::new("<other>val</other>");
        assert!(xml.tag_contents("name").is_empty());
    }

    #[test]
    fn tag_contents_unclosed() {
        let xml = Xml::new("<name>hello");
        assert!(xml.tag_contents("name").is_empty());
    }

    #[test]
    fn first_tag_content_some() {
        let xml = Xml::new("<v>1.0</v><v>2.0</v>");
        assert_eq!(xml.first_tag_content("v"), Some(crate::vstr!("1.0")));
    }

    #[test]
    fn first_tag_content_none() {
        let xml = Xml::new("<other>val</other>");
        assert_eq!(xml.first_tag_content("name"), None);
    }
}
