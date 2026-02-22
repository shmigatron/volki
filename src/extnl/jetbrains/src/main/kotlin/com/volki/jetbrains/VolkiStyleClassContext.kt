package com.volki.jetbrains

import com.intellij.psi.PsiElement
import com.intellij.psi.TokenType

object VolkiStyleClassContext {

    data class ClassNameSpan(val text: String, val startOffset: Int, val endOffset: Int)

    fun isClassAttributeValue(element: PsiElement): Boolean {
        if (element.node?.elementType != VolkiTokenTypes.STRING) return false
        // Walk backwards: STRING <- OPERATOR(=) <- ATTRIBUTE("class")
        var prev = element.prevSibling
        while (prev != null && prev.node?.elementType == TokenType.WHITE_SPACE) {
            prev = prev.prevSibling
        }
        if (prev == null || prev.node?.elementType != VolkiTokenTypes.OPERATOR || prev.text != "=") return false
        prev = prev.prevSibling
        while (prev != null && prev.node?.elementType == TokenType.WHITE_SPACE) {
            prev = prev.prevSibling
        }
        if (prev == null || prev.node?.elementType != VolkiTokenTypes.ATTRIBUTE) return false
        return prev.text == "class"
    }

    fun extractClassNames(stringToken: PsiElement): List<ClassNameSpan> {
        val text = stringToken.text
        val baseOffset = stringToken.textRange.startOffset
        // Strip quotes
        val inner: String
        val innerStart: Int
        if (text.length >= 2 && (text.startsWith('"') || text.startsWith('\''))) {
            inner = text.substring(1, text.length - 1)
            innerStart = baseOffset + 1
        } else {
            inner = text
            innerStart = baseOffset
        }

        val result = mutableListOf<ClassNameSpan>()
        var i = 0
        while (i < inner.length) {
            // Skip whitespace
            if (inner[i].isWhitespace()) {
                i++
                continue
            }
            val start = i
            while (i < inner.length && !inner[i].isWhitespace()) {
                i++
            }
            val className = inner.substring(start, i)
            result.add(ClassNameSpan(className, innerStart + start, innerStart + i))
        }
        return result
    }

    fun getClassAtOffset(stringToken: PsiElement, offset: Int): ClassNameSpan? {
        val spans = extractClassNames(stringToken)
        return spans.find { offset >= it.startOffset && offset <= it.endOffset }
    }
}
