package com.volki.jetbrains

import com.intellij.lang.documentation.AbstractDocumentationProvider
import com.intellij.psi.PsiElement

class VolkiDocumentationProvider : AbstractDocumentationProvider() {

    override fun generateDoc(element: PsiElement?, originalElement: PsiElement?): String? {
        val source = originalElement ?: return null
        val tokenType = source.node?.elementType ?: return null

        // Class attribute value — show CSS docs for hovered class name
        if (tokenType == VolkiTokenTypes.STRING && VolkiStyleClassContext.isClassAttributeValue(source)) {
            val offset = originalElement.textRange.startOffset
            val span = VolkiStyleClassContext.getClassAtOffset(source, offset)
            if (span != null) {
                return generateClassDoc(span)
            }
        }

        return when (tokenType) {
            VolkiTokenTypes.TAG_NAME -> generateTagDoc(source.text)
            VolkiTokenTypes.ATTRIBUTE -> generateAttributeDoc(source)
            VolkiTokenTypes.IDENTIFIER -> generateIdentifierDoc(source)
            VolkiTokenTypes.TYPE -> generateIdentifierDoc(source)
            VolkiTokenTypes.KEYWORD -> generateKeywordDoc(source)
            else -> null
        }
    }

    override fun getQuickNavigateInfo(element: PsiElement?, originalElement: PsiElement?): String? {
        val source = originalElement ?: return null
        val tokenType = source.node?.elementType ?: return null

        // Class attribute value — show quick info for hovered class name
        if (tokenType == VolkiTokenTypes.STRING && VolkiStyleClassContext.isClassAttributeValue(source)) {
            val offset = originalElement.textRange.startOffset
            val span = VolkiStyleClassContext.getClassAtOffset(source, offset)
            if (span != null) {
                val parsed = VolkiStyleVariants.parse(span.text)
                val resolved = VolkiStyleResolver.resolve(parsed.utility)
                val category = VolkiStyleResolver.category(parsed.utility)
                return if (resolved != null) {
                    "${span.text}  [${category ?: "volkistyle"}]"
                } else {
                    "${span.text}  [unknown class]"
                }
            }
        }

        return when (tokenType) {
            VolkiTokenTypes.TAG_NAME -> {
                val tag = source.text
                val info = VolkiElementRegistry.getElement(tag)
                if (info != null) {
                    "pub fn ${info.rustConstructor}() -> HtmlElement  [${info.category}]"
                } else if (VolkiDeclarationResolver.isSpecialCompilerTag(tag)) {
                    "<$tag>  [Compiler Element]"
                } else if (tag.isNotEmpty() && tag[0].isUpperCase()) {
                    "fn $tag() -> Html  [Custom Component]"
                } else {
                    null
                }
            }
            VolkiTokenTypes.ATTRIBUTE -> {
                val attrName = source.text
                val tag = findEnclosingTag(source)
                val attr = if (tag != null) VolkiElementRegistry.getAttribute(tag, attrName) else null
                    ?: VolkiElementRegistry.getGlobalAttribute(attrName)
                if (attr != null) {
                    "${attr.name}: ${attr.valueType} — ${attr.description}"
                } else {
                    "$attrName: String"
                }
            }
            VolkiTokenTypes.IDENTIFIER, VolkiTokenTypes.TYPE -> getIdentifierQuickInfo(source)
            VolkiTokenTypes.KEYWORD -> getKeywordQuickInfo(source)
            else -> null
        }
    }

    // --- Class attribute documentation ---

    private fun generateClassDoc(span: VolkiStyleClassContext.ClassNameSpan): String {
        val parsed = VolkiStyleVariants.parse(span.text)
        val resolved = VolkiStyleResolver.resolve(parsed.utility)
        val category = VolkiStyleResolver.category(parsed.utility)

        return buildString {
            append("<div class='definition'><pre>")
            append("<b>${escapeHtml(span.text)}</b>")
            append("</pre></div>")

            append("<div class='content'>")

            if (resolved != null) {
                val declarations = when (resolved) {
                    is VolkiStyleResolver.ResolvedUtility.Standard -> resolved.declarations
                    is VolkiStyleResolver.ResolvedUtility.Custom -> resolved.declarations
                }

                if (category != null) {
                    append("<p><b>Category:</b> $category</p>")
                }

                if (parsed.variants.isNotEmpty()) {
                    append("<p><b>Variants:</b> ${parsed.variants.joinToString(", ")}</p>")
                }

                if (parsed.important) {
                    append("<p><b>!important</b></p>")
                }

                // CSS output — declarations is a single string like "display:flex;"
                val declParts = declarations.split(";").filter { it.isNotBlank() }
                append("<h2>Generated CSS</h2>")
                append("<pre><code>")
                append(".${escapeHtml(span.text)} {\n")
                for (decl in declParts) {
                    append("  ${escapeHtml(decl.trim())};\n")
                }
                append("}")
                append("</code></pre>")

                // Color swatch for color utilities
                val colorName = VolkiStyleResolver.extractColorName(parsed.utility)
                if (colorName != null) {
                    val hex = VolkiStylePalette.colorHex(colorName)
                    if (hex != null) {
                        append("<p><b>Color:</b> <span style='background-color:#$hex;color:#${contrastColor(hex)};padding:2px 8px;border-radius:3px'>#$hex</span> ($colorName)</p>")
                    }
                }
            } else {
                append("<p><b>Unknown volkistyle class.</b></p>")
                append("<p>This class name does not match any known utility in the volkistyle CSS compiler.</p>")
            }

            append("</div>")
        }
    }

    private fun contrastColor(hex: String): String {
        val r = hex.substring(0, 2).toIntOrNull(16) ?: 128
        val g = hex.substring(2, 4).toIntOrNull(16) ?: 128
        val b = hex.substring(4, 6).toIntOrNull(16) ?: 128
        val luminance = 0.299 * r + 0.587 * g + 0.114 * b
        return if (luminance > 128) "000000" else "ffffff"
    }

    // --- Identifier documentation ---

    private fun generateIdentifierDoc(source: PsiElement): String? {
        val text = source.text
        val file = source.containingFile ?: return null
        val project = source.project
        val useKeyword = VolkiImportResolver.findEnclosingUseKeyword(source)

        // Case 1: Path segment in use statement
        if (useKeyword != null && VolkiImportResolver.isFollowedByPathSeparator(source)) {
            val stmt = VolkiImportResolver.findContainingUseStatement(file, source)
            val segIdx = VolkiImportResolver.findSegmentIndex(file, source)
            if (stmt != null && segIdx >= 0) {
                val resolved = VolkiImportResolver.resolvePathSegment(project, stmt.pathSegments, segIdx, file)
                val pathStr = stmt.pathSegments.take(segIdx + 1).joinToString("::") { it.text }
                return buildString {
                    append("<div class='definition'><pre>")
                    append("mod <b>$text</b>")
                    append("</pre></div>")
                    append("<div class='content'>")
                    append("<p>Module in path: <code>$pathStr</code></p>")
                    if (resolved != null) {
                        append("<p><b>File:</b> <code>${resolved.path}</code></p>")
                    }
                    append("</div>")
                }
            }
        }

        // Case 2: Imported symbol in use statement
        if (useKeyword != null && (VolkiImportResolver.isInsideUseBraces(source) || VolkiImportResolver.isPrecededByPathSeparator(source))) {
            val stmt = VolkiImportResolver.findContainingUseStatement(file, source) ?: return null
            val moduleFile = VolkiImportResolver.resolveModulePath(project, stmt.pathSegments, file)
            if (moduleFile != null) {
                val resolved = VolkiImportResolver.findSymbolInFile(project, moduleFile, text)
                    ?: VolkiImportResolver.findExportedSymbols(project, moduleFile).find { it.name == text }
                if (resolved != null) {
                    val modulePath = stmt.pathSegments.joinToString("::") { it.text }
                    return buildString {
                        append("<div class='definition'><pre>")
                        append(escapeHtml(resolved.signature))
                        append("</pre></div>")
                        append("<div class='content'>")
                        append("<p><b>Kind:</b> ${resolved.kind.name.lowercase()}</p>")
                        append("<p><b>Module:</b> <code>$modulePath</code></p>")
                        append("<p><b>File:</b> <code>${resolved.file.path}</code></p>")
                        append("</div>")
                    }
                }
            }
            return null
        }

        // Case 3: Identifier in code/RSX — resolve through imports
        val resolved = VolkiImportResolver.findImportForIdentifier(file, text)
        if (resolved != null) {
            val stmt = findImportStatementForSymbol(file, text)
            val importPath = stmt?.let {
                val pathStr = it.pathSegments.joinToString("::") { seg -> seg.text }
                "use $pathStr::$text"
            }
            return buildString {
                append("<div class='definition'><pre>")
                append(escapeHtml(resolved.signature))
                append("</pre></div>")
                append("<div class='content'>")
                append("<p><b>Kind:</b> ${resolved.kind.name.lowercase()}</p>")
                if (importPath != null) {
                    append("<p><b>Imported via:</b> <code>$importPath</code></p>")
                }
                append("<p><b>File:</b> <code>${resolved.file.path}</code></p>")
                append("</div>")
            }
        }

        // Fall back — check local file for definition
        val psiFile = file
        val fileText = psiFile.text
        val fnPattern = Regex("""(pub\s+)?fn\s+$text\s*(<[^>]*>)?\s*\([^)]*\)(\s*->\s*\S+)?""")
        val match = fnPattern.find(fileText)
        if (match != null) {
            return buildString {
                append("<div class='definition'><pre>")
                append(escapeHtml(match.value.trim()))
                append("</pre></div>")
                append("<div class='content'>")
                append("<p>Defined in current file.</p>")
                append("</div>")
            }
        }

        return null
    }

    private fun getIdentifierQuickInfo(source: PsiElement): String? {
        val text = source.text
        val file = source.containingFile ?: return null
        val project = source.project
        val useKeyword = VolkiImportResolver.findEnclosingUseKeyword(source)

        // Path segment in use statement
        if (useKeyword != null && VolkiImportResolver.isFollowedByPathSeparator(source)) {
            val stmt = VolkiImportResolver.findContainingUseStatement(file, source)
            if (stmt != null) {
                val segIdx = VolkiImportResolver.findSegmentIndex(file, source)
                val pathStr = stmt.pathSegments.take(segIdx + 1).joinToString("::") { it.text }
                return "mod $text  [$pathStr]"
            }
        }

        // Imported symbol in use statement
        if (useKeyword != null && (VolkiImportResolver.isInsideUseBraces(source) || VolkiImportResolver.isPrecededByPathSeparator(source))) {
            val stmt = VolkiImportResolver.findContainingUseStatement(file, source) ?: return null
            val moduleFile = VolkiImportResolver.resolveModulePath(project, stmt.pathSegments, file)
            if (moduleFile != null) {
                val resolved = VolkiImportResolver.findSymbolInFile(project, moduleFile, text)
                    ?: VolkiImportResolver.findExportedSymbols(project, moduleFile).find { it.name == text }
                if (resolved != null) {
                    val modulePath = stmt.pathSegments.joinToString("::") { it.text }
                    return "${resolved.signature}  [$modulePath]"
                }
            }
            return null
        }

        // Code/RSX identifier — resolve through imports
        val resolved = VolkiImportResolver.findImportForIdentifier(file, text)
        if (resolved != null) {
            val stmt = findImportStatementForSymbol(file, text)
            val modulePath = stmt?.pathSegments?.joinToString("::") { it.text } ?: ""
            return "${resolved.signature}  [$modulePath]"
        }

        return null
    }

    // --- Keyword documentation ---

    private fun generateKeywordDoc(source: PsiElement): String? {
        val text = source.text
        VolkiImportResolver.findEnclosingUseKeyword(source) ?: return null

        return when (text) {
            "crate" -> buildString {
                append("<div class='definition'><pre>")
                append("<b>crate</b>")
                append("</pre></div>")
                append("<div class='content'>")
                append("<p>Refers to the root of the current crate (<code>src/</code>).</p>")
                append("</div>")
            }
            "self" -> buildString {
                append("<div class='definition'><pre>")
                append("<b>self</b>")
                append("</pre></div>")
                append("<div class='content'>")
                append("<p>Refers to the current module.</p>")
                append("</div>")
            }
            "super" -> buildString {
                append("<div class='definition'><pre>")
                append("<b>super</b>")
                append("</pre></div>")
                append("<div class='content'>")
                append("<p>Refers to the parent module.</p>")
                append("</div>")
            }
            else -> null
        }
    }

    private fun getKeywordQuickInfo(source: PsiElement): String? {
        val text = source.text
        VolkiImportResolver.findEnclosingUseKeyword(source) ?: return null

        return when (text) {
            "crate" -> "crate — root of current crate"
            "self" -> "self — current module"
            "super" -> "super — parent module"
            else -> null
        }
    }

    // --- Helpers ---

    private fun findImportStatementForSymbol(file: PsiElement, symbolText: String): VolkiImportResolver.ParsedUseStatement? {
        val psiFile = file as? com.intellij.psi.PsiFile ?: file.containingFile ?: return null
        val stmts = VolkiImportResolver.parseUseStatements(psiFile)
        for (stmt in stmts) {
            for (sym in stmt.symbols) {
                if (sym.text == symbolText || sym.isGlob) return stmt
            }
        }
        return null
    }

    // --- Existing tag/attribute documentation ---

    private fun generateTagDoc(tag: String): String {
        val info = VolkiElementRegistry.getElement(tag)
        if (info != null) {
            return buildString {
                append("<div class='definition'><pre>")
                append("pub fn <b>${info.rustConstructor}</b>() -&gt; HtmlElement")
                append("</pre></div>")

                append("<div class='content'>")
                append("<p>${info.description}</p>")
                append("<p><b>Category:</b> ${info.category}")
                if (info.isVoid) append(" &nbsp;|&nbsp; <b>Self-closing</b>")
                append("</p>")

                append("<p><b>RSX:</b> <code>&lt;${info.tag}&gt;</code> &rarr; <code>${info.rustConstructor}()</code></p>")

                // Attributes table
                val attrs = info.attributes
                if (attrs.isNotEmpty()) {
                    append("<h2>Attributes</h2>")
                    append("<table>")
                    append("<tr><th>Name</th><th>Type</th><th>Description</th></tr>")
                    for (attr in attrs) {
                        append("<tr>")
                        append("<td><code>${attr.name}</code></td>")
                        append("<td>${attr.valueType}</td>")
                        append("<td>${attr.description}</td>")
                        append("</tr>")
                    }
                    append("</table>")
                }

                // Builder methods table
                val methods = info.builderMethods
                if (methods.isNotEmpty()) {
                    append("<h2>Builder Methods</h2>")
                    append("<table>")
                    append("<tr><th>Method</th><th>Signature</th><th>Description</th></tr>")
                    for (m in methods) {
                        append("<tr>")
                        append("<td><code>.${m.name}()</code></td>")
                        append("<td><code>${escapeHtml(m.signature)}</code></td>")
                        append("<td>${m.description}</td>")
                        append("</tr>")
                    }
                    append("</table>")
                }

                append("</div>")
            }
        }

        // Special compiler element (Style, Head, Stylesheet)
        if (VolkiDeclarationResolver.isSpecialCompilerTag(tag)) {
            val desc = when (tag) {
                "Style" -> "Compiler element for scoped CSS. Content is extracted at compile time, processed through the volkistyle CSS compiler, and injected into the document head."
                "Head" -> "Compiler element for injecting nodes into the document &lt;head&gt; section. Children are moved to &lt;head&gt; at compile time."
                "Stylesheet" -> "Compiler element for linking external stylesheets. Generates a &lt;link rel=\"stylesheet\"&gt; tag in the document head."
                else -> "Special compiler element."
            }
            return buildString {
                append("<div class='definition'><pre>")
                append("&lt;<b>$tag</b>&gt;  [Compiler Element]")
                append("</pre></div>")
                append("<div class='content'>")
                append("<p>$desc</p>")
                append("<p>This is <b>not</b> a regular HTML element. It is processed at compile time and does not appear in the rendered output.</p>")
                append("</div>")
            }
        }

        // Custom component (uppercase, not in registry)
        if (tag.isNotEmpty() && tag[0].isUpperCase()) {
            return buildString {
                append("<div class='definition'><pre>")
                append("fn <b>$tag</b>() -&gt; Html")
                append("</pre></div>")
                append("<div class='content'>")
                append("<p>Custom RSX component. Compiles to a function call <code>${tag}()</code>.</p>")
                append("<p>Cmd+click to navigate to its definition in the project.</p>")
                append("</div>")
            }
        }

        return "<p>Unknown tag: <code>&lt;$tag&gt;</code></p>"
    }

    private fun generateAttributeDoc(element: PsiElement): String {
        val attrName = element.text
        val tag = findEnclosingTag(element)

        val attr = if (tag != null) VolkiElementRegistry.getAttribute(tag, attrName) else null
            ?: VolkiElementRegistry.getGlobalAttribute(attrName)

        return buildString {
            append("<div class='definition'><pre>")
            if (attr != null) {
                append("<b>${attr.name}</b>: ${attr.valueType}")
            } else {
                append("<b>$attrName</b>: String")
            }
            append("</pre></div>")

            append("<div class='content'>")
            if (attr != null) {
                append("<p>${attr.description}</p>")
            }

            if (tag != null) {
                append("<p><b>Element:</b> <code>&lt;$tag&gt;</code></p>")
            }

            // Show compilation info
            append("<h2>Compilation</h2>")
            when (attrName) {
                "class" -> {
                    append("<p>Compiles to builder method: <code>.class(\"...\")</code></p>")
                    val m = VolkiElementRegistry.getBuilderMethod("class")
                    if (m != null) append("<p><code>${escapeHtml(m.signature)}</code></p>")
                }
                "id" -> {
                    append("<p>Compiles to builder method: <code>.id(\"...\")</code></p>")
                    val m = VolkiElementRegistry.getBuilderMethod("id")
                    if (m != null) append("<p><code>${escapeHtml(m.signature)}</code></p>")
                }
                else -> {
                    append("<p>Compiles to: <code>.attr(\"$attrName\", \"...\")</code></p>")
                    val m = VolkiElementRegistry.getBuilderMethod("attr")
                    if (m != null) append("<p><code>${escapeHtml(m.signature)}</code></p>")
                }
            }

            append("</div>")
        }
    }

    private fun findEnclosingTag(element: PsiElement): String? {
        var sibling = element.prevSibling
        while (sibling != null) {
            if (sibling.node?.elementType == VolkiTokenTypes.TAG_NAME) {
                return sibling.text
            }
            // Stop searching if we hit a tag bracket that isn't part of the opening tag
            val type = sibling.node?.elementType
            if (type == VolkiTokenTypes.BRACE_OPEN || type == VolkiTokenTypes.BRACE_CLOSE) {
                break
            }
            sibling = sibling.prevSibling
        }
        return null
    }

    private fun escapeHtml(text: String): String {
        return text
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
    }
}
