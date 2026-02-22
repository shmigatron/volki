//! Next.js-style metadata for HTML pages.

use super::element::{HtmlNode, meta, link};
use super::escape::escape_html;
use super::render::render_node;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::libs::web::http::request::Request;

/// Function type for generating metadata per-request.
pub type MetadataFn = fn(&Request) -> Metadata;

/// Robots directives for search engine crawlers.
pub struct Robots {
    pub index: bool,
    pub follow: bool,
    pub no_archive: bool,
    pub no_snippet: bool,
}

impl Robots {
    pub fn new() -> Self {
        Self {
            index: true,
            follow: true,
            no_archive: false,
            no_snippet: false,
        }
    }

    pub fn no_index(mut self) -> Self {
        self.index = false;
        self
    }

    pub fn no_follow(mut self) -> Self {
        self.follow = false;
        self
    }

    pub fn no_archive(mut self) -> Self {
        self.no_archive = true;
        self
    }

    pub fn no_snippet(mut self) -> Self {
        self.no_snippet = true;
        self
    }

    fn render_content(&self) -> String {
        let mut parts = Vec::new();
        if self.index {
            parts.push("index");
        } else {
            parts.push("noindex");
        }
        if self.follow {
            parts.push("follow");
        } else {
            parts.push("nofollow");
        }
        if self.no_archive {
            parts.push("noarchive");
        }
        if self.no_snippet {
            parts.push("nosnippet");
        }
        let mut out = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(part);
        }
        out
    }
}

/// Severity of a metadata validation issue.
pub enum MetadataWarning {
    UnknownOgType(String),
    UnknownTwitterCard(String),
    PartialOg(String),
    PartialTwitter(String),
    EmptyField(String),
}

/// Next.js-style metadata for HTML pages.
pub struct Metadata {
    // Basic
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,

    // Document
    pub charset: Option<String>,
    pub viewport: Option<String>,
    pub color_scheme: Option<String>,
    pub theme_color: Option<String>,

    // Canonical & robots
    pub canonical: Option<String>,
    pub robots: Option<Robots>,

    // Open Graph
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_type: Option<String>,
    pub og_url: Option<String>,
    pub og_image: Option<String>,
    pub og_site_name: Option<String>,
    pub og_locale: Option<String>,

    // Twitter
    pub twitter_card: Option<String>,
    pub twitter_title: Option<String>,
    pub twitter_description: Option<String>,
    pub twitter_image: Option<String>,
    pub twitter_site: Option<String>,

    // Icons
    pub favicon: Option<String>,
    pub apple_touch_icon: Option<String>,

    // Other
    pub author: Option<String>,
    pub generator: Option<String>,

    // Extensibility
    pub custom_meta: Vec<(String, String)>,
    pub custom_head: Vec<HtmlNode>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            keywords: None,
            charset: Some(String::from("utf-8")),
            viewport: Some(String::from("width=device-width, initial-scale=1")),
            color_scheme: None,
            theme_color: None,
            canonical: None,
            robots: None,
            og_title: None,
            og_description: None,
            og_type: None,
            og_url: None,
            og_image: None,
            og_site_name: None,
            og_locale: None,
            twitter_card: None,
            twitter_title: None,
            twitter_description: None,
            twitter_image: None,
            twitter_site: None,
            favicon: None,
            apple_touch_icon: None,
            author: None,
            generator: None,
            custom_meta: Vec::new(),
            custom_head: Vec::new(),
        }
    }

    /// Clear default charset and viewport.
    pub fn no_defaults(mut self) -> Self {
        self.charset = None;
        self.viewport = None;
        self
    }

    // ── Builder methods ──────────────────────────────────────────────

    pub fn title(mut self, v: &str) -> Self {
        self.title = Some(String::from(v));
        self
    }

    pub fn description(mut self, v: &str) -> Self {
        self.description = Some(String::from(v));
        self
    }

    pub fn keywords(mut self, v: &[&str]) -> Self {
        let mut kw = Vec::new();
        for k in v {
            kw.push(String::from(*k));
        }
        self.keywords = Some(kw);
        self
    }

    pub fn charset(mut self, v: &str) -> Self {
        self.charset = Some(String::from(v));
        self
    }

    pub fn viewport(mut self, v: &str) -> Self {
        self.viewport = Some(String::from(v));
        self
    }

    pub fn color_scheme(mut self, v: &str) -> Self {
        self.color_scheme = Some(String::from(v));
        self
    }

    pub fn theme_color(mut self, v: &str) -> Self {
        self.theme_color = Some(String::from(v));
        self
    }

    pub fn canonical(mut self, v: &str) -> Self {
        self.canonical = Some(String::from(v));
        self
    }

    pub fn robots(mut self, r: Robots) -> Self {
        self.robots = Some(r);
        self
    }

    pub fn og_title(mut self, v: &str) -> Self {
        self.og_title = Some(String::from(v));
        self
    }

    pub fn og_description(mut self, v: &str) -> Self {
        self.og_description = Some(String::from(v));
        self
    }

    pub fn og_type(mut self, v: &str) -> Self {
        self.og_type = Some(String::from(v));
        self
    }

    pub fn og_url(mut self, v: &str) -> Self {
        self.og_url = Some(String::from(v));
        self
    }

    pub fn og_image(mut self, v: &str) -> Self {
        self.og_image = Some(String::from(v));
        self
    }

    pub fn og_site_name(mut self, v: &str) -> Self {
        self.og_site_name = Some(String::from(v));
        self
    }

    pub fn og_locale(mut self, v: &str) -> Self {
        self.og_locale = Some(String::from(v));
        self
    }

    pub fn twitter_card(mut self, v: &str) -> Self {
        self.twitter_card = Some(String::from(v));
        self
    }

    pub fn twitter_title(mut self, v: &str) -> Self {
        self.twitter_title = Some(String::from(v));
        self
    }

    pub fn twitter_description(mut self, v: &str) -> Self {
        self.twitter_description = Some(String::from(v));
        self
    }

    pub fn twitter_image(mut self, v: &str) -> Self {
        self.twitter_image = Some(String::from(v));
        self
    }

    pub fn twitter_site(mut self, v: &str) -> Self {
        self.twitter_site = Some(String::from(v));
        self
    }

    pub fn favicon(mut self, v: &str) -> Self {
        self.favicon = Some(String::from(v));
        self
    }

    pub fn apple_touch_icon(mut self, v: &str) -> Self {
        self.apple_touch_icon = Some(String::from(v));
        self
    }

    pub fn author(mut self, v: &str) -> Self {
        self.author = Some(String::from(v));
        self
    }

    pub fn generator(mut self, v: &str) -> Self {
        self.generator = Some(String::from(v));
        self
    }

    pub fn custom(mut self, name: &str, content: &str) -> Self {
        self.custom_meta.push((String::from(name), String::from(content)));
        self
    }

    pub fn custom_node(mut self, node: HtmlNode) -> Self {
        self.custom_head.push(node);
        self
    }

    // ── Validation ───────────────────────────────────────────────────

    pub fn validate(&self) -> Vec<MetadataWarning> {
        let mut warnings = Vec::new();

        // Check for empty strings
        macro_rules! check_empty {
            ($field:expr, $name:expr) => {
                if let Some(ref v) = $field {
                    if v.is_empty() {
                        warnings.push(MetadataWarning::EmptyField(String::from($name)));
                    }
                }
            };
        }

        check_empty!(self.title, "title");
        check_empty!(self.description, "description");
        check_empty!(self.og_title, "og_title");
        check_empty!(self.og_description, "og_description");
        check_empty!(self.og_type, "og_type");
        check_empty!(self.og_url, "og_url");
        check_empty!(self.og_image, "og_image");
        check_empty!(self.twitter_card, "twitter_card");
        check_empty!(self.twitter_title, "twitter_title");
        check_empty!(self.twitter_description, "twitter_description");
        check_empty!(self.twitter_image, "twitter_image");

        // Validate og_type
        if let Some(ref og_type) = self.og_type {
            if !og_type.is_empty() {
                let known = [
                    "website", "article", "profile", "book",
                    "music.song", "music.album", "music.playlist", "music.radio_station",
                    "video.movie", "video.episode", "video.tv_show", "video.other",
                ];
                let mut found = false;
                for k in &known {
                    if og_type.as_str() == *k {
                        found = true;
                        break;
                    }
                }
                if !found {
                    warnings.push(MetadataWarning::UnknownOgType(og_type.clone()));
                }
            }
        }

        // Validate twitter_card
        if let Some(ref card) = self.twitter_card {
            if !card.is_empty() {
                let known = ["summary", "summary_large_image", "app", "player"];
                let mut found = false;
                for k in &known {
                    if card.as_str() == *k {
                        found = true;
                        break;
                    }
                }
                if !found {
                    warnings.push(MetadataWarning::UnknownTwitterCard(card.clone()));
                }
            }
        }

        // Partial OG: image set but no title
        if self.og_image.is_some() && self.og_title.is_none() {
            warnings.push(MetadataWarning::PartialOg(
                String::from("og:image set without og:title"),
            ));
        }

        // Partial Twitter: image set but no card
        if self.twitter_image.is_some() && self.twitter_card.is_none() {
            warnings.push(MetadataWarning::PartialTwitter(
                String::from("twitter:image set without twitter:card"),
            ));
        }

        warnings
    }

    // ── Rendering (HtmlNode elements for HtmlDocument integration) ───

    /// Render metadata as `Vec<HtmlNode>` for use with `HtmlDocument`.
    /// Does NOT include `<title>` (that's handled by HtmlDocument directly).
    pub fn render_head_nodes(&self) -> Vec<HtmlNode> {
        let mut nodes = Vec::new();

        // charset
        if let Some(ref v) = self.charset {
            nodes.push(meta().attr("charset", v.as_str()).into_node());
        }

        // viewport
        if let Some(ref v) = self.viewport {
            nodes.push(
                meta().attr("name", "viewport").attr("content", v.as_str()).into_node(),
            );
        }

        // description
        if let Some(ref v) = self.description {
            nodes.push(
                meta().attr("name", "description").attr("content", v.as_str()).into_node(),
            );
        }

        // keywords
        if let Some(ref kw) = self.keywords {
            let mut joined = String::new();
            for (i, k) in kw.iter().enumerate() {
                if i > 0 {
                    joined.push_str(", ");
                }
                joined.push_str(k.as_str());
            }
            nodes.push(
                meta().attr("name", "keywords").attr("content", joined.as_str()).into_node(),
            );
        }

        // author
        if let Some(ref v) = self.author {
            nodes.push(
                meta().attr("name", "author").attr("content", v.as_str()).into_node(),
            );
        }

        // generator
        if let Some(ref v) = self.generator {
            nodes.push(
                meta().attr("name", "generator").attr("content", v.as_str()).into_node(),
            );
        }

        // robots
        if let Some(ref r) = self.robots {
            let content = r.render_content();
            nodes.push(
                meta().attr("name", "robots").attr("content", content.as_str()).into_node(),
            );
        }

        // canonical
        if let Some(ref v) = self.canonical {
            nodes.push(
                link().attr("rel", "canonical").attr("href", v.as_str()).into_node(),
            );
        }

        // color-scheme
        if let Some(ref v) = self.color_scheme {
            nodes.push(
                meta().attr("name", "color-scheme").attr("content", v.as_str()).into_node(),
            );
        }

        // theme-color
        if let Some(ref v) = self.theme_color {
            nodes.push(
                meta().attr("name", "theme-color").attr("content", v.as_str()).into_node(),
            );
        }

        // Open Graph
        self.push_og_nodes(&mut nodes);

        // Twitter
        self.push_twitter_nodes(&mut nodes);

        // Icons
        if let Some(ref v) = self.favicon {
            nodes.push(
                link().attr("rel", "icon").attr("href", v.as_str()).into_node(),
            );
        }
        if let Some(ref v) = self.apple_touch_icon {
            nodes.push(
                link().attr("rel", "apple-touch-icon").attr("href", v.as_str()).into_node(),
            );
        }

        // Custom meta
        for (name, content) in self.custom_meta.iter() {
            nodes.push(
                meta()
                    .attr("name", name.as_str())
                    .attr("content", content.as_str())
                    .into_node(),
            );
        }

        // Custom head nodes
        for node in self.custom_head.iter() {
            // We need to clone-ish; since HtmlNode doesn't impl Clone,
            // re-render and wrap as Raw
            let rendered = render_node(node);
            nodes.push(HtmlNode::Raw(rendered));
        }

        nodes
    }

    fn push_og_nodes(&self, nodes: &mut Vec<HtmlNode>) {
        fn og(property: &str, content: &str) -> HtmlNode {
            meta().attr("property", property).attr("content", content).into_node()
        }

        if let Some(ref v) = self.og_title {
            nodes.push(og("og:title", v.as_str()));
        }
        if let Some(ref v) = self.og_description {
            nodes.push(og("og:description", v.as_str()));
        }
        if let Some(ref v) = self.og_type {
            nodes.push(og("og:type", v.as_str()));
        }
        if let Some(ref v) = self.og_url {
            nodes.push(og("og:url", v.as_str()));
        }
        if let Some(ref v) = self.og_image {
            nodes.push(og("og:image", v.as_str()));
        }
        if let Some(ref v) = self.og_site_name {
            nodes.push(og("og:site_name", v.as_str()));
        }
        if let Some(ref v) = self.og_locale {
            nodes.push(og("og:locale", v.as_str()));
        }
    }

    fn push_twitter_nodes(&self, nodes: &mut Vec<HtmlNode>) {
        fn tw(name: &str, content: &str) -> HtmlNode {
            meta().attr("name", name).attr("content", content).into_node()
        }

        if let Some(ref v) = self.twitter_card {
            nodes.push(tw("twitter:card", v.as_str()));
        }
        if let Some(ref v) = self.twitter_title {
            nodes.push(tw("twitter:title", v.as_str()));
        }
        if let Some(ref v) = self.twitter_description {
            nodes.push(tw("twitter:description", v.as_str()));
        }
        if let Some(ref v) = self.twitter_image {
            nodes.push(tw("twitter:image", v.as_str()));
        }
        if let Some(ref v) = self.twitter_site {
            nodes.push(tw("twitter:site", v.as_str()));
        }
    }

    // ── Rendering (raw HTML string for auto-injection path) ──────────

    /// Render all metadata as an HTML string for injection before `</head>`.
    pub fn render_head_tags(&self) -> String {
        let mut out = String::with_capacity(1024);

        // charset
        if let Some(ref v) = self.charset {
            out.push_str("<meta charset=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }

        // viewport
        if let Some(ref v) = self.viewport {
            out.push_str("<meta name=\"viewport\" content=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }

        // title
        if let Some(ref v) = self.title {
            out.push_str("<title>");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("</title>\n");
        }

        // description
        if let Some(ref v) = self.description {
            out.push_str("<meta name=\"description\" content=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }

        // keywords
        if let Some(ref kw) = self.keywords {
            let mut joined = String::new();
            for (i, k) in kw.iter().enumerate() {
                if i > 0 {
                    joined.push_str(", ");
                }
                joined.push_str(k.as_str());
            }
            out.push_str("<meta name=\"keywords\" content=\"");
            out.push_str(escape_html(joined.as_str()).as_str());
            out.push_str("\">\n");
        }

        // author
        if let Some(ref v) = self.author {
            out.push_str("<meta name=\"author\" content=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }

        // generator
        if let Some(ref v) = self.generator {
            out.push_str("<meta name=\"generator\" content=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }

        // robots
        if let Some(ref r) = self.robots {
            let content = r.render_content();
            out.push_str("<meta name=\"robots\" content=\"");
            out.push_str(content.as_str());
            out.push_str("\">\n");
        }

        // canonical
        if let Some(ref v) = self.canonical {
            out.push_str("<link rel=\"canonical\" href=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }

        // color-scheme
        if let Some(ref v) = self.color_scheme {
            out.push_str("<meta name=\"color-scheme\" content=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }

        // theme-color
        if let Some(ref v) = self.theme_color {
            out.push_str("<meta name=\"theme-color\" content=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }

        // Open Graph
        self.render_og_tags(&mut out);

        // Twitter
        self.render_twitter_tags(&mut out);

        // Icons
        if let Some(ref v) = self.favicon {
            out.push_str("<link rel=\"icon\" href=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }
        if let Some(ref v) = self.apple_touch_icon {
            out.push_str("<link rel=\"apple-touch-icon\" href=\"");
            out.push_str(escape_html(v.as_str()).as_str());
            out.push_str("\">\n");
        }

        // Custom meta
        for (name, content) in self.custom_meta.iter() {
            out.push_str("<meta name=\"");
            out.push_str(escape_html(name.as_str()).as_str());
            out.push_str("\" content=\"");
            out.push_str(escape_html(content.as_str()).as_str());
            out.push_str("\">\n");
        }

        // Custom head nodes
        for node in self.custom_head.iter() {
            let rendered = render_node(node);
            out.push_str(rendered.as_str());
            out.push('\n');
        }

        out
    }

    fn render_og_tags(&self, out: &mut String) {
        fn og_tag(out: &mut String, property: &str, content: &str) {
            out.push_str("<meta property=\"");
            out.push_str(property);
            out.push_str("\" content=\"");
            out.push_str(escape_html(content).as_str());
            out.push_str("\">\n");
        }

        if let Some(ref v) = self.og_title { og_tag(out, "og:title", v.as_str()); }
        if let Some(ref v) = self.og_description { og_tag(out, "og:description", v.as_str()); }
        if let Some(ref v) = self.og_type { og_tag(out, "og:type", v.as_str()); }
        if let Some(ref v) = self.og_url { og_tag(out, "og:url", v.as_str()); }
        if let Some(ref v) = self.og_image { og_tag(out, "og:image", v.as_str()); }
        if let Some(ref v) = self.og_site_name { og_tag(out, "og:site_name", v.as_str()); }
        if let Some(ref v) = self.og_locale { og_tag(out, "og:locale", v.as_str()); }
    }

    fn render_twitter_tags(&self, out: &mut String) {
        fn tw_tag(out: &mut String, name: &str, content: &str) {
            out.push_str("<meta name=\"");
            out.push_str(name);
            out.push_str("\" content=\"");
            out.push_str(escape_html(content).as_str());
            out.push_str("\">\n");
        }

        if let Some(ref v) = self.twitter_card { tw_tag(out, "twitter:card", v.as_str()); }
        if let Some(ref v) = self.twitter_title { tw_tag(out, "twitter:title", v.as_str()); }
        if let Some(ref v) = self.twitter_description { tw_tag(out, "twitter:description", v.as_str()); }
        if let Some(ref v) = self.twitter_image { tw_tag(out, "twitter:image", v.as_str()); }
        if let Some(ref v) = self.twitter_site { tw_tag(out, "twitter:site", v.as_str()); }
    }
}

/// Inject metadata tags into an HTML response body before `</head>`.
pub fn inject_metadata(body: &mut Vec<u8>, meta: &Metadata) {
    let tags = meta.render_head_tags();
    if tags.is_empty() {
        return;
    }

    // Find </head> in the body
    let needle = b"</head>";
    let body_len = body.len();
    if body_len < needle.len() {
        return;
    }

    let mut pos = None;
    for i in 0..=(body_len - needle.len()) {
        if &body.as_slice()[i..i + needle.len()] == needle {
            pos = Some(i);
            break;
        }
    }

    if let Some(idx) = pos {
        let tag_bytes = tags.as_bytes();
        let mut new_body = Vec::with_capacity(body.len() + tag_bytes.len());
        new_body.extend_from_slice(&body.as_slice()[..idx]);
        new_body.extend_from_slice(tag_bytes);
        new_body.extend_from_slice(&body.as_slice()[idx..]);
        *body = new_body;
    }
}

/// Check if response headers indicate HTML content.
pub fn is_html_content_type(content_type: &str) -> bool {
    content_type.contains("text/html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = Metadata::new();
        assert_eq!(m.charset.as_ref().unwrap().as_str(), "utf-8");
        assert_eq!(
            m.viewport.as_ref().unwrap().as_str(),
            "width=device-width, initial-scale=1"
        );
    }

    #[test]
    fn test_no_defaults() {
        let m = Metadata::new().no_defaults();
        assert!(m.charset.is_none());
        assert!(m.viewport.is_none());
    }

    #[test]
    fn test_render_basic() {
        let m = Metadata::new()
            .no_defaults()
            .title("My Page")
            .description("A cool page");

        let tags = m.render_head_tags();
        assert!(tags.contains("<title>My Page</title>"));
        assert!(tags.contains("<meta name=\"description\" content=\"A cool page\">"));
    }

    #[test]
    fn test_render_og() {
        let m = Metadata::new()
            .no_defaults()
            .og_title("OG Title")
            .og_description("OG Desc")
            .og_type("website")
            .og_url("https://example.com")
            .og_image("https://example.com/img.png");

        let tags = m.render_head_tags();
        assert!(tags.contains("<meta property=\"og:title\" content=\"OG Title\">"));
        assert!(tags.contains("<meta property=\"og:description\" content=\"OG Desc\">"));
        assert!(tags.contains("<meta property=\"og:type\" content=\"website\">"));
        assert!(tags.contains("<meta property=\"og:url\" content=\"https://example.com\">"));
        assert!(tags.contains("<meta property=\"og:image\" content=\"https://example.com/img.png\">"));
    }

    #[test]
    fn test_render_twitter() {
        let m = Metadata::new()
            .no_defaults()
            .twitter_card("summary_large_image")
            .twitter_title("Tweet Title")
            .twitter_site("@example");

        let tags = m.render_head_tags();
        assert!(tags.contains("<meta name=\"twitter:card\" content=\"summary_large_image\">"));
        assert!(tags.contains("<meta name=\"twitter:title\" content=\"Tweet Title\">"));
        assert!(tags.contains("<meta name=\"twitter:site\" content=\"@example\">"));
    }

    #[test]
    fn test_render_robots() {
        let m = Metadata::new()
            .no_defaults()
            .robots(Robots::new().no_index().no_archive());

        let tags = m.render_head_tags();
        assert!(tags.contains("<meta name=\"robots\" content=\"noindex, follow, noarchive\">"));
    }

    #[test]
    fn test_render_robots_default() {
        let m = Metadata::new()
            .no_defaults()
            .robots(Robots::new());

        let tags = m.render_head_tags();
        assert!(tags.contains("<meta name=\"robots\" content=\"index, follow\">"));
    }

    #[test]
    fn test_validate_bad_og_type() {
        let m = Metadata::new().og_type("banana");
        let warnings = m.validate();
        let mut found = false;
        for w in &warnings {
            if let MetadataWarning::UnknownOgType(v) = w {
                assert_eq!(v.as_str(), "banana");
                found = true;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_validate_good_og_type() {
        let m = Metadata::new().og_type("website");
        let warnings = m.validate();
        for w in &warnings {
            if let MetadataWarning::UnknownOgType(_) = w {
                panic!("should not warn for valid og type");
            }
        }
    }

    #[test]
    fn test_validate_partial_og() {
        let m = Metadata::new().og_image("https://example.com/img.png");
        let warnings = m.validate();
        let mut found = false;
        for w in &warnings {
            if let MetadataWarning::PartialOg(_) = w {
                found = true;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_validate_bad_twitter_card() {
        let m = Metadata::new().twitter_card("mega_card");
        let warnings = m.validate();
        let mut found = false;
        for w in &warnings {
            if let MetadataWarning::UnknownTwitterCard(v) = w {
                assert_eq!(v.as_str(), "mega_card");
                found = true;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_validate_partial_twitter() {
        let m = Metadata::new().twitter_image("https://example.com/img.png");
        let warnings = m.validate();
        let mut found = false;
        for w in &warnings {
            if let MetadataWarning::PartialTwitter(_) = w {
                found = true;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_inject_into_html() {
        let html = "<!DOCTYPE html><html><head><title>Test</title></head><body></body></html>";
        let mut body: Vec<u8> = Vec::new();
        body.extend_from_slice(html.as_bytes());

        let m = Metadata::new()
            .no_defaults()
            .description("Injected desc");

        inject_metadata(&mut body, &m);

        let result = core::str::from_utf8(body.as_slice()).unwrap();
        assert!(result.contains("<meta name=\"description\" content=\"Injected desc\">"));
        // Should appear before </head>
        let desc_pos = result.find("description").unwrap();
        let head_close_pos = result.find("</head>").unwrap();
        assert!(desc_pos < head_close_pos);
    }

    #[test]
    fn test_no_inject_for_missing_head() {
        let html = "<html><body>hello</body></html>";
        let mut body: Vec<u8> = Vec::new();
        body.extend_from_slice(html.as_bytes());
        let original_len = body.len();

        let m = Metadata::new().description("test");
        inject_metadata(&mut body, &m);

        // Body should be unchanged
        assert_eq!(body.len(), original_len);
    }

    #[test]
    fn test_render_head_nodes() {
        let m = Metadata::new()
            .no_defaults()
            .description("Node test")
            .og_title("OG Node");

        let nodes = m.render_head_nodes();
        let mut found_desc = false;
        let mut found_og = false;
        for node in &nodes {
            let rendered = render_node(node);
            if rendered.as_str().contains("description") && rendered.as_str().contains("Node test")
            {
                found_desc = true;
            }
            if rendered.as_str().contains("og:title") && rendered.as_str().contains("OG Node") {
                found_og = true;
            }
        }
        assert!(found_desc);
        assert!(found_og);
    }

    #[test]
    fn test_render_keywords() {
        let m = Metadata::new()
            .no_defaults()
            .keywords(&["rust", "web", "framework"]);

        let tags = m.render_head_tags();
        assert!(tags.contains("<meta name=\"keywords\" content=\"rust, web, framework\">"));
    }

    #[test]
    fn test_render_icons() {
        let m = Metadata::new()
            .no_defaults()
            .favicon("/favicon.ico")
            .apple_touch_icon("/apple-icon.png");

        let tags = m.render_head_tags();
        assert!(tags.contains("<link rel=\"icon\" href=\"/favicon.ico\">"));
        assert!(tags.contains("<link rel=\"apple-touch-icon\" href=\"/apple-icon.png\">"));
    }

    #[test]
    fn test_render_custom_meta() {
        let m = Metadata::new()
            .no_defaults()
            .custom("my-tag", "my-value");

        let tags = m.render_head_tags();
        assert!(tags.contains("<meta name=\"my-tag\" content=\"my-value\">"));
    }

    #[test]
    fn test_is_html_content_type() {
        assert!(is_html_content_type("text/html; charset=utf-8"));
        assert!(is_html_content_type("text/html"));
        assert!(!is_html_content_type("application/json"));
        assert!(!is_html_content_type("text/plain"));
    }

    #[test]
    fn test_validate_empty_field() {
        let m = Metadata::new().title("");
        let warnings = m.validate();
        let mut found = false;
        for w in &warnings {
            if let MetadataWarning::EmptyField(name) = w {
                if name.as_str() == "title" {
                    found = true;
                }
            }
        }
        assert!(found);
    }
}
