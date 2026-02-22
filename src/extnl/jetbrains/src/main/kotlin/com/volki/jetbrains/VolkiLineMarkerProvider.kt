package com.volki.jetbrains

import com.intellij.codeInsight.daemon.LineMarkerInfo
import com.intellij.codeInsight.daemon.LineMarkerProvider
import com.intellij.openapi.editor.markup.GutterIconRenderer
import com.intellij.psi.PsiElement
import com.intellij.psi.TokenType
import com.intellij.icons.AllIcons

class VolkiLineMarkerProvider : LineMarkerProvider {

    override fun getLineMarkerInfo(element: PsiElement): LineMarkerInfo<*>? {
        val type = element.node?.elementType ?: return null

        // Only look at KEYWORD tokens to avoid processing every element
        if (type != VolkiTokenTypes.KEYWORD) return null

        return when (element.text) {
            "fn" -> handleFunction(element)
            else -> null
        }
    }

    private fun handleFunction(fnKeyword: PsiElement): LineMarkerInfo<*>? {
        // Check if preceded by "pub"
        var prev = fnKeyword.prevSibling
        while (prev != null && prev.node?.elementType == TokenType.WHITE_SPACE) prev = prev.prevSibling
        val isPub = prev?.node?.elementType == VolkiTokenTypes.KEYWORD && prev.text == "pub"

        // Get function name
        var next = fnKeyword.nextSibling
        while (next != null && next.node?.elementType == TokenType.WHITE_SPACE) next = next.nextSibling
        val nameEl = next ?: return null
        val name = nameEl.text

        // Check if this is a component (uppercase name)
        val isComponent = name.isNotEmpty() && name[0].isUpperCase()

        // Check if it returns Html (scan for -> Html)
        val returnsHtml = checkReturnsHtml(nameEl)

        if (isComponent && returnsHtml) {
            return LineMarkerInfo(
                fnKeyword,
                fnKeyword.textRange,
                AllIcons.Nodes.AbstractClass,
                { "RSX Component: $name" },
                null,
                GutterIconRenderer.Alignment.LEFT,
                { "RSX Component" }
            )
        }

        if (isPub && returnsHtml) {
            return LineMarkerInfo(
                fnKeyword,
                fnKeyword.textRange,
                AllIcons.Nodes.Function,
                { "Page handler: $name" },
                null,
                GutterIconRenderer.Alignment.LEFT,
                { "Page handler" }
            )
        }

        return null
    }

    private fun checkReturnsHtml(afterName: PsiElement): Boolean {
        var cur: PsiElement? = afterName.nextSibling
        while (cur != null) {
            val type = cur.node?.elementType
            // Stop at function body
            if (type == VolkiTokenTypes.BRACE_OPEN) break
            if (type == VolkiTokenTypes.TAG_BRACKET) break

            if (type == VolkiTokenTypes.OPERATOR && cur.text == "->") {
                // Next non-whitespace should be the return type
                var retType = cur.nextSibling
                while (retType != null && retType.node?.elementType == TokenType.WHITE_SPACE) {
                    retType = retType.nextSibling
                }
                if (retType != null) {
                    val retText = retType.text
                    return retText == "Html" || retText == "Fragment" || retText == "HtmlNode"
                }
            }
            cur = cur.nextSibling
        }
        return false
    }
}
