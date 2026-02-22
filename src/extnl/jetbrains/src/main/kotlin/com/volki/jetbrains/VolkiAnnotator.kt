package com.volki.jetbrains

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.editor.markup.GutterIconRenderer
import com.intellij.psi.PsiElement
import com.intellij.psi.TokenType

class VolkiAnnotator : Annotator {

    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        // Color highlighting for TAG_NAME tokens
        if (element.node.elementType == VolkiTokenTypes.TAG_NAME) {
            val tagText = element.text
            val key = when {
                tagText[0].isUpperCase() -> VolkiSyntaxHighlighter.CUSTOM_COMPONENT_NAME
                VolkiElementRegistry.isBuiltinTag(tagText) -> VolkiSyntaxHighlighter.HTML_TAG_NAME
                else -> null
            }
            if (key != null) {
                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .textAttributes(key)
                    .range(element)
                    .create()
            }
        }

        // Per-element import highlighting
        annotateSemanticToken(element, holder)
        annotateImportToken(element, holder)
        annotateConditionalOperator(element, holder)

        // File-level error checking — run once on the file element
        if (element is VolkiFile) {
            val first = element.firstChild ?: return
            checkTagErrors(first, holder)
            checkBraceErrors(first, holder)
            checkConditionalExpressionErrors(first, holder)
            checkImportErrors(element, holder)
            checkComponentErrors(first, holder, element.project)
            checkUnknownHtmlTags(first, holder)
            checkClassAttributeErrors(first, holder)
        }
    }

    // --- Import token highlighting ---

    private fun annotateSemanticToken(element: PsiElement, holder: AnnotationHolder) {
        if (VolkiImportResolver.findEnclosingUseKeyword(element) != null) return

        when (element.node.elementType) {
            VolkiTokenTypes.OPERATOR -> {
                if (element.text == "->") {
                    holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                        .textAttributes(VolkiSyntaxHighlighter.RETURN_ARROW)
                        .range(element)
                        .create()
                }
            }

            VolkiTokenTypes.TYPE -> {
                val key = if (isReturnTypeToken(element)) {
                    VolkiSyntaxHighlighter.RETURN_TYPE
                } else {
                    VolkiSyntaxHighlighter.TYPE_REFERENCE
                }
                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .textAttributes(key)
                    .range(element)
                    .create()
            }

            VolkiTokenTypes.IDENTIFIER -> {
                val key = when {
                    isFunctionDeclarationName(element) -> VolkiSyntaxHighlighter.FUNCTION_DECL
                    isMethodCallName(element) -> VolkiSyntaxHighlighter.METHOD_CALL
                    isFunctionCallName(element) -> VolkiSyntaxHighlighter.FUNCTION_CALL
                    isReturnTypeToken(element) -> VolkiSyntaxHighlighter.RETURN_TYPE
                    isTypeReferenceToken(element) -> VolkiSyntaxHighlighter.TYPE_REFERENCE
                    isVariableDeclarationName(element) -> VolkiSyntaxHighlighter.VARIABLE
                    isVariableReference(element) -> VolkiSyntaxHighlighter.VARIABLE
                    else -> null
                } ?: return

                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .textAttributes(key)
                    .range(element)
                    .create()
            }
        }
    }

    private fun annotateImportToken(element: PsiElement, holder: AnnotationHolder) {
        val type = element.node.elementType
        val text = element.text

        // Only handle tokens that can appear in use statements
        if (type != VolkiTokenTypes.IDENTIFIER && type != VolkiTokenTypes.KEYWORD &&
            type != VolkiTokenTypes.TYPE && type != VolkiTokenTypes.OPERATOR) return

        // Check if this token is inside a use statement
        VolkiImportResolver.findEnclosingUseKeyword(element) ?: return

        when {
            // Glob wildcard: * after ::
            type == VolkiTokenTypes.OPERATOR && text == "*" && VolkiImportResolver.isPrecededByPathSeparator(element) -> {
                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .textAttributes(VolkiSyntaxHighlighter.USE_GLOB)
                    .range(element)
                    .create()
            }

            // Path separator :: — skip, let it use default operator color
            type == VolkiTokenTypes.OPERATOR && text == "::" -> { /* no-op */ }

            // Other operators (; , etc.) — skip
            type == VolkiTokenTypes.OPERATOR -> { /* no-op */ }

            // crate/self/super keywords in use paths always get USE_PATH
            type == VolkiTokenTypes.KEYWORD && text in USE_PATH_KEYWORDS -> {
                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .textAttributes(VolkiSyntaxHighlighter.USE_PATH)
                    .range(element)
                    .create()
            }

            // `use` keyword itself — skip, let it keep normal keyword highlighting
            type == VolkiTokenTypes.KEYWORD && text == "use" -> { /* no-op */ }

            // Inside braces — it's an imported symbol
            VolkiImportResolver.isInsideUseBraces(element) -> {
                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .textAttributes(VolkiSyntaxHighlighter.USE_SYMBOL)
                    .range(element)
                    .create()
            }

            // Followed by :: — it's a path segment
            VolkiImportResolver.isFollowedByPathSeparator(element) -> {
                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .textAttributes(VolkiSyntaxHighlighter.USE_PATH)
                    .range(element)
                    .create()
            }

            // Terminal symbol (not followed by ::, not in braces, preceded by ::)
            VolkiImportResolver.isPrecededByPathSeparator(element) -> {
                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .textAttributes(VolkiSyntaxHighlighter.USE_SYMBOL)
                    .range(element)
                    .create()
            }
        }
    }

    private fun annotateConditionalOperator(element: PsiElement, holder: AnnotationHolder) {
        if (element.node.elementType != VolkiTokenTypes.OPERATOR) return
        val text = element.text
        val key = when (text) {
            "?" -> VolkiSyntaxHighlighter.TERNARY_OPERATOR
            ":" -> VolkiSyntaxHighlighter.TERNARY_OPERATOR
            "&&" -> VolkiSyntaxHighlighter.CONDITIONAL_AND
            else -> null
        } ?: return

        holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
            .textAttributes(key)
            .range(element)
            .create()
    }

    // --- Import error checking ---

    private fun checkImportErrors(file: VolkiFile, holder: AnnotationHolder) {
        val project = file.project
        val useStatements = VolkiImportResolver.parseUseStatements(file)

        for (stmt in useStatements) {
            if (stmt.pathSegments.isEmpty()) continue

            // Try to resolve the module path
            val moduleFile = VolkiImportResolver.resolveModulePath(project, stmt.pathSegments, file)

            if (moduleFile == null) {
                // Find the last resolvable segment to error on the right one
                var lastResolvable = -1
                for (i in stmt.pathSegments.indices) {
                    val partial = VolkiImportResolver.resolvePathSegment(
                        project, stmt.pathSegments, i, file
                    )
                    if (partial != null) {
                        lastResolvable = i
                    } else {
                        break
                    }
                }
                val errorIdx = lastResolvable + 1
                if (errorIdx < stmt.pathSegments.size) {
                    val errorSeg = stmt.pathSegments[errorIdx]
                    holder.newAnnotation(
                        HighlightSeverity.ERROR,
                        "Unresolved module: `${errorSeg.text}`"
                    )
                        .range(errorSeg.element)
                        .create()
                }
                continue
            }

            // Module resolved — check individual symbols for brace imports
            if (stmt.symbols.any { it.isGlob }) continue // glob imports don't check symbols

            for (sym in stmt.symbols) {
                if (sym.isSelf) continue
                if (sym.element == null) continue

                val found = VolkiImportResolver.findSymbolInFile(project, moduleFile, sym.text)
                if (found == null) {
                    // Also check exported symbols (re-exports)
                    val exports = VolkiImportResolver.findExportedSymbols(project, moduleFile)
                    if (exports.none { it.name == sym.text }) {
                        holder.newAnnotation(
                            HighlightSeverity.ERROR,
                            "Unresolved import: `${sym.text}` not found in module"
                        )
                            .range(sym.element)
                            .create()
                    }
                }
            }
        }
    }

    // --- Component + expression checks ---

    private fun checkComponentErrors(firstElement: PsiElement, holder: AnnotationHolder, project: com.intellij.openapi.project.Project) {
        var current: PsiElement? = firstElement
        while (current != null) {
            if (current.node.elementType == VolkiTokenTypes.TAG_NAME) {
                val tagName = current.text
                if (tagName.isNotEmpty() && tagName[0].isUpperCase() &&
                    !VolkiDeclarationResolver.isSpecialCompilerTag(tagName) &&
                    isOpeningTagName(current)
                ) {
                    // Check declaration files before flagging as unresolved
                    val declResult = VolkiDeclarationResolver.findSymbolInDeclarations(project, tagName)
                    if (declResult != null) {
                        current = current.nextSibling
                        continue
                    }

                    val returnType = resolveFunctionReturnType(current, tagName)
                    if (returnType == null) {
                        holder.newAnnotation(
                            HighlightSeverity.ERROR,
                            "Unresolved component `${tagName}`"
                        ).range(current).create()
                    } else if (returnType != "Fragment") {
                        holder.newAnnotation(
                            HighlightSeverity.ERROR,
                            "Component `${tagName}` must return `Fragment` (found `${returnType}`)"
                        ).range(current).create()
                    }
                }
            }
            current = current.nextSibling
        }
    }

    private fun checkUnknownHtmlTags(firstElement: PsiElement, holder: AnnotationHolder) {
        var current: PsiElement? = firstElement
        while (current != null) {
            if (current.node.elementType == VolkiTokenTypes.TAG_NAME) {
                val tagName = current.text
                // Only check lowercase tags (uppercase are components, handled by checkComponentErrors)
                if (tagName.isNotEmpty() && tagName[0].isLowerCase() && isOpeningTagName(current)) {
                    if (!VolkiElementRegistry.isBuiltinTag(tagName)) {
                        holder.newAnnotation(
                            HighlightSeverity.ERROR,
                            "Unknown HTML element `<$tagName>`"
                        ).range(current).create()
                    }
                }
            }
            current = current.nextSibling
        }
    }

    private fun checkClassAttributeErrors(firstElement: PsiElement, holder: AnnotationHolder) {
        var current: PsiElement? = firstElement
        while (current != null) {
            if (current.node.elementType == VolkiTokenTypes.STRING &&
                VolkiStyleClassContext.isClassAttributeValue(current)
            ) {
                val spans = VolkiStyleClassContext.extractClassNames(current)
                for (span in spans) {
                    val parsed = VolkiStyleVariants.parse(span.text)

                    // Validate variant prefixes
                    var invalidVariant = false
                    for (v in parsed.variants) {
                        if (!VolkiStyleVariants.isValidVariant(v)) {
                            holder.newAnnotation(
                                HighlightSeverity.WARNING,
                                "Unknown variant prefix `$v`"
                            ).range(com.intellij.openapi.util.TextRange(span.startOffset, span.endOffset)).create()
                            invalidVariant = true
                            break
                        }
                    }
                    if (invalidVariant) continue

                    // Skip custom: prefixed and bare arbitrary [...] classes
                    if (parsed.isCustom) continue
                    if (parsed.utility.startsWith("[") && parsed.utility.endsWith("]")) continue

                    val resolved = VolkiStyleResolver.resolve(parsed.utility)
                    if (resolved == null) {
                        // Unknown utility class
                        holder.newAnnotation(
                            HighlightSeverity.WARNING,
                            "Unknown volkistyle class `${parsed.utility}`"
                        ).range(com.intellij.openapi.util.TextRange(span.startOffset, span.endOffset)).create()
                    } else {
                        // Check if it's a color utility — add gutter icon
                        val colorName = VolkiStyleResolver.extractColorName(parsed.utility)
                        if (colorName != null) {
                            val awtColor = VolkiStylePalette.colorToAwtColor(colorName)
                            if (awtColor != null) {
                                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                                    .range(com.intellij.openapi.util.TextRange(span.startOffset, span.endOffset))
                                    .gutterIconRenderer(VolkiStyleGutterIcon(awtColor, span.text))
                                    .create()
                            }
                        }
                    }
                }
            }
            current = current.nextSibling
        }
    }

    private fun checkBraceErrors(firstElement: PsiElement, holder: AnnotationHolder) {
        val stack = mutableListOf<PsiElement>()
        var current: PsiElement? = firstElement

        while (current != null) {
            when (current.node.elementType) {
                VolkiTokenTypes.BRACE_OPEN -> stack.add(current)
                VolkiTokenTypes.BRACE_CLOSE -> {
                    if (stack.isEmpty()) {
                        holder.newAnnotation(HighlightSeverity.ERROR, "Unmatched `}`")
                            .range(current)
                            .create()
                    } else {
                        stack.removeAt(stack.lastIndex)
                    }
                }
            }
            current = current.nextSibling
        }

        for (open in stack) {
            holder.newAnnotation(HighlightSeverity.ERROR, "Missing closing `}`")
                .range(open)
                .create()
        }
    }

    private fun checkConditionalExpressionErrors(firstElement: PsiElement, holder: AnnotationHolder) {
        var current: PsiElement? = firstElement

        while (current != null) {
            if (current.node.elementType == VolkiTokenTypes.BRACE_OPEN) {
                val end = findMatchingBraceClose(current)
                if (end != null) {
                    validateConditionalBlock(current, end, holder)
                    current = end
                }
            }
            current = current?.nextSibling
        }
    }

    private fun validateConditionalBlock(
        open: PsiElement,
        close: PsiElement,
        holder: AnnotationHolder
    ) {
        val tokens = mutableListOf<PsiElement>()
        var cur = open.nextSibling
        while (cur != null && cur != close) {
            if (cur.node.elementType != TokenType.WHITE_SPACE) {
                tokens.add(cur)
            }
            cur = cur.nextSibling
        }
        if (tokens.isEmpty()) return

        val qIdx = tokens.indexOfFirst { it.node.elementType == VolkiTokenTypes.OPERATOR && it.text == "?" }
        if (qIdx >= 0) {
            val cIdx = tokens.indexOfFirst { it.node.elementType == VolkiTokenTypes.OPERATOR && it.text == ":" }
            if (cIdx < 0) {
                holder.newAnnotation(
                    HighlightSeverity.ERROR,
                    "Invalid ternary expression: expected `:`"
                ).range(tokens[qIdx]).create()
            } else {
                val hasCond = qIdx > 0
                val hasTrue = cIdx > qIdx + 1
                val hasFalse = cIdx < tokens.lastIndex
                if (!hasCond || !hasTrue || !hasFalse) {
                    holder.newAnnotation(
                        HighlightSeverity.ERROR,
                        "Invalid ternary expression: expected `cond ? a : b`"
                    ).range(tokens[qIdx]).create()
                }
            }
        }

        val andIdx = tokens.indexOfFirst { it.node.elementType == VolkiTokenTypes.OPERATOR && it.text == "&&" }
        if (andIdx >= 0) {
            val hasLeft = andIdx > 0
            val hasRight = andIdx < tokens.lastIndex
            if (!hasLeft || !hasRight) {
                holder.newAnnotation(
                    HighlightSeverity.ERROR,
                    "Invalid conditional expression: expected `cond && expr`"
                ).range(tokens[andIdx]).create()
            }
        }
    }

    private fun findMatchingBraceClose(open: PsiElement): PsiElement? {
        var depth = 1
        var cur = open.nextSibling
        while (cur != null) {
            when (cur.node.elementType) {
                VolkiTokenTypes.BRACE_OPEN -> depth++
                VolkiTokenTypes.BRACE_CLOSE -> {
                    depth--
                    if (depth == 0) return cur
                }
            }
            cur = cur.nextSibling
        }
        return null
    }

    private fun isOpeningTagName(tagNameElement: PsiElement): Boolean {
        val prev = findPrevNonWhitespace(tagNameElement) ?: return false
        return prev.node.elementType == VolkiTokenTypes.TAG_BRACKET && prev.text == "<"
    }

    private fun isFunctionDeclarationName(element: PsiElement): Boolean {
        val prev = findPrevNonWhitespace(element) ?: return false
        return prev.node.elementType == VolkiTokenTypes.KEYWORD && prev.text == "fn"
    }

    private fun isVariableDeclarationName(element: PsiElement): Boolean {
        val prev = findPrevNonWhitespace(element) ?: return false
        if (prev.node.elementType == VolkiTokenTypes.KEYWORD && prev.text in setOf("let", "const")) {
            return true
        }
        if (prev.node.elementType == VolkiTokenTypes.KEYWORD && prev.text == "mut") {
            val prev2 = findPrevNonWhitespace(prev) ?: return false
            if (prev2.node.elementType == VolkiTokenTypes.KEYWORD && prev2.text == "let") {
                return true
            }
        }
        if (prev.node.elementType == VolkiTokenTypes.KEYWORD && prev.text == "for") {
            return true
        }
        return false
    }

    private fun isMethodCallName(element: PsiElement): Boolean {
        val prev = findPrevNonWhitespace(element) ?: return false
        val next = findNextNonWhitespace(element) ?: return false
        return prev.node.elementType == VolkiTokenTypes.OPERATOR &&
            prev.text == "." &&
            next.node.elementType == VolkiTokenTypes.PAREN_OPEN
    }

    private fun isFunctionCallName(element: PsiElement): Boolean {
        if (isFunctionDeclarationName(element) || isMethodCallName(element)) return false
        val next = findNextNonWhitespace(element) ?: return false
        return next.node.elementType == VolkiTokenTypes.PAREN_OPEN
    }

    private fun isReturnTypeToken(element: PsiElement): Boolean {
        var prev = findPrevNonWhitespace(element)
        while (prev != null) {
            if (prev.node.elementType == VolkiTokenTypes.OPERATOR && prev.text == "->") return true
            if (prev.node.elementType == VolkiTokenTypes.BRACE_OPEN || prev.node.elementType == VolkiTokenTypes.OPERATOR && prev.text == ";") {
                return false
            }
            prev = findPrevNonWhitespace(prev)
        }
        return false
    }

    private fun isTypeReferenceToken(element: PsiElement): Boolean {
        val prev = findPrevNonWhitespace(element) ?: return false
        if (prev.node.elementType == VolkiTokenTypes.OPERATOR && prev.text in setOf(":", "::")) {
            return true
        }
        if (prev.node.elementType == VolkiTokenTypes.KEYWORD && prev.text in setOf("as", "impl", "dyn")) {
            return true
        }
        return false
    }

    private fun isVariableReference(element: PsiElement): Boolean {
        if (isFunctionDeclarationName(element) || isMethodCallName(element) || isFunctionCallName(element)) {
            return false
        }
        if (isTypeReferenceToken(element) || isReturnTypeToken(element)) return false
        val prev = findPrevNonWhitespace(element)
        if (prev != null && prev.node.elementType == VolkiTokenTypes.OPERATOR && prev.text == "::") {
            return false
        }
        return true
    }

    private fun findPrevNonWhitespace(element: PsiElement): PsiElement? {
        var current = element.prevSibling
        while (current != null && current.node.elementType == TokenType.WHITE_SPACE) {
            current = current.prevSibling
        }
        return current
    }

    private fun resolveFunctionReturnType(element: PsiElement, name: String): String? {
        val file = element.containingFile ?: return null
        val local = findReturnTypeInText(file.text, name)
        if (local != null) return local

        val resolved = VolkiImportResolver.findImportForIdentifier(file, name)
        if (resolved != null) {
            return extractReturnTypeFromSignature(resolved.signature)
        }
        return null
    }

    private fun findReturnTypeInText(text: String, fnName: String): String? {
        val escaped = Regex.escape(fnName)
        val regex = Regex("""\bfn\s+$escaped\s*(<[^>]*>)?\s*\([^)]*\)\s*->\s*([A-Za-z_][A-Za-z0-9_]*)""")
        val match = regex.find(text) ?: return null
        return match.groupValues.getOrNull(2)
    }

    private fun extractReturnTypeFromSignature(signature: String): String? {
        val idx = signature.indexOf("->")
        if (idx < 0) return null
        val tail = signature.substring(idx + 2).trim()
        if (tail.isEmpty()) return null
        val end = tail.indexOfAny(charArrayOf(' ', '{', ';', '\n', '\r', ',')).let { if (it < 0) tail.length else it }
        return tail.substring(0, end)
    }

    // --- Tag error checking (existing) ---

    private data class TagEntry(val name: String, val element: PsiElement)

    private fun checkTagErrors(firstElement: PsiElement, holder: AnnotationHolder) {
        val stack = mutableListOf<TagEntry>()
        var current: PsiElement? = firstElement

        while (current != null) {
            if (current.node.elementType == VolkiTokenTypes.TAG_BRACKET) {
                val bracketText = current.text

                when {
                    // Opening tag: <
                    bracketText == "<" -> {
                        val tagNameElement = findNextNonWhitespace(current)
                        if (tagNameElement != null && tagNameElement.node.elementType == VolkiTokenTypes.TAG_NAME) {
                            val tagName = tagNameElement.text
                            if (!isTagSelfClosing(current) && !isVoidElement(tagName)) {
                                stack.add(TagEntry(tagName, tagNameElement))
                            }
                        }
                    }

                    // Closing tag: </
                    bracketText == "</" -> {
                        val tagNameElement = findNextNonWhitespace(current)
                        if (tagNameElement != null && tagNameElement.node.elementType == VolkiTokenTypes.TAG_NAME) {
                            val closingName = tagNameElement.text

                            if (stack.isNotEmpty() && stack.last().name == closingName) {
                                // Perfect match — pop
                                stack.removeAt(stack.lastIndex)
                            } else {
                                // Check if it exists deeper in the stack
                                val deepIndex = stack.indexOfLast { it.name == closingName }
                                if (deepIndex >= 0) {
                                    // Mark everything above as unclosed
                                    for (i in stack.lastIndex downTo deepIndex + 1) {
                                        holder.newAnnotation(HighlightSeverity.ERROR, "Unclosed tag `<${stack[i].name}>`")
                                            .range(stack[i].element)
                                            .create()
                                    }
                                    // Remove from deepIndex to end (including the match)
                                    while (stack.size > deepIndex) {
                                        stack.removeAt(stack.lastIndex)
                                    }
                                } else {
                                    // Not in stack at all — mismatched closing tag
                                    holder.newAnnotation(HighlightSeverity.ERROR, "Mismatched closing tag `</${closingName}>`")
                                        .range(tagNameElement)
                                        .create()
                                }
                            }
                        }
                    }
                }
            }

            current = current.nextSibling
        }

        // Remaining stack entries are unclosed
        for (entry in stack) {
            holder.newAnnotation(HighlightSeverity.ERROR, "Unclosed tag `<${entry.name}>`")
                .range(entry.element)
                .create()
        }
    }

    private fun isTagSelfClosing(openBracket: PsiElement): Boolean {
        var current: PsiElement? = openBracket.nextSibling
        var braceDepth = 0
        var seenBrace = false

        while (current != null) {
            val type = current.node.elementType
            val text = current.text

            if (type == VolkiTokenTypes.BRACE_OPEN) {
                braceDepth++
                seenBrace = true
            } else if (type == VolkiTokenTypes.BRACE_CLOSE) {
                braceDepth--
            }

            if (braceDepth == 0) {
                // TAG_BRACKET /> means self-closing (normal case, lexer in tag state)
                if (type == VolkiTokenTypes.TAG_BRACKET && text == "/>") return true
                // TAG_BRACKET > means not self-closing
                if (type == VolkiTokenTypes.TAG_BRACKET && text == ">") return false

                // Post-brace fallback: after a brace expression the lexer drops to RUST state,
                // so /> becomes two OPERATOR tokens and > becomes one OPERATOR token.
                if (seenBrace && type == VolkiTokenTypes.OPERATOR && text == "/") {
                    val next = findNextNonWhitespace(current)
                    if (next != null && next.node.elementType == VolkiTokenTypes.OPERATOR && next.text == ">") {
                        return true
                    }
                }
                if (seenBrace && type == VolkiTokenTypes.OPERATOR && text == ">") {
                    return false
                }

                // New tag starting means previous tag ended (malformed)
                if (type == VolkiTokenTypes.TAG_BRACKET && (text == "<" || text == "</")) {
                    return false
                }
            }

            current = current.nextSibling
        }
        return false
    }

    private fun isVoidElement(tagName: String): Boolean {
        return VolkiElementRegistry.getElement(tagName)?.isVoid == true
    }

    private fun findNextNonWhitespace(element: PsiElement): PsiElement? {
        var current = element.nextSibling
        while (current != null && current.node.elementType == TokenType.WHITE_SPACE) {
            current = current.nextSibling
        }
        return current
    }

    companion object {
        private val USE_PATH_KEYWORDS = setOf("crate", "self", "super")
    }
}
