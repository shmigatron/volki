package com.volki.jetbrains

import com.intellij.codeInsight.completion.*
import com.intellij.codeInsight.lookup.LookupElementBuilder
import com.intellij.patterns.PlatformPatterns
import com.intellij.psi.PsiElement
import com.intellij.psi.TokenType
import com.intellij.util.ProcessingContext
import java.awt.Color
import java.awt.Component
import java.awt.Graphics
import javax.swing.Icon

class VolkiCompletionContributor : CompletionContributor() {

    init {
        // Complete tag names after < or </
        extend(
            CompletionType.BASIC,
            PlatformPatterns.psiElement(VolkiTokenTypes.TAG_NAME),
            TagNameCompletionProvider()
        )

        // Complete attribute names inside tags
        extend(
            CompletionType.BASIC,
            PlatformPatterns.psiElement(VolkiTokenTypes.ATTRIBUTE),
            AttributeCompletionProvider()
        )

        // Complete attribute values (e.g. type="..." for input)
        extend(
            CompletionType.BASIC,
            PlatformPatterns.psiElement(VolkiTokenTypes.STRING),
            AttributeValueCompletionProvider()
        )
    }

    private class TagNameCompletionProvider : CompletionProvider<CompletionParameters>() {
        override fun addCompletions(
            parameters: CompletionParameters,
            context: ProcessingContext,
            result: CompletionResultSet
        ) {
            val element = parameters.position
            val isClosingTag = isInClosingTag(element)

            if (isClosingTag) {
                // For closing tags, suggest from the open tag stack
                addClosingTagCompletions(element, result)
            } else {
                // For opening tags, suggest all known HTML elements
                addOpeningTagCompletions(result)
            }
        }

        private fun addOpeningTagCompletions(result: CompletionResultSet) {
            // Built-in HTML elements
            val categories = mutableMapOf<String, MutableList<HtmlElementInfo>>()
            for (tag in COMMON_TAGS) {
                val info = VolkiElementRegistry.getElement(tag) ?: continue
                categories.getOrPut(info.category) { mutableListOf() }.add(info)
            }

            for ((_, elements) in categories) {
                for (info in elements) {
                    val builder = LookupElementBuilder.create(info.tag)
                        .withTypeText(info.category)
                        .withTailText(if (info.isVoid) " (self-closing)" else "")
                        .withInsertHandler { ctx, _ ->
                            val editor = ctx.editor
                            val doc = editor.document
                            val offset = ctx.tailOffset
                            if (info.isVoid) {
                                doc.insertString(offset, " />")
                                editor.caretModel.moveToOffset(offset + 1) // cursor after space
                            } else {
                                doc.insertString(offset, "></${info.tag}>")
                                editor.caretModel.moveToOffset(offset + 1) // cursor after >
                            }
                        }
                        .bold()
                    result.addElement(builder)
                }
            }

            // Special compiler elements
            for (tag in SPECIAL_COMPILER_TAGS) {
                val desc = when (tag) {
                    "Style" -> "Scoped CSS block"
                    "Head" -> "Inject into <head>"
                    "Stylesheet" -> "Link stylesheet"
                    else -> ""
                }
                val isSelfClosing = tag == "Stylesheet"
                val builder = LookupElementBuilder.create(tag)
                    .withTypeText("Compiler")
                    .withTailText(if (isSelfClosing) " (self-closing)" else "")
                    .withInsertHandler { ctx, _ ->
                        val editor = ctx.editor
                        val doc = editor.document
                        val offset = ctx.tailOffset
                        if (isSelfClosing) {
                            doc.insertString(offset, " />")
                            editor.caretModel.moveToOffset(offset + 1)
                        } else {
                            doc.insertString(offset, "></$tag>")
                            editor.caretModel.moveToOffset(offset + 1)
                        }
                    }
                    .bold()
                result.addElement(builder)
            }
        }

        private fun addClosingTagCompletions(element: PsiElement, result: CompletionResultSet) {
            // Walk backwards to find unclosed opening tags
            val unclosed = findUnclosedTags(element)
            for ((index, tag) in unclosed.reversed().withIndex()) {
                val builder = LookupElementBuilder.create(tag)
                    .withTypeText("Close tag")
                    .withInsertHandler { ctx, _ ->
                        val editor = ctx.editor
                        val doc = editor.document
                        doc.insertString(ctx.tailOffset, ">")
                        editor.caretModel.moveToOffset(ctx.tailOffset + 1)
                    }
                    .withPriority((unclosed.size - index).toDouble())
                result.addElement(builder)
            }
        }

        private fun isInClosingTag(element: PsiElement): Boolean {
            var sibling = element.prevSibling
            while (sibling != null) {
                if (sibling.node?.elementType == VolkiTokenTypes.TAG_BRACKET) {
                    return sibling.text == "</"
                }
                if (sibling.node?.elementType != TokenType.WHITE_SPACE) break
                sibling = sibling.prevSibling
            }
            return false
        }

        private fun findUnclosedTags(fromElement: PsiElement): List<String> {
            val stack = mutableListOf<String>()
            val file = fromElement.containingFile ?: return emptyList()
            var current: PsiElement? = file.firstChild

            while (current != null && current != fromElement) {
                if (current.node?.elementType == VolkiTokenTypes.TAG_BRACKET) {
                    when (current.text) {
                        "<" -> {
                            val tagName = findNextTagName(current)
                            if (tagName != null && !isVoidTag(tagName) && !isSelfClosingFrom(current)) {
                                stack.add(tagName)
                            }
                        }
                        "</" -> {
                            val tagName = findNextTagName(current)
                            if (tagName != null && stack.isNotEmpty()) {
                                val idx = stack.indexOfLast { it == tagName }
                                if (idx >= 0) {
                                    stack.removeAt(idx)
                                }
                            }
                        }
                    }
                }
                current = current.nextSibling
            }
            return stack
        }

        private fun isVoidTag(tag: String): Boolean =
            VolkiElementRegistry.getElement(tag)?.isVoid == true

        private fun isSelfClosingFrom(bracket: PsiElement): Boolean {
            var cur: PsiElement? = bracket.nextSibling
            var braceDepth = 0
            while (cur != null) {
                val type = cur.node?.elementType
                if (type == VolkiTokenTypes.BRACE_OPEN) braceDepth++
                else if (type == VolkiTokenTypes.BRACE_CLOSE) braceDepth--
                if (braceDepth == 0) {
                    if (type == VolkiTokenTypes.TAG_BRACKET && cur.text == "/>") return true
                    if (type == VolkiTokenTypes.TAG_BRACKET && cur.text == ">") return false
                    if (type == VolkiTokenTypes.TAG_BRACKET && (cur.text == "<" || cur.text == "</")) return false
                }
                cur = cur.nextSibling
            }
            return false
        }

        private fun findNextTagName(bracket: PsiElement): String? {
            var cur = bracket.nextSibling
            while (cur != null && cur.node?.elementType == TokenType.WHITE_SPACE) {
                cur = cur.nextSibling
            }
            return if (cur?.node?.elementType == VolkiTokenTypes.TAG_NAME) cur.text else null
        }
    }

    private class AttributeCompletionProvider : CompletionProvider<CompletionParameters>() {
        override fun addCompletions(
            parameters: CompletionParameters,
            context: ProcessingContext,
            result: CompletionResultSet
        ) {
            val element = parameters.position
            val tag = findEnclosingTagName(element) ?: return
            val info = VolkiElementRegistry.getElement(tag) ?: return

            for (attr in info.attributes) {
                val builder = LookupElementBuilder.create(attr.name)
                    .withTypeText(attr.valueType)
                    .withTailText("=\"...\"")
                    .withInsertHandler { ctx, _ ->
                        val editor = ctx.editor
                        val doc = editor.document
                        val offset = ctx.tailOffset
                        if (attr.valueType == "Boolean") {
                            // Boolean attrs don't need a value
                        } else {
                            doc.insertString(offset, "=\"\"")
                            editor.caretModel.moveToOffset(offset + 2)
                        }
                    }
                result.addElement(builder)
            }
        }
    }

    private class AttributeValueCompletionProvider : CompletionProvider<CompletionParameters>() {
        override fun addCompletions(
            parameters: CompletionParameters,
            context: ProcessingContext,
            result: CompletionResultSet
        ) {
            val element = parameters.position
            val attrName = findPrecedingAttributeName(element) ?: return

            if (attrName == "class") {
                addClassCompletions(parameters, element, result)
                return
            }

            val values = KNOWN_ATTRIBUTE_VALUES[attrName] ?: return
            for (value in values) {
                result.addElement(LookupElementBuilder.create(value).withTypeText(attrName))
            }
        }

        private fun addClassCompletions(
            parameters: CompletionParameters,
            element: PsiElement,
            result: CompletionResultSet
        ) {
            // Compute the current word prefix within the class string
            val text = element.text
            val caretOffsetInFile = parameters.offset
            val elementStart = element.textRange.startOffset
            val caretInElement = caretOffsetInFile - elementStart

            // Find the start of the current word (after last whitespace or opening quote)
            val inner = if (text.length >= 2 && (text.startsWith("\"") || text.startsWith("'"))) {
                text.substring(1)
            } else text
            val innerOffset = if (text.length >= 2 && (text.startsWith("\"") || text.startsWith("'"))) 1 else 0
            val posInInner = caretInElement - innerOffset

            var wordStart = posInInner
            while (wordStart > 0 && !inner[wordStart - 1].isWhitespace()) {
                wordStart--
            }
            val currentPrefix = if (posInInner > wordStart) inner.substring(wordStart, posInInner) else ""

            // Use a prefix matcher for just the current word
            val wordResult = result.withPrefixMatcher(currentPrefix)

            // Detect variant chain prefix (everything up to last :)
            val lastColon = currentPrefix.lastIndexOf(':')
            val variantChain = if (lastColon >= 0) currentPrefix.substring(0, lastColon + 1) else ""

            // Add variant prefix completions when typing after a colon or at start
            if (lastColon >= 0 || currentPrefix.isEmpty()) {
                for (variant in VolkiStyleVariants.VARIANT_PREFIXES) {
                    val insertText = variantChain + variant + ":"
                    val builder = LookupElementBuilder.create(insertText)
                        .withTypeText("Variant")
                        .withPresentableText(variant + ":")
                        .withLookupString(insertText)
                    wordResult.addElement(builder)
                }
            }

            // Add static utility completions
            for (entry in VolkiStyleCompletions.ALL_ENTRIES) {
                val insertText = variantChain + entry.className
                val builder = LookupElementBuilder.create(insertText)
                    .withTypeText(entry.category)
                    .withTailText("  ${entry.cssPreview}", true)
                    .withPresentableText(entry.className)
                    .withLookupString(insertText)
                wordResult.addElement(builder)
            }

            // Add dynamic prefix completions with common values
            for (prefix in VolkiStyleCompletions.DYNAMIC_PREFIXES) {
                for (example in prefix.examples) {
                    val className = prefix.prefix + example
                    val insertText = variantChain + className
                    val resolved = VolkiStyleResolver.resolve(className)
                    val preview = if (resolved != null) {
                        when (resolved) {
                            is VolkiStyleResolver.ResolvedUtility.Standard -> resolved.declarations
                            is VolkiStyleResolver.ResolvedUtility.Custom -> resolved.declarations
                        }
                    } else ""
                    val builder = LookupElementBuilder.create(insertText)
                        .withTypeText(prefix.category)
                        .withTailText("  $preview", true)
                        .withPresentableText(className)
                        .withLookupString(insertText)
                    wordResult.addElement(builder)
                }
            }

            // Add color completions with colored icons
            for (colorPrefix in VolkiStyleCompletions.COLOR_PREFIXES) {
                for (family in VolkiStylePalette.COLOR_FAMILIES) {
                    for (shade in VolkiStylePalette.SHADES) {
                        val colorName = "$family-$shade"
                        val className = "$colorPrefix$colorName"
                        val awtColor = VolkiStylePalette.colorToAwtColor(colorName) ?: continue
                        val insertText = variantChain + className
                        val builder = LookupElementBuilder.create(insertText)
                            .withTypeText("Color")
                            .withPresentableText(className)
                            .withLookupString(insertText)
                            .withIcon(CompletionColorIcon(10, awtColor))
                        wordResult.addElement(builder)
                    }
                }
                // Also add white/black/transparent/current/inherit
                for (special in listOf("white", "black", "transparent", "current", "inherit")) {
                    val className = "$colorPrefix$special"
                    val awtColor = VolkiStylePalette.colorToAwtColor(special)
                    val insertText = variantChain + className
                    val builder = LookupElementBuilder.create(insertText)
                        .withTypeText("Color")
                        .withPresentableText(className)
                        .withLookupString(insertText)
                    val withIcon = if (awtColor != null) {
                        builder.withIcon(CompletionColorIcon(10, awtColor))
                    } else builder
                    wordResult.addElement(withIcon)
                }
            }
        }
    }

    private class CompletionColorIcon(private val size: Int, private val color: Color) : Icon {
        override fun paintIcon(c: Component?, g: Graphics, x: Int, y: Int) {
            g.color = color
            g.fillRect(x + 1, y + 1, size - 2, size - 2)
            g.color = Color(
                (color.red * 0.6).toInt(),
                (color.green * 0.6).toInt(),
                (color.blue * 0.6).toInt()
            )
            g.drawRect(x, y, size - 1, size - 1)
        }
        override fun getIconWidth(): Int = size
        override fun getIconHeight(): Int = size
    }

    companion object {
        private val SPECIAL_COMPILER_TAGS = listOf("Style", "Head", "Stylesheet")

        private val COMMON_TAGS = listOf(
            "div", "span", "p", "a", "h1", "h2", "h3", "h4", "h5", "h6",
            "header", "footer", "main", "nav", "section", "article",
            "ul", "ol", "li", "table", "thead", "tbody", "tr", "th", "td",
            "form", "input", "button", "label", "textarea", "select", "option",
            "img", "br", "hr", "pre", "code", "blockquote",
            "strong", "em", "small",
            "script", "style", "meta", "link"
        )

        private val KNOWN_ATTRIBUTE_VALUES = mapOf(
            "type" to listOf("text", "password", "email", "number", "tel", "url", "search",
                "checkbox", "radio", "submit", "button", "reset", "file", "hidden",
                "date", "time", "datetime-local", "color", "range"),
            "target" to listOf("_blank", "_self", "_parent", "_top"),
            "method" to listOf("GET", "POST"),
            "rel" to listOf("stylesheet", "icon", "preconnect", "preload", "noopener", "noreferrer"),
            "loading" to listOf("lazy", "eager"),
            "enctype" to listOf("application/x-www-form-urlencoded", "multipart/form-data", "text/plain"),
        )

        private fun findEnclosingTagName(element: PsiElement): String? {
            var sibling = element.prevSibling
            while (sibling != null) {
                if (sibling.node?.elementType == VolkiTokenTypes.TAG_NAME) return sibling.text
                val type = sibling.node?.elementType
                if (type == VolkiTokenTypes.TAG_BRACKET) break
                if (type == VolkiTokenTypes.BRACE_OPEN || type == VolkiTokenTypes.BRACE_CLOSE) break
                sibling = sibling.prevSibling
            }
            return null
        }

        private fun findPrecedingAttributeName(element: PsiElement): String? {
            var sibling = element.prevSibling
            while (sibling != null) {
                if (sibling.node?.elementType == VolkiTokenTypes.ATTRIBUTE) return sibling.text
                if (sibling.node?.elementType == VolkiTokenTypes.TAG_BRACKET) break
                sibling = sibling.prevSibling
            }
            return null
        }

        @Suppress("unused")
        private fun LookupElementBuilder.withPriority(@Suppress("UNUSED_PARAMETER") priority: Double): LookupElementBuilder {
            return this
        }
    }
}
