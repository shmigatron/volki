package com.volki.jetbrains

data class HtmlElementInfo(
    val tag: String,
    val description: String,
    val category: String,
    val isVoid: Boolean,
    val attributes: List<HtmlAttributeInfo>,
    val builderMethods: List<BuilderMethodInfo>,
    val rustConstructor: String
)

data class HtmlAttributeInfo(
    val name: String,
    val description: String,
    val valueType: String
)

data class BuilderMethodInfo(
    val name: String,
    val signature: String,
    val description: String
)

object VolkiElementRegistry {

    private val globalAttributes = listOf(
        HtmlAttributeInfo("class", "Space-separated CSS class names", "String"),
        HtmlAttributeInfo("id", "Unique identifier for the element", "String"),
        HtmlAttributeInfo("style", "Inline CSS styles", "String"),
        HtmlAttributeInfo("title", "Advisory text shown on hover", "String"),
        HtmlAttributeInfo("hidden", "Indicates the element is not yet relevant", "Boolean"),
        HtmlAttributeInfo("tabindex", "Tab order of the element", "Integer"),
        HtmlAttributeInfo("data-*", "Custom data attributes", "String"),
        HtmlAttributeInfo("onclick", "Click event handler", "EventHandler"),
        HtmlAttributeInfo("onchange", "Change event handler", "EventHandler"),
        HtmlAttributeInfo("onsubmit", "Submit event handler", "EventHandler"),
        HtmlAttributeInfo("oninput", "Input event handler", "EventHandler"),
        HtmlAttributeInfo("onkeydown", "Key down event handler", "EventHandler"),
        HtmlAttributeInfo("onkeyup", "Key up event handler", "EventHandler"),
        HtmlAttributeInfo("onfocus", "Focus event handler", "EventHandler"),
        HtmlAttributeInfo("onblur", "Blur event handler", "EventHandler"),
        HtmlAttributeInfo("onmouseover", "Mouse over event handler", "EventHandler"),
        HtmlAttributeInfo("onmouseout", "Mouse out event handler", "EventHandler")
    )

    val builderMethods = listOf(
        BuilderMethodInfo("attr", "pub fn attr(mut self, name: &str, value: &str) -> Self", "Adds a generic HTML attribute to the element"),
        BuilderMethodInfo("class", "pub fn class(self, value: &str) -> Self", "Sets the CSS class attribute (convenience for .attr(\"class\", ...))"),
        BuilderMethodInfo("id", "pub fn id(self, value: &str) -> Self", "Sets the id attribute (convenience for .attr(\"id\", ...))"),
        BuilderMethodInfo("child", "pub fn child(mut self, node: HtmlNode) -> Self", "Adds a single child node to the element"),
        BuilderMethodInfo("text", "pub fn text(mut self, t: &str) -> Self", "Adds text content as a child node"),
        BuilderMethodInfo("raw", "pub fn raw(mut self, html: &str) -> Self", "Adds raw HTML as a child node"),
        BuilderMethodInfo("children", "pub fn children(mut self, nodes: Vec<HtmlNode>) -> Self", "Adds multiple child nodes at once"),
        BuilderMethodInfo("into_node", "pub fn into_node(self) -> HtmlNode", "Converts the HtmlElement into an HtmlNode")
    )

    private val anchorAttributes = listOf(
        HtmlAttributeInfo("href", "URL the link points to", "URL"),
        HtmlAttributeInfo("target", "Browsing context for the link (_blank, _self, etc.)", "String"),
        HtmlAttributeInfo("rel", "Relationship between current and linked document", "String"),
        HtmlAttributeInfo("download", "Download the linked resource instead of navigating", "String")
    )

    private val imgAttributes = listOf(
        HtmlAttributeInfo("src", "URL of the image", "URL"),
        HtmlAttributeInfo("alt", "Alternative text description", "String"),
        HtmlAttributeInfo("width", "Display width of the image", "String"),
        HtmlAttributeInfo("height", "Display height of the image", "String"),
        HtmlAttributeInfo("loading", "Loading behavior (lazy, eager)", "String")
    )

    private val inputAttributes = listOf(
        HtmlAttributeInfo("type", "Type of input control (text, password, email, etc.)", "String"),
        HtmlAttributeInfo("name", "Name of the input for form submission", "String"),
        HtmlAttributeInfo("value", "Current value of the input", "String"),
        HtmlAttributeInfo("placeholder", "Placeholder text when empty", "String"),
        HtmlAttributeInfo("required", "Whether the field is required", "Boolean"),
        HtmlAttributeInfo("disabled", "Whether the input is disabled", "Boolean"),
        HtmlAttributeInfo("readonly", "Whether the input is read-only", "Boolean"),
        HtmlAttributeInfo("checked", "Whether a checkbox/radio is checked", "Boolean"),
        HtmlAttributeInfo("min", "Minimum value for numeric inputs", "String"),
        HtmlAttributeInfo("max", "Maximum value for numeric inputs", "String"),
        HtmlAttributeInfo("pattern", "Regex pattern for validation", "String")
    )

    private val formAttributes = listOf(
        HtmlAttributeInfo("action", "URL to submit the form to", "URL"),
        HtmlAttributeInfo("method", "HTTP method (GET, POST)", "String"),
        HtmlAttributeInfo("enctype", "Encoding type for form data", "String")
    )

    private val buttonAttributes = listOf(
        HtmlAttributeInfo("type", "Button type (submit, button, reset)", "String"),
        HtmlAttributeInfo("disabled", "Whether the button is disabled", "Boolean"),
        HtmlAttributeInfo("name", "Name for form submission", "String"),
        HtmlAttributeInfo("value", "Value for form submission", "String")
    )

    private val textareaAttributes = listOf(
        HtmlAttributeInfo("name", "Name for form submission", "String"),
        HtmlAttributeInfo("rows", "Visible number of text lines", "Integer"),
        HtmlAttributeInfo("cols", "Visible width in characters", "Integer"),
        HtmlAttributeInfo("placeholder", "Placeholder text when empty", "String"),
        HtmlAttributeInfo("required", "Whether the field is required", "Boolean"),
        HtmlAttributeInfo("disabled", "Whether the textarea is disabled", "Boolean"),
        HtmlAttributeInfo("readonly", "Whether the textarea is read-only", "Boolean")
    )

    private val selectAttributes = listOf(
        HtmlAttributeInfo("name", "Name for form submission", "String"),
        HtmlAttributeInfo("multiple", "Allow multiple selections", "Boolean"),
        HtmlAttributeInfo("required", "Whether a selection is required", "Boolean"),
        HtmlAttributeInfo("disabled", "Whether the select is disabled", "Boolean")
    )

    private val optionAttributes = listOf(
        HtmlAttributeInfo("value", "Value submitted with the form", "String"),
        HtmlAttributeInfo("selected", "Whether the option is pre-selected", "Boolean"),
        HtmlAttributeInfo("disabled", "Whether the option is disabled", "Boolean")
    )

    private val labelAttributes = listOf(
        HtmlAttributeInfo("for", "ID of the form element this label is for", "String")
    )

    private val metaAttributes = listOf(
        HtmlAttributeInfo("name", "Metadata name (viewport, description, etc.)", "String"),
        HtmlAttributeInfo("content", "Value of the metadata", "String"),
        HtmlAttributeInfo("charset", "Character encoding declaration", "String"),
        HtmlAttributeInfo("http-equiv", "Pragma directive", "String"),
        HtmlAttributeInfo("property", "Open Graph / social meta property", "String")
    )

    private val linkAttributes = listOf(
        HtmlAttributeInfo("rel", "Relationship (stylesheet, icon, etc.)", "String"),
        HtmlAttributeInfo("href", "URL of the linked resource", "URL"),
        HtmlAttributeInfo("type", "MIME type of the linked resource", "String"),
        HtmlAttributeInfo("media", "Media query for conditional loading", "String")
    )

    private val scriptAttributes = listOf(
        HtmlAttributeInfo("src", "URL of an external script", "URL"),
        HtmlAttributeInfo("type", "Script MIME type or module", "String"),
        HtmlAttributeInfo("defer", "Defer script execution", "Boolean"),
        HtmlAttributeInfo("async", "Execute script asynchronously", "Boolean")
    )

    private val tdThAttributes = listOf(
        HtmlAttributeInfo("colspan", "Number of columns the cell spans", "Integer"),
        HtmlAttributeInfo("rowspan", "Number of rows the cell spans", "Integer")
    )

    private fun element(
        tag: String,
        description: String,
        category: String,
        isVoid: Boolean = false,
        extraAttributes: List<HtmlAttributeInfo> = emptyList(),
        constructorOverride: String? = null
    ): HtmlElementInfo {
        val constructor = constructorOverride ?: tag
        return HtmlElementInfo(
            tag = tag,
            description = description,
            category = category,
            isVoid = isVoid,
            attributes = globalAttributes + extraAttributes,
            builderMethods = if (isVoid) builderMethods.filter { it.name != "child" && it.name != "text" && it.name != "raw" && it.name != "children" } else builderMethods,
            rustConstructor = constructor
        )
    }

    private val elements: Map<String, HtmlElementInfo> = listOf(
        // Layout
        element("div", "Generic container element for grouping content", "Layout"),
        element("span", "Inline container for phrasing content", "Layout"),
        element("header", "Introductory content or navigational aids", "Layout"),
        element("footer", "Footer for its nearest sectioning content", "Layout"),
        element("main", "Dominant content of the document body", "Layout", constructorOverride = "main_el"),
        element("nav", "Section with navigation links", "Layout"),
        element("section", "Standalone section of content", "Layout"),
        element("article", "Self-contained composition (blog post, comment, etc.)", "Layout"),

        // Text
        element("p", "Paragraph of text", "Text"),
        element("h1", "Top-level heading", "Text"),
        element("h2", "Second-level heading", "Text"),
        element("h3", "Third-level heading", "Text"),
        element("h4", "Fourth-level heading", "Text"),
        element("h5", "Fifth-level heading", "Text"),
        element("h6", "Sixth-level heading", "Text"),
        element("strong", "Strong importance (typically bold)", "Text"),
        element("em", "Emphasis (typically italic)", "Text"),
        element("pre", "Preformatted text preserving whitespace", "Text"),
        element("code", "Fragment of computer code", "Text"),
        element("blockquote", "Block quotation from another source", "Text"),

        // Links & Media
        element("a", "Hyperlink to another page or resource", "Links & Media", extraAttributes = anchorAttributes),
        element("img", "Embeds an image", "Links & Media", isVoid = true, extraAttributes = imgAttributes),

        // Lists
        element("ul", "Unordered (bulleted) list", "Lists"),
        element("ol", "Ordered (numbered) list", "Lists"),
        element("li", "List item within ul or ol", "Lists"),

        // Tables
        element("table", "Table container", "Tables"),
        element("thead", "Table header row group", "Tables"),
        element("tbody", "Table body row group", "Tables"),
        element("tr", "Table row", "Tables"),
        element("th", "Table header cell", "Tables", extraAttributes = tdThAttributes),
        element("td", "Table data cell", "Tables", extraAttributes = tdThAttributes),

        // Forms
        element("form", "Form for user input", "Forms", extraAttributes = formAttributes),
        element("button", "Clickable button", "Forms", extraAttributes = buttonAttributes),
        element("label", "Caption for a form control", "Forms", extraAttributes = labelAttributes),
        element("input", "Interactive form control", "Forms", isVoid = true, extraAttributes = inputAttributes),
        element("textarea", "Multi-line text input", "Forms", extraAttributes = textareaAttributes),
        element("select", "Drop-down selection control", "Forms", extraAttributes = selectAttributes),
        element("option", "Option within a select element", "Forms", extraAttributes = optionAttributes),

        // Scripting & Style
        element("script", "Embedded or external script", "Scripting", extraAttributes = scriptAttributes),
        element("style", "Embedded CSS styles", "Scripting"),

        // Void / Metadata
        element("br", "Line break", "Void", isVoid = true),
        element("hr", "Thematic break (horizontal rule)", "Void", isVoid = true),
        element("meta", "Document metadata", "Metadata", isVoid = true, extraAttributes = metaAttributes),
        element("link", "External resource link (stylesheets, icons)", "Metadata", isVoid = true, extraAttributes = linkAttributes)
    ).associateBy { it.tag }

    fun getElement(tag: String): HtmlElementInfo? = elements[tag.lowercase()]

    fun isBuiltinTag(tag: String): Boolean = elements.containsKey(tag.lowercase())

    fun getAttribute(tag: String, attrName: String): HtmlAttributeInfo? {
        val element = getElement(tag) ?: return null
        return element.attributes.find { it.name == attrName }
    }

    fun getGlobalAttribute(attrName: String): HtmlAttributeInfo? {
        return globalAttributes.find { it.name == attrName }
    }

    fun getBuilderMethod(name: String): BuilderMethodInfo? {
        return builderMethods.find { it.name == name }
    }
}
