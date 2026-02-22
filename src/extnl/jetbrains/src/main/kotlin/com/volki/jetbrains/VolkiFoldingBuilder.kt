package com.volki.jetbrains

import com.intellij.lang.ASTNode
import com.intellij.lang.folding.FoldingBuilderEx
import com.intellij.lang.folding.FoldingDescriptor
import com.intellij.openapi.editor.Document
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement
import com.intellij.psi.TokenType

class VolkiFoldingBuilder : FoldingBuilderEx() {

    override fun buildFoldRegions(root: PsiElement, document: Document, quick: Boolean): Array<FoldingDescriptor> {
        val descriptors = mutableListOf<FoldingDescriptor>()

        var child: PsiElement? = root.firstChild
        while (child != null) {
            val type = child.node?.elementType

            when (type) {
                // Fold block comments /* ... */
                VolkiTokenTypes.BLOCK_COMMENT -> {
                    if (child.textLength > 4 && child.text.contains('\n')) {
                        descriptors.add(FoldingDescriptor(child.node, child.textRange))
                    }
                }

                // Fold brace blocks { ... }
                VolkiTokenTypes.BRACE_OPEN -> {
                    val closingBrace = findMatchingBrace(child)
                    if (closingBrace != null) {
                        val startLine = document.getLineNumber(child.textRange.startOffset)
                        val endLine = document.getLineNumber(closingBrace.textRange.endOffset)
                        if (endLine > startLine) {
                            val range = TextRange(child.textRange.startOffset, closingBrace.textRange.endOffset)
                            descriptors.add(FoldingDescriptor(child.node, range))
                        }
                    }
                }

                // Fold RSX tag blocks <tag>...</tag>
                VolkiTokenTypes.TAG_BRACKET -> {
                    if (child.text == "<") {
                        val tagNameEl = findNextNonWhitespace(child)
                        if (tagNameEl?.node?.elementType == VolkiTokenTypes.TAG_NAME) {
                            val tagName = tagNameEl.text
                            val isVoid = VolkiElementRegistry.getElement(tagName)?.isVoid == true
                            if (!isVoid && !isSelfClosing(child)) {
                                val closingTag = findClosingTag(child, tagName, document)
                                if (closingTag != null) {
                                    val startLine = document.getLineNumber(child.textRange.startOffset)
                                    val endLine = document.getLineNumber(closingTag.endOffset)
                                    if (endLine > startLine) {
                                        val range = TextRange(child.textRange.startOffset, closingTag.endOffset)
                                        descriptors.add(FoldingDescriptor(child.node, range))
                                    }
                                }
                            }
                        }
                    }
                }

                // Fold doc comment blocks (consecutive //! lines)
                VolkiTokenTypes.DOC_COMMENT -> {
                    if (!isPrecededByDocComment(child)) {
                        val lastDoc = findLastConsecutiveDocComment(child)
                        if (lastDoc != child) {
                            val range = TextRange(child.textRange.startOffset, lastDoc.textRange.endOffset)
                            descriptors.add(FoldingDescriptor(child.node, range))
                        }
                    }
                }
            }

            child = child.nextSibling
        }

        return descriptors.toTypedArray()
    }

    override fun getPlaceholderText(node: ASTNode): String {
        return when (node.elementType) {
            VolkiTokenTypes.BLOCK_COMMENT -> "/* ... */"
            VolkiTokenTypes.DOC_COMMENT -> "//! ..."
            VolkiTokenTypes.BRACE_OPEN -> "{...}"
            VolkiTokenTypes.TAG_BRACKET -> {
                val tagName = findNextNonWhitespaceNode(node)
                if (tagName?.elementType == VolkiTokenTypes.TAG_NAME) {
                    "<${tagName.text}>...</${tagName.text}>"
                } else {
                    "<...>"
                }
            }
            else -> "..."
        }
    }

    override fun isCollapsedByDefault(node: ASTNode): Boolean = false

    private fun findMatchingBrace(openBrace: PsiElement): PsiElement? {
        var depth = 1
        var cur: PsiElement? = openBrace.nextSibling
        while (cur != null) {
            when (cur.node?.elementType) {
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

    private fun isSelfClosing(openBracket: PsiElement): Boolean {
        var cur: PsiElement? = openBracket.nextSibling
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

    private fun findClosingTag(openBracket: PsiElement, tagName: String, document: Document): TextRange? {
        var depth = 1
        var cur: PsiElement? = openBracket.nextSibling

        while (cur != null) {
            if (cur.node?.elementType == VolkiTokenTypes.TAG_BRACKET) {
                when (cur.text) {
                    "<" -> {
                        val nameEl = findNextNonWhitespace(cur)
                        if (nameEl?.node?.elementType == VolkiTokenTypes.TAG_NAME && nameEl.text == tagName) {
                            if (!isSelfClosing(cur)) depth++
                        }
                    }
                    "</" -> {
                        val nameEl = findNextNonWhitespace(cur)
                        if (nameEl?.node?.elementType == VolkiTokenTypes.TAG_NAME && nameEl.text == tagName) {
                            depth--
                            if (depth == 0) {
                                // Find the closing >
                                var end: PsiElement? = nameEl.nextSibling
                                while (end != null) {
                                    if (end.node?.elementType == VolkiTokenTypes.TAG_BRACKET && end.text == ">") {
                                        return TextRange(cur.textRange.startOffset, end.textRange.endOffset)
                                    }
                                    end = end.nextSibling
                                }
                                return TextRange(cur.textRange.startOffset, nameEl.textRange.endOffset)
                            }
                        }
                    }
                }
            }
            cur = cur.nextSibling
        }
        return null
    }

    private fun isPrecededByDocComment(element: PsiElement): Boolean {
        var prev = element.prevSibling
        while (prev != null && prev.node?.elementType == TokenType.WHITE_SPACE) {
            if (prev.text.contains('\n') && prev.text.count { it == '\n' } > 1) return false
            prev = prev.prevSibling
        }
        return prev?.node?.elementType == VolkiTokenTypes.DOC_COMMENT
    }

    private fun findLastConsecutiveDocComment(element: PsiElement): PsiElement {
        var last = element
        var next = element.nextSibling
        while (next != null) {
            if (next.node?.elementType == TokenType.WHITE_SPACE) {
                if (next.text.count { it == '\n' } > 1) break
                next = next.nextSibling
                continue
            }
            if (next.node?.elementType == VolkiTokenTypes.DOC_COMMENT) {
                last = next
                next = next.nextSibling
            } else {
                break
            }
        }
        return last
    }

    private fun findNextNonWhitespace(element: PsiElement): PsiElement? {
        var cur = element.nextSibling
        while (cur != null && cur.node?.elementType == TokenType.WHITE_SPACE) cur = cur.nextSibling
        return cur
    }

    private fun findNextNonWhitespaceNode(node: ASTNode): ASTNode? {
        var cur = node.treeNext
        while (cur != null && cur.elementType == TokenType.WHITE_SPACE) cur = cur.treeNext
        return cur
    }
}
