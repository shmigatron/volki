//! HTML element types and builder functions.
//!
//! This module provides a complete tree representation of HTML documents through
//! the [`HtmlNode`] enum and the [`HtmlElement`] struct. Every standard HTML tag
//! has a dedicated constructor function (e.g. [`div()`], [`span()`], [`a()`]),
//! and elements are assembled using a chainable builder API that reads naturally
//! from top to bottom.
//!
//! # Building elements
//!
//! Each constructor returns an [`HtmlElement`] that you can chain methods on.
//! Call [`.into_node()`](HtmlElement::into_node) at the end to wrap it in an
//! [`HtmlNode`] for embedding inside another element's children.
//!
//! ```
//! # use volki::libs::web::html::element::*;
//! # use volki::core::volkiwithstds::collections::{String, Vec};
//!
//! let page = div().class("page")
//!     .child(
//!         header().class("top-bar")
//!             .child(h1().text("My Site").into_node())
//!             .into_node(),
//!     )
//!     .child(
//!         main_el().class("content")
//!             .child(p().text("Welcome to the site.").into_node())
//!             .into_node(),
//!     )
//!     .into_node();
//! ```
//!
//! # Attributes
//!
//! Use [`.attr(name, value)`](HtmlElement::attr) for arbitrary attributes, or
//! the shorthand helpers [`.class()`](HtmlElement::class) and
//! [`.id()`](HtmlElement::id):
//!
//! ```
//! # use volki::libs::web::html::element::*;
//! # use volki::core::volkiwithstds::collections::{String, Vec};
//! let link = a()
//!     .attr("href", "https://example.com")
//!     .attr("target", "_blank")
//!     .class("external-link")
//!     .text("Visit Example")
//!     .into_node();
//! ```
//!
//! # Text vs. raw content
//!
//! - [`.text()`](HtmlElement::text) and [`text()`] create nodes whose content
//!   is HTML-escaped during rendering (`<` becomes `&lt;`, etc.).
//! - [`.raw()`](HtmlElement::raw) and [`raw_html()`] insert content verbatim
//!   with no escaping — use this only for trusted HTML fragments.
//!
//! # Void elements
//!
//! Void (self-closing) elements like `<br>`, `<img>`, and `<input>` are created
//! with their own constructors and have `self_closing` set to `true`
//! automatically. They render without a closing tag.
//!
//! ```
//! # use volki::libs::web::html::element::*;
//! # use volki::core::volkiwithstds::collections::{String, Vec};
//! let avatar = img()
//!     .attr("src", "/avatar.png")
//!     .attr("alt", "User avatar")
//!     .class("avatar")
//!     .into_node();
//! ```
//!
//! # RSX integration
//!
//! The [`IntoChildren`] trait allows the RSX compiler to accept `{expr}` blocks
//! that return an [`HtmlNode`], a [`Vec<HtmlNode>`], or an [`HtmlElement`]
//! interchangeably. You generally don't need to call this trait directly — the
//! compiled RSX output calls it for you.

use crate::core::volkiwithstds::collections::{String, Vec};

/// A single node in the HTML tree.
///
/// An HTML document is represented as a tree of `HtmlNode` values. Each node
/// is one of three kinds:
///
/// | Variant              | Description                                       |
/// |----------------------|---------------------------------------------------|
/// | [`Element`](Self::Element) | A full HTML element with tag, attributes, and children |
/// | [`Text`](Self::Text)       | Plain text content (escaped on render)            |
/// | [`Raw`](Self::Raw)         | Pre-formed HTML inserted verbatim (no escaping)   |
///
/// # Examples
///
/// Creating nodes directly:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// // Text node — content will be escaped
/// let greeting = HtmlNode::Text(String::from("Hello <world>"));
///
/// // Raw node — content is NOT escaped
/// let icon = HtmlNode::Raw(String::from("<svg>...</svg>"));
///
/// // Element node — use a constructor + into_node()
/// let card = div().class("card").text("Content").into_node();
/// ```
pub enum HtmlNode {
    /// A structured HTML element with a tag, attributes, and child nodes.
    ///
    /// This is the most common variant. It is typically created by calling a
    /// tag constructor (e.g. [`div()`]) followed by [`.into_node()`](HtmlElement::into_node):
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let node = div().class("wrapper").into_node();
    /// // Produces: <div class="wrapper"></div>
    /// ```
    Element(HtmlElement),

    /// A text node whose content will be HTML-escaped during rendering.
    ///
    /// Characters like `<`, `>`, `&`, and `"` are converted to their HTML
    /// entity equivalents so that user-supplied strings are always safe to
    /// render. Use this for any content that comes from user input or
    /// untrusted sources.
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let safe = HtmlNode::Text(String::from("2 < 3"));
    /// // Renders as: 2 &lt; 3
    /// ```
    Text(String),

    /// Raw HTML that will be inserted into the output without any escaping.
    ///
    /// Use this when you have a pre-built HTML fragment (e.g. an SVG icon
    /// or a snippet from a markdown-to-HTML converter) that you trust and
    /// want to embed directly. **Never** use this with untrusted input — it
    /// can introduce XSS vulnerabilities.
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let svg = HtmlNode::Raw(String::from("<svg viewBox=\"0 0 24 24\">...</svg>"));
    /// ```
    Raw(String),
}

/// A structured HTML element with a tag name, attributes, and child nodes.
///
/// `HtmlElement` is the main building block of the HTML tree. It uses a
/// chainable builder pattern: every setter method consumes `self` and returns
/// the modified element, so you can write a full element definition as a
/// single expression.
///
/// # Builder methods
///
/// | Method                              | Purpose                                      |
/// |-------------------------------------|----------------------------------------------|
/// | [`.attr(name, value)`](Self::attr)  | Add any HTML attribute                       |
/// | [`.class(value)`](Self::class)      | Shorthand for `.attr("class", value)`        |
/// | [`.id(value)`](Self::id)            | Shorthand for `.attr("id", value)`           |
/// | [`.child(node)`](Self::child)       | Append a single child [`HtmlNode`]           |
/// | [`.children(nodes)`](Self::children)| Append a vector of child nodes               |
/// | [`.text(t)`](Self::text)            | Append a text child (escaped)                |
/// | [`.raw(html)`](Self::raw)           | Append a raw HTML child (not escaped)        |
/// | [`.into_node()`](Self::into_node)   | Wrap into [`HtmlNode::Element`] for nesting  |
///
/// # Examples
///
/// A navigation bar with links:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let navbar = nav().class("navbar")
///     .child(
///         a().attr("href", "/").text("Home").into_node()
///     )
///     .child(
///         a().attr("href", "/about").text("About").into_node()
///     )
///     .child(
///         a().attr("href", "/contact").text("Contact").into_node()
///     )
///     .into_node();
/// ```
///
/// A form with labeled inputs:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let login_form = form().attr("method", "POST").attr("action", "/login")
///     .child(label().attr("for", "email").text("Email").into_node())
///     .child(input().attr("type", "email").attr("id", "email").into_node())
///     .child(label().attr("for", "pass").text("Password").into_node())
///     .child(input().attr("type", "password").attr("id", "pass").into_node())
///     .child(button().attr("type", "submit").text("Log in").into_node())
///     .into_node();
/// ```
pub struct HtmlElement {
    /// The HTML tag name (e.g. `"div"`, `"span"`, `"input"`).
    ///
    /// This is a `&'static str` because tag names come from the fixed set of
    /// constructor functions and are always string literals.
    pub tag: &'static str,

    /// Key-value attribute pairs applied to this element.
    ///
    /// Each entry is a `(name, value)` tuple. Attributes are rendered in the
    /// order they were added. Duplicate attribute names are allowed but the
    /// browser will use the first occurrence.
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let el = div().attr("data-x", "1").attr("data-y", "2");
    /// assert_eq!(el.attrs.len(), 2);
    /// ```
    pub attrs: Vec<(String, String)>,

    /// Ordered child nodes nested inside this element.
    ///
    /// Children are rendered in order. For void elements (like `<br>` or
    /// `<img>`), this vector is always empty and any children would be
    /// ignored during rendering.
    pub children: Vec<HtmlNode>,

    /// Whether this element is self-closing (void).
    ///
    /// When `true`, the element renders as `<tag ... />` with no closing tag
    /// and no children. This is set automatically by the void element
    /// constructors ([`br()`], [`hr()`], [`img()`], [`input()`], [`meta()`],
    /// [`link()`]).
    pub self_closing: bool,
}

impl HtmlElement {
    /// Creates a new element with the given tag name.
    ///
    /// The element starts with no attributes, no children, and `self_closing`
    /// set to `false`. Use the builder methods to configure it.
    ///
    /// For standard tags, prefer the convenience constructors ([`div()`],
    /// [`span()`], etc.) which call this internally. Use `new` directly only
    /// when you need a tag that doesn't have a dedicated constructor.
    ///
    /// # Arguments
    ///
    /// * `tag` - The HTML tag name as a static string literal.
    ///
    /// # Examples
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// // Using a tag that has no convenience constructor
    /// let details = HtmlElement::new("details")
    ///     .child(HtmlElement::new("summary").text("Click to expand").into_node())
    ///     .text("Hidden content here.")
    ///     .into_node();
    /// ```
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            attrs: Vec::new(),
            children: Vec::new(),
            self_closing: false,
        }
    }

    /// Creates a new void (self-closing) element with the given tag name.
    ///
    /// Void elements cannot have children and render without a closing tag
    /// (e.g. `<br />`, `<img src="..." />`). This constructor sets
    /// `self_closing` to `true` automatically.
    ///
    /// This is used internally by the void element constructors ([`br()`],
    /// [`img()`], etc.) and is not exposed publicly.
    ///
    /// # Arguments
    ///
    /// * `tag` - The void element's tag name as a static string literal.
    fn new_void(tag: &'static str) -> Self {
        Self {
            tag,
            attrs: Vec::new(),
            children: Vec::new(),
            self_closing: true,
        }
    }

    /// Adds an arbitrary HTML attribute to this element.
    ///
    /// Attributes are stored in insertion order and rendered as
    /// `name="value"` in the output. This method consumes and returns `self`
    /// for chaining.
    ///
    /// For the two most common attributes, prefer the dedicated shorthands
    /// [`.class()`](Self::class) and [`.id()`](Self::id).
    ///
    /// # Arguments
    ///
    /// * `name`  - The attribute name (e.g. `"href"`, `"data-id"`, `"aria-label"`).
    /// * `value` - The attribute value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let link = a()
    ///     .attr("href", "https://example.com")
    ///     .attr("target", "_blank")
    ///     .attr("rel", "noopener noreferrer")
    ///     .text("Example")
    ///     .into_node();
    /// // Produces: <a href="https://example.com" target="_blank" rel="noopener noreferrer">Example</a>
    /// ```
    ///
    /// Data attributes and ARIA:
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let toggle = button()
    ///     .attr("data-action", "toggle-menu")
    ///     .attr("aria-expanded", "false")
    ///     .attr("aria-controls", "main-nav")
    ///     .text("Menu")
    ///     .into_node();
    /// ```
    pub fn attr(mut self, name: &str, value: &str) -> Self {
        self.attrs.push((String::from(name), String::from(value)));
        self
    }

    /// Sets the `class` attribute on this element.
    ///
    /// This is a shorthand for `.attr("class", value)`. Pass a
    /// space-separated string for multiple CSS classes.
    ///
    /// # Arguments
    ///
    /// * `value` - One or more CSS class names, space-separated.
    ///
    /// # Examples
    ///
    /// Single class:
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let card = div().class("card").text("Hello").into_node();
    /// // Produces: <div class="card">Hello</div>
    /// ```
    ///
    /// Multiple classes:
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let alert = div().class("alert alert-danger").text("Error!").into_node();
    /// // Produces: <div class="alert alert-danger">Error!</div>
    /// ```
    pub fn class(self, value: &str) -> Self {
        self.attr("class", value)
    }

    /// Sets the `id` attribute on this element.
    ///
    /// This is a shorthand for `.attr("id", value)`. IDs should be unique
    /// within a document.
    ///
    /// # Arguments
    ///
    /// * `value` - The element's unique identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let hero = section().id("hero").class("full-width")
    ///     .child(h1().text("Welcome").into_node())
    ///     .into_node();
    /// // Produces: <section id="hero" class="full-width"><h1>Welcome</h1></section>
    /// ```
    pub fn id(self, value: &str) -> Self {
        self.attr("id", value)
    }

    /// Appends a single child node to this element.
    ///
    /// The child is added after any previously added children. Use
    /// [`.into_node()`](Self::into_node) on child elements to convert them
    /// to [`HtmlNode`] before passing them here.
    ///
    /// # Arguments
    ///
    /// * `node` - The child [`HtmlNode`] to append.
    ///
    /// # Examples
    ///
    /// Nesting elements:
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let list = ul()
    ///     .child(li().text("First").into_node())
    ///     .child(li().text("Second").into_node())
    ///     .child(li().text("Third").into_node())
    ///     .into_node();
    /// // Produces: <ul><li>First</li><li>Second</li><li>Third</li></ul>
    /// ```
    ///
    /// Mixing element and text children:
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let paragraph = p()
    ///     .text("Click ")
    ///     .child(a().attr("href", "/here").text("here").into_node())
    ///     .text(" to continue.")
    ///     .into_node();
    /// // Produces: <p>Click <a href="/here">here</a> to continue.</p>
    /// ```
    pub fn child(mut self, node: HtmlNode) -> Self {
        self.children.push(node);
        self
    }

    /// Appends a text child node to this element.
    ///
    /// The text content will be HTML-escaped during rendering, making it safe
    /// to pass user-supplied strings. Characters like `<`, `>`, `&`, and `"`
    /// are converted to their entity equivalents.
    ///
    /// # Arguments
    ///
    /// * `t` - The text content to append.
    ///
    /// # Examples
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let greeting = p().text("Hello, world!").into_node();
    /// // Produces: <p>Hello, world!</p>
    /// ```
    ///
    /// Safe with special characters:
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let safe = p().text("Use <div> for containers & <span> for inline.").into_node();
    /// // Produces: <p>Use &lt;div&gt; for containers &amp; &lt;span&gt; for inline.</p>
    /// ```
    pub fn text(mut self, t: &str) -> Self {
        self.children.push(HtmlNode::Text(String::from(t)));
        self
    }

    /// Appends a raw HTML child node to this element.
    ///
    /// The content is inserted **without any escaping**, so it must be trusted
    /// HTML. Use this for embedding pre-built HTML fragments like SVG icons,
    /// rendered markdown, or content from other template engines.
    ///
    /// # Safety
    ///
    /// Never pass untrusted or user-supplied strings to this method. Doing so
    /// can introduce XSS vulnerabilities. Use [`.text()`](Self::text) instead
    /// for any content that might contain special characters.
    ///
    /// # Arguments
    ///
    /// * `html` - A trusted HTML fragment to insert verbatim.
    ///
    /// # Examples
    ///
    /// Embedding an SVG icon:
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let icon_btn = button().class("icon-btn")
    ///     .raw("<svg viewBox=\"0 0 24 24\"><path d=\"M3 12h18\"/></svg>")
    ///     .text(" Settings")
    ///     .into_node();
    /// ```
    pub fn raw(mut self, html: &str) -> Self {
        self.children.push(HtmlNode::Raw(String::from(html)));
        self
    }

    /// Appends multiple child nodes at once.
    ///
    /// This is a convenience method for adding a pre-built vector of children
    /// in a single call. Nodes are appended in order after any existing
    /// children.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A vector of [`HtmlNode`] values to append.
    ///
    /// # Examples
    ///
    /// Building a list from a dynamic collection:
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let items = vec!["Apples", "Bananas", "Cherries"];
    /// let mut nodes = Vec::new();
    /// for item in items {
    ///     nodes.push(li().text(item).into_node());
    /// }
    /// let list = ul().class("fruit-list").children(nodes).into_node();
    /// // Produces: <ul class="fruit-list"><li>Apples</li><li>Bananas</li><li>Cherries</li></ul>
    /// ```
    pub fn children(mut self, nodes: Vec<HtmlNode>) -> Self {
        for node in nodes {
            self.children.push(node);
        }
        self
    }

    /// Converts this element into an [`HtmlNode::Element`].
    ///
    /// This is the bridge between the builder API and the tree structure.
    /// Call this when you need to pass an element as a child to another
    /// element's [`.child()`](Self::child) method, since `.child()` expects
    /// an [`HtmlNode`], not an [`HtmlElement`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use volki::libs::web::html::element::*;
    /// # use volki::core::volkiwithstds::collections::{String, Vec};
    /// let card = div().class("card")
    ///     .child(
    ///         // .into_node() converts HtmlElement -> HtmlNode
    ///         h2().text("Title").into_node()
    ///     )
    ///     .child(
    ///         p().text("Description").into_node()
    ///     )
    ///     .into_node();
    /// ```
    pub fn into_node(self) -> HtmlNode {
        HtmlNode::Element(self)
    }
}

// ── Convenience element constructors ────────────────────────────────────────
//
// Each function below creates an empty `HtmlElement` for the corresponding
// HTML tag. Use the builder methods (`.attr()`, `.class()`, `.child()`, etc.)
// to configure the element, then call `.into_node()` to embed it in a tree.

/// Creates a `<div>` element — a generic block-level flow container.
///
/// The `<div>` element is the most general-purpose container in HTML. It has
/// no inherent semantics and is used purely for grouping content and applying
/// styles or layout.
///
/// # Examples
///
/// Simple wrapper:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let wrapper = div().class("wrapper")
///     .child(p().text("Inside the wrapper.").into_node())
///     .into_node();
/// // Produces: <div class="wrapper"><p>Inside the wrapper.</p></div>
/// ```
///
/// Grid layout:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let grid = div().class("grid grid-cols-3 gap-4")
///     .child(div().class("col").text("Column 1").into_node())
///     .child(div().class("col").text("Column 2").into_node())
///     .child(div().class("col").text("Column 3").into_node())
///     .into_node();
/// ```
pub fn div() -> HtmlElement { HtmlElement::new("div") }

/// Creates a `<span>` element — a generic inline container.
///
/// Unlike [`div()`], `<span>` is an inline element. It does not start a new
/// line and only takes up as much width as its content. Use it to style or
/// target a portion of text within a block element.
///
/// # Examples
///
/// Highlighting a word:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let msg = p()
///     .text("Status: ")
///     .child(span().class("text-green").text("Online").into_node())
///     .into_node();
/// // Produces: <p>Status: <span class="text-green">Online</span></p>
/// ```
pub fn span() -> HtmlElement { HtmlElement::new("span") }

/// Creates a `<p>` element — a paragraph of text.
///
/// Browsers add vertical margin around paragraphs by default. Use this for
/// any block of running text.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let intro = p().class("intro")
///     .text("Welcome to the documentation. This guide covers all available elements.")
///     .into_node();
/// ```
pub fn p() -> HtmlElement { HtmlElement::new("p") }

/// Creates an `<h1>` element — the top-level heading.
///
/// There should typically be only one `<h1>` per page, representing the main
/// title. Screen readers and search engines use heading hierarchy to
/// understand document structure.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let title = h1().text("Volki Documentation").into_node();
/// ```
pub fn h1() -> HtmlElement { HtmlElement::new("h1") }

/// Creates an `<h2>` element — a second-level section heading.
///
/// Use `<h2>` for major sections beneath the page title.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let section_title = h2().id("getting-started").text("Getting Started").into_node();
/// ```
pub fn h2() -> HtmlElement { HtmlElement::new("h2") }

/// Creates an `<h3>` element — a third-level subsection heading.
///
/// Use `<h3>` for subsections within an `<h2>` section.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let subsection = h3().text("Installation").into_node();
/// ```
pub fn h3() -> HtmlElement { HtmlElement::new("h3") }

/// Creates an `<h4>` element — a fourth-level heading.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let heading = h4().text("Platform Requirements").into_node();
/// ```
pub fn h4() -> HtmlElement { HtmlElement::new("h4") }

/// Creates an `<h5>` element — a fifth-level heading.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let heading = h5().text("macOS Notes").into_node();
/// ```
pub fn h5() -> HtmlElement { HtmlElement::new("h5") }

/// Creates an `<h6>` element — a sixth-level heading (the smallest).
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let heading = h6().text("Footnote").into_node();
/// ```
pub fn h6() -> HtmlElement { HtmlElement::new("h6") }

/// Creates an `<a>` element — a hyperlink.
///
/// The `<a>` element creates a clickable link to another page, a file, an
/// email address, a location on the same page, or anything else a URL can
/// address. Always set the `href` attribute.
///
/// # Examples
///
/// Basic link:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let link = a().attr("href", "/about").text("About Us").into_node();
/// // Produces: <a href="/about">About Us</a>
/// ```
///
/// External link that opens in a new tab:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let ext = a()
///     .attr("href", "https://github.com")
///     .attr("target", "_blank")
///     .attr("rel", "noopener noreferrer")
///     .text("GitHub")
///     .into_node();
/// ```
///
/// Link wrapping another element:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let card_link = a().attr("href", "/post/1")
///     .child(div().class("card").text("Read more...").into_node())
///     .into_node();
/// ```
pub fn a() -> HtmlElement { HtmlElement::new("a") }

/// Creates a `<nav>` element — a navigation section.
///
/// The `<nav>` element represents a section of a page whose purpose is to
/// provide navigation links, either within the current document or to other
/// documents. Screen readers use this to identify site navigation.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let nav_bar = nav().class("main-nav")
///     .child(a().attr("href", "/").text("Home").into_node())
///     .child(a().attr("href", "/docs").text("Docs").into_node())
///     .child(a().attr("href", "/blog").text("Blog").into_node())
///     .into_node();
/// ```
pub fn nav() -> HtmlElement { HtmlElement::new("nav") }

/// Creates a `<header>` element — introductory content or navigational aids.
///
/// Typically contains the site logo, navigation, and/or a search form. A
/// page can have multiple `<header>` elements (e.g. one for the page and one
/// inside an `<article>`).
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let site_header = header().class("site-header")
///     .child(h1().text("My App").into_node())
///     .child(nav().child(a().attr("href", "/").text("Home").into_node()).into_node())
///     .into_node();
/// ```
pub fn header() -> HtmlElement { HtmlElement::new("header") }

/// Creates a `<footer>` element — footer for its nearest sectioning content.
///
/// Typically contains information about the author, copyright data, or
/// related links.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let site_footer = footer().class("site-footer")
///     .child(p().text("Copyright 2026 Volki.").into_node())
///     .into_node();
/// ```
pub fn footer() -> HtmlElement { HtmlElement::new("footer") }

/// Creates a `<main>` element — the dominant content of the document body.
///
/// There should be only one `<main>` per page (not nested inside `<article>`,
/// `<aside>`, `<footer>`, `<header>`, or `<nav>`). It represents the central
/// content that is unique to this page.
///
/// This function is named `main_el` instead of `main` to avoid conflicting
/// with Rust's `main` keyword.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let content = main_el().class("container")
///     .child(h1().text("Dashboard").into_node())
///     .child(p().text("Welcome back.").into_node())
///     .into_node();
/// ```
pub fn main_el() -> HtmlElement { HtmlElement::new("main") }

/// Creates a `<section>` element — a standalone section of a document.
///
/// Use `<section>` to group thematically related content, typically with a
/// heading. It provides more semantic meaning than a plain `<div>`.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let features = section().id("features")
///     .child(h2().text("Features").into_node())
///     .child(ul()
///         .child(li().text("Fast").into_node())
///         .child(li().text("Reliable").into_node())
///         .into_node())
///     .into_node();
/// ```
pub fn section() -> HtmlElement { HtmlElement::new("section") }

/// Creates an `<article>` element — a self-contained composition.
///
/// An `<article>` represents a self-contained piece of content that could be
/// independently distributed or reused — blog posts, news stories, forum
/// posts, product cards, etc.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let post = article().class("blog-post")
///     .child(h2().text("How to Use Volki").into_node())
///     .child(p().text("This tutorial walks you through...").into_node())
///     .into_node();
/// ```
pub fn article() -> HtmlElement { HtmlElement::new("article") }

/// Creates a `<ul>` element — an unordered (bulleted) list.
///
/// Contains [`<li>`](li) items. The browser renders bullet points by default.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let groceries = ul().class("groceries")
///     .child(li().text("Milk").into_node())
///     .child(li().text("Eggs").into_node())
///     .child(li().text("Bread").into_node())
///     .into_node();
/// // Produces:
/// // <ul class="groceries">
/// //   <li>Milk</li>
/// //   <li>Eggs</li>
/// //   <li>Bread</li>
/// // </ul>
/// ```
pub fn ul() -> HtmlElement { HtmlElement::new("ul") }

/// Creates an `<ol>` element — an ordered (numbered) list.
///
/// Contains [`<li>`](li) items. The browser renders sequential numbers by
/// default.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let steps = ol()
///     .child(li().text("Install Volki.").into_node())
///     .child(li().text("Run `volki init`.").into_node())
///     .child(li().text("Start building.").into_node())
///     .into_node();
/// ```
pub fn ol() -> HtmlElement { HtmlElement::new("ol") }

/// Creates an `<li>` element — a list item.
///
/// Must be a child of [`<ul>`](ul), [`<ol>`](ol), or `<menu>`.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let item = li().class("active").text("Dashboard").into_node();
/// ```
pub fn li() -> HtmlElement { HtmlElement::new("li") }

/// Creates a `<table>` element — tabular data.
///
/// Use tables for data that is logically arranged in rows and columns (not
/// for page layout). Typically contains [`<thead>`](thead),
/// [`<tbody>`](tbody), [`<tr>`](tr), [`<th>`](th), and [`<td>`](td).
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let data_table = table().class("data-table")
///     .child(thead()
///         .child(tr()
///             .child(th().text("Name").into_node())
///             .child(th().text("Version").into_node())
///             .into_node())
///         .into_node())
///     .child(tbody()
///         .child(tr()
///             .child(td().text("volki").into_node())
///             .child(td().text("1.0.0").into_node())
///             .into_node())
///         .into_node())
///     .into_node();
/// ```
pub fn table() -> HtmlElement { HtmlElement::new("table") }

/// Creates a `<thead>` element — the header row group of a table.
///
/// Contains one or more [`<tr>`](tr) rows that define column headers.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let head = thead()
///     .child(tr()
///         .child(th().text("Package").into_node())
///         .child(th().text("License").into_node())
///         .into_node())
///     .into_node();
/// ```
pub fn thead() -> HtmlElement { HtmlElement::new("thead") }

/// Creates a `<tbody>` element — the body row group of a table.
///
/// Contains the data rows of a table, as opposed to the header
/// ([`<thead>`](thead)) or footer.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let body = tbody()
///     .child(tr()
///         .child(td().text("serde").into_node())
///         .child(td().text("MIT").into_node())
///         .into_node())
///     .into_node();
/// ```
pub fn tbody() -> HtmlElement { HtmlElement::new("tbody") }

/// Creates a `<tr>` element — a table row.
///
/// Contains [`<th>`](th) (header) or [`<td>`](td) (data) cells.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let row = tr()
///     .child(td().text("tokio").into_node())
///     .child(td().text("MIT").into_node())
///     .into_node();
/// ```
pub fn tr() -> HtmlElement { HtmlElement::new("tr") }

/// Creates a `<th>` element — a table header cell.
///
/// Renders as bold and centered by default. Used inside [`<tr>`](tr) within
/// a [`<thead>`](thead).
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let header_cell = th().attr("scope", "col").text("Status").into_node();
/// ```
pub fn th() -> HtmlElement { HtmlElement::new("th") }

/// Creates a `<td>` element — a table data cell.
///
/// Used inside [`<tr>`](tr) within a [`<tbody>`](tbody) to hold data values.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let cell = td().class("numeric").text("42").into_node();
/// ```
pub fn td() -> HtmlElement { HtmlElement::new("td") }

/// Creates a `<form>` element — an interactive form for user input.
///
/// Forms collect user input and submit it to a server. Set the `method`
/// (GET/POST) and `action` (target URL) attributes.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let search_form = form().attr("method", "GET").attr("action", "/search")
///     .child(input().attr("type", "text").attr("name", "q").attr("placeholder", "Search...").into_node())
///     .child(button().attr("type", "submit").text("Go").into_node())
///     .into_node();
/// ```
pub fn form() -> HtmlElement { HtmlElement::new("form") }

/// Creates a `<button>` element — a clickable button.
///
/// Buttons can submit forms (`type="submit"`), reset them (`type="reset"`),
/// or act as generic interactive controls (`type="button"`).
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let save_btn = button()
///     .attr("type", "button")
///     .class("btn btn-primary")
///     .text("Save Changes")
///     .into_node();
/// ```
pub fn button() -> HtmlElement { HtmlElement::new("button") }

/// Creates a `<label>` element — a caption for a form control.
///
/// Use the `for` attribute to associate the label with an input element's
/// `id`. Clicking the label will focus the associated input.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let email_label = label()
///     .attr("for", "email-input")
///     .text("Email Address")
///     .into_node();
/// ```
pub fn label() -> HtmlElement { HtmlElement::new("label") }

/// Creates a `<textarea>` element — a multi-line text input field.
///
/// Unlike [`<input>`](input), a `<textarea>` can hold multiple lines of
/// text. Set `rows` and `cols` attributes to control default size.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let comment_box = textarea()
///     .attr("name", "comment")
///     .attr("rows", "4")
///     .attr("placeholder", "Write a comment...")
///     .into_node();
/// ```
pub fn textarea() -> HtmlElement { HtmlElement::new("textarea") }

/// Creates a `<select>` element — a drop-down selection list.
///
/// Contains [`<option>`](option) elements representing the available choices.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let picker = select().attr("name", "color")
///     .child(option().attr("value", "red").text("Red").into_node())
///     .child(option().attr("value", "blue").text("Blue").into_node())
///     .child(option().attr("value", "green").text("Green").into_node())
///     .into_node();
/// ```
pub fn select() -> HtmlElement { HtmlElement::new("select") }

/// Creates an `<option>` element — an item in a [`<select>`](select) list.
///
/// Set the `value` attribute to define the value submitted with the form.
/// Add the `selected` attribute to make it the default selection.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let opt = option()
///     .attr("value", "dark")
///     .attr("selected", "selected")
///     .text("Dark Mode")
///     .into_node();
/// ```
pub fn option() -> HtmlElement { HtmlElement::new("option") }

/// Creates a `<pre>` element — preformatted text.
///
/// Content inside `<pre>` is rendered in a monospace font and whitespace
/// (spaces, tabs, newlines) is preserved exactly as written. Often used
/// together with [`<code>`](code) for code blocks.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let code_block = pre()
///     .child(code().text("fn main() {\n    println!(\"Hello\");\n}").into_node())
///     .into_node();
/// ```
pub fn pre() -> HtmlElement { HtmlElement::new("pre") }

/// Creates a `<code>` element — an inline code fragment.
///
/// Used for short inline code references within prose. For multi-line code
/// blocks, wrap it inside [`<pre>`](pre).
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let tip = p()
///     .text("Run ")
///     .child(code().text("volki license --path .").into_node())
///     .text(" to scan your project.")
///     .into_node();
/// // Produces: <p>Run <code>volki license --path .</code> to scan your project.</p>
/// ```
pub fn code() -> HtmlElement { HtmlElement::new("code") }

/// Creates a `<strong>` element — strong importance (typically bold).
///
/// Indicates that its content has strong importance, seriousness, or urgency.
/// Screen readers may use a different tone of voice for `<strong>` content.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let warning = p()
///     .child(strong().text("Warning:").into_node())
///     .text(" This action cannot be undone.")
///     .into_node();
/// ```
pub fn strong() -> HtmlElement { HtmlElement::new("strong") }

/// Creates an `<em>` element — stress emphasis (typically italic).
///
/// Indicates emphasis that subtly changes the meaning of a sentence, similar
/// to how you might stress a word when speaking aloud.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let note = p()
///     .text("You ")
///     .child(em().text("must").into_node())
///     .text(" accept the terms to continue.")
///     .into_node();
/// ```
pub fn em() -> HtmlElement { HtmlElement::new("em") }

/// Creates a `<blockquote>` element — a block-level quotation.
///
/// Used to quote a block of text from another source. Browsers typically
/// indent blockquotes. Use the `cite` attribute to reference the source URL.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let quote = blockquote().attr("cite", "https://example.com/article")
///     .child(p().text("The only way to do great work is to love what you do.").into_node())
///     .into_node();
/// ```
pub fn blockquote() -> HtmlElement { HtmlElement::new("blockquote") }

/// Creates a `<script>` element — embedded or linked JavaScript.
///
/// Use the `src` attribute to link an external script file, or add inline
/// script content with [`.text()`](HtmlElement::text). Add `defer` or
/// `async` attributes to control loading behavior.
///
/// # Examples
///
/// External script:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let js = script().attr("src", "/app.js").attr("defer", "").into_node();
/// // Produces: <script src="/app.js" defer=""></script>
/// ```
///
/// Inline script:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let inline = script().text("console.log('loaded');").into_node();
/// ```
pub fn script() -> HtmlElement { HtmlElement::new("script") }

/// Creates a `<style>` element — an embedded CSS stylesheet.
///
/// Inline styles defined here apply to the entire document (or shadow root).
/// For external stylesheets, use [`<link>`](link) instead.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let css = style().text("body { margin: 0; font-family: sans-serif; }").into_node();
/// ```
pub fn style() -> HtmlElement { HtmlElement::new("style") }

// ── Void elements ─────────────────────────────────────────────────────────
//
// Void elements are self-closing — they cannot have children and render
// without a closing tag (e.g. `<br />`, `<img src="..." />`).

/// Creates a `<br>` void element — a line break.
///
/// Forces a line break within inline content. Do not use `<br>` for spacing
/// between block elements — use CSS margins instead.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let address = p()
///     .text("123 Main Street")
///     .child(br().into_node())
///     .text("Springfield, IL 62704")
///     .into_node();
/// // Produces: <p>123 Main Street<br />Springfield, IL 62704</p>
/// ```
pub fn br() -> HtmlElement { HtmlElement::new_void("br") }

/// Creates an `<hr>` void element — a thematic break (horizontal rule).
///
/// Represents a thematic break between paragraph-level content, such as a
/// scene change in a story or a shift of topic within a section. Renders as
/// a horizontal line by default.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let content = div()
///     .child(p().text("Chapter 1 content...").into_node())
///     .child(hr().class("divider").into_node())
///     .child(p().text("Chapter 2 content...").into_node())
///     .into_node();
/// ```
pub fn hr() -> HtmlElement { HtmlElement::new_void("hr") }

/// Creates an `<img>` void element — an image embed.
///
/// Always set the `src` attribute (image URL) and the `alt` attribute
/// (alternative text for accessibility). The `alt` text is shown when the
/// image fails to load and is read by screen readers.
///
/// # Examples
///
/// Basic image:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let photo = img()
///     .attr("src", "/images/hero.jpg")
///     .attr("alt", "A scenic mountain landscape")
///     .class("hero-image")
///     .into_node();
/// // Produces: <img src="/images/hero.jpg" alt="A scenic mountain landscape" class="hero-image" />
/// ```
///
/// Responsive image with dimensions:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let avatar = img()
///     .attr("src", "/avatar.png")
///     .attr("alt", "User profile picture")
///     .attr("width", "48")
///     .attr("height", "48")
///     .attr("loading", "lazy")
///     .into_node();
/// ```
pub fn img() -> HtmlElement { HtmlElement::new_void("img") }

/// Creates an `<input>` void element — a form input control.
///
/// The `type` attribute determines the kind of input (`text`, `email`,
/// `password`, `checkbox`, `radio`, `number`, `file`, etc.). Always set a
/// `name` attribute so the value is submitted with the form.
///
/// # Examples
///
/// Text input:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let name_input = input()
///     .attr("type", "text")
///     .attr("name", "username")
///     .attr("placeholder", "Enter your name")
///     .attr("required", "")
///     .into_node();
/// ```
///
/// Checkbox:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let checkbox = input()
///     .attr("type", "checkbox")
///     .attr("name", "agree")
///     .attr("id", "agree-checkbox")
///     .into_node();
/// ```
///
/// Hidden field:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let csrf = input()
///     .attr("type", "hidden")
///     .attr("name", "_token")
///     .attr("value", "abc123")
///     .into_node();
/// ```
pub fn input() -> HtmlElement { HtmlElement::new_void("input") }

/// Creates a `<meta>` void element — document-level metadata.
///
/// Used in the `<head>` to specify character set, viewport configuration,
/// page description, and other metadata that doesn't appear as visible
/// content.
///
/// # Examples
///
/// Character encoding:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let charset = meta().attr("charset", "UTF-8").into_node();
/// ```
///
/// Viewport for responsive design:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let viewport = meta()
///     .attr("name", "viewport")
///     .attr("content", "width=device-width, initial-scale=1.0")
///     .into_node();
/// ```
///
/// Page description for SEO:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let desc = meta()
///     .attr("name", "description")
///     .attr("content", "Volki scans your project dependencies for license information.")
///     .into_node();
/// ```
pub fn meta() -> HtmlElement { HtmlElement::new_void("meta") }

/// Creates a `<link>` void element — an external resource link.
///
/// Most commonly used to link CSS stylesheets, favicons, and preloaded
/// resources. Placed in the `<head>` section of the document.
///
/// # Examples
///
/// Stylesheet:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let css = link()
///     .attr("rel", "stylesheet")
///     .attr("href", "/styles.css")
///     .into_node();
/// // Produces: <link rel="stylesheet" href="/styles.css" />
/// ```
///
/// Favicon:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let icon = link()
///     .attr("rel", "icon")
///     .attr("type", "image/png")
///     .attr("href", "/favicon.png")
///     .into_node();
/// ```
///
/// Preloading a font:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let font = link()
///     .attr("rel", "preload")
///     .attr("href", "/fonts/inter.woff2")
///     .attr("as", "font")
///     .attr("type", "font/woff2")
///     .attr("crossorigin", "anonymous")
///     .into_node();
/// ```
pub fn link() -> HtmlElement { HtmlElement::new_void("link") }

// ── Text/raw constructors ─────────────────────────────────────────────────

/// Creates a standalone [`HtmlNode::Text`] node.
///
/// The content will be HTML-escaped during rendering, making it safe for
/// user-supplied strings. Use this when you need a text node that isn't
/// attached to a parent element yet (e.g. for conditional insertion).
///
/// For adding text directly to an element, the
/// [`.text()`](HtmlElement::text) builder method is more convenient.
///
/// # Arguments
///
/// * `t` - The text content.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// // Standalone text node for conditional use
/// # let logged_in = true;
/// let greeting = if logged_in {
///     text("Welcome back!")
/// } else {
///     text("Please log in.")
/// };
///
/// let banner = div().class("banner").child(greeting).into_node();
/// ```
pub fn text(t: &str) -> HtmlNode {
    HtmlNode::Text(String::from(t))
}

/// Creates a standalone [`HtmlNode::Raw`] node with unescaped HTML.
///
/// The content is inserted verbatim into the rendered output with no
/// escaping. Use this for trusted HTML fragments like rendered markdown,
/// SVG icons, or content from other template systems.
///
/// For adding raw HTML directly to an element, the
/// [`.raw()`](HtmlElement::raw) builder method is more convenient.
///
/// # Safety
///
/// Never pass untrusted or user-supplied strings to this function. Doing so
/// can introduce XSS vulnerabilities. Use [`text()`] for untrusted content.
///
/// # Arguments
///
/// * `html` - A trusted HTML fragment.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// // Embedding rendered markdown
/// let rendered = raw_html("<h2>Title</h2><p>Paragraph from markdown.</p>");
/// let wrapper = div().class("markdown-body").child(rendered).into_node();
/// ```
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// // Embedding an SVG
/// let icon = raw_html("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\"><circle cx=\"12\" cy=\"12\" r=\"10\"/></svg>");
/// let btn = button().class("icon-btn").child(icon).into_node();
/// ```
pub fn raw_html(html: &str) -> HtmlNode {
    HtmlNode::Raw(String::from(html))
}

// ── IntoChildren trait ─────────────────────────────────────────────────────

/// Trait for converting values into a `Vec<HtmlNode>`.
///
/// This trait is the glue between the RSX compiler and the builder API. When
/// the RSX compiler encounters an `{expression}` inside a template, the
/// expression's return value is passed through `into_children()` to produce a
/// `Vec<HtmlNode>` that gets appended to the parent element's children.
///
/// Three implementations are provided out of the box:
///
/// | Type             | Behavior                                              |
/// |------------------|-------------------------------------------------------|
/// | [`HtmlNode`]     | Wrapped in a single-element vector                    |
/// | [`Vec<HtmlNode>`]| Returned as-is (identity conversion)                  |
/// | [`HtmlElement`]  | Converted via `.into_node()`, then wrapped in a vector|
///
/// # When you need this
///
/// You generally don't call `into_children()` directly. The RSX compiler
/// generates calls to it automatically. However, understanding it is useful
/// when writing helper functions that return dynamic children.
///
/// # Examples
///
/// A helper function returning a variable number of nodes:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// fn render_tags(tags: &[&str]) -> Vec<HtmlNode> {
///     let mut nodes = Vec::new();
///     for tag in tags {
///         nodes.push(span().class("tag").text(tag).into_node());
///     }
///     nodes
/// }
///
/// // In RSX, this would be used as {render_tags(&["rust", "cli"])}
/// // The Vec<HtmlNode> return type implements IntoChildren, so the
/// // compiler can splice all the nodes into the parent.
/// ```
///
/// A helper returning a single element:
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// fn render_greeting(name: &str) -> HtmlElement {
///     p().class("greeting").text(name)
/// }
///
/// // In RSX: {render_greeting("Alice")}
/// // HtmlElement implements IntoChildren, so this works seamlessly.
/// ```
pub trait IntoChildren {
    /// Converts `self` into a vector of child nodes for insertion into a
    /// parent element.
    ///
    /// # Returns
    ///
    /// A `Vec<HtmlNode>` containing one or more nodes to be appended as
    /// children.
    fn into_children(self) -> Vec<HtmlNode>;
}

/// Wraps a single [`HtmlNode`] in a one-element vector.
///
/// This allows any `HtmlNode` value (text, raw, or element) to be used
/// directly as an RSX `{expression}` child.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let node = text("hello");
/// let children = node.into_children();
/// assert_eq!(children.len(), 1);
/// ```
impl IntoChildren for HtmlNode {
    fn into_children(self) -> Vec<HtmlNode> {
        let mut v = Vec::new();
        v.push(self);
        v
    }
}

/// Returns the vector as-is — an identity conversion.
///
/// This allows functions that build a dynamic list of nodes to return
/// `Vec<HtmlNode>` directly in RSX expressions without wrapping.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let mut items = Vec::new();
/// items.push(li().text("A").into_node());
/// items.push(li().text("B").into_node());
///
/// let children = items.into_children();
/// assert_eq!(children.len(), 2);
/// ```
impl IntoChildren for Vec<HtmlNode> {
    fn into_children(self) -> Vec<HtmlNode> {
        self
    }
}

/// Converts an [`HtmlElement`] into an [`HtmlNode::Element`] and wraps it in
/// a one-element vector.
///
/// This allows builder-style element expressions to be used directly in RSX
/// `{expression}` slots without manually calling `.into_node()`.
///
/// # Examples
///
/// ```
/// # use volki::libs::web::html::element::*;
/// # use volki::core::volkiwithstds::collections::{String, Vec};
/// let element = div().class("box");
/// let children = element.into_children();
/// assert_eq!(children.len(), 1);
/// ```
impl IntoChildren for HtmlElement {
    fn into_children(self) -> Vec<HtmlNode> {
        let mut v = Vec::new();
        v.push(self.into_node());
        v
    }
}
