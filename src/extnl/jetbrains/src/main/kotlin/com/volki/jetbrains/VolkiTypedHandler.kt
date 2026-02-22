package com.volki.jetbrains

import com.intellij.codeInsight.editorActions.TypedHandlerDelegate
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiFile

class VolkiTypedHandler : TypedHandlerDelegate() {

    override fun charTyped(c: Char, project: Project, editor: Editor, file: PsiFile): Result {
        if (file !is VolkiFile) return Result.CONTINUE
        if (c != '>') return Result.CONTINUE

        val document = editor.document
        val offset = editor.caretModel.offset
        if (offset < 2) return Result.CONTINUE

        val text = document.charsSequence

        // Don't auto-close if this is a self-closing tag (/>)
        if (text[offset - 2] == '/') return Result.CONTINUE

        // Don't auto-close if this is a closing tag (</...>)
        // Don't auto-close operators like ->, =>, >=
        if (offset >= 2) {
            val prev = text[offset - 2]
            if (prev == '-' || prev == '=' || prev == '!') return Result.CONTINUE
        }

        // Find the opening < and extract tag name
        val tagName = findOpeningTagName(text, offset - 1) ?: return Result.CONTINUE

        // Don't auto-close void elements
        if (VolkiElementRegistry.getElement(tagName)?.isVoid == true) return Result.CONTINUE

        // Insert closing tag
        document.insertString(offset, "</$tagName>")

        return Result.STOP
    }

    private fun findOpeningTagName(text: CharSequence, closingBracketPos: Int): String? {
        // Walk backwards from > to find < and extract tag name
        var pos = closingBracketPos - 1
        // Skip attributes: walk back past strings, identifiers, =, whitespace
        var braceDepth = 0
        while (pos >= 0) {
            val ch = text[pos]
            if (ch == '}') braceDepth++
            else if (ch == '{') {
                if (braceDepth > 0) braceDepth--
                else break
            }
            if (braceDepth > 0) { pos--; continue }

            if (ch == '<') {
                // Check it's not </
                if (pos + 1 < text.length && text[pos + 1] == '/') return null
                // Extract tag name after <
                val nameStart = pos + 1
                var nameEnd = nameStart
                while (nameEnd < text.length && (text[nameEnd].isLetterOrDigit() || text[nameEnd] == '_')) {
                    nameEnd++
                }
                if (nameEnd == nameStart) return null
                val name = text.subSequence(nameStart, nameEnd).toString()
                // Verify it's a known tag or a component (uppercase)
                if (VolkiElementRegistry.isBuiltinTag(name) || (name.isNotEmpty() && name[0].isUpperCase())) {
                    return name
                }
                return null
            }
            pos--
        }
        return null
    }
}
