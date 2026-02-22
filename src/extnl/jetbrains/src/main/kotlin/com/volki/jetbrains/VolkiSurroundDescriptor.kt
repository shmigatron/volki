package com.volki.jetbrains

import com.intellij.lang.surroundWith.SurroundDescriptor
import com.intellij.lang.surroundWith.Surrounder
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile

class VolkiSurroundDescriptor : SurroundDescriptor {

    override fun getElementsToSurround(file: PsiFile, startOffset: Int, endOffset: Int): Array<PsiElement> {
        if (file !is VolkiFile) return PsiElement.EMPTY_ARRAY
        if (startOffset >= endOffset) return PsiElement.EMPTY_ARRAY

        // Collect all elements in the selection range
        val elements = mutableListOf<PsiElement>()
        var child: PsiElement? = file.firstChild
        while (child != null) {
            val range = child.textRange
            if (range.startOffset >= startOffset && range.endOffset <= endOffset) {
                elements.add(child)
            } else if (range.startOffset >= endOffset) {
                break
            }
            child = child.nextSibling
        }

        return if (elements.isNotEmpty()) elements.toTypedArray() else PsiElement.EMPTY_ARRAY
    }

    override fun getSurrounders(): Array<Surrounder> = arrayOf(
        VolkiTagSurrounder("div"),
        VolkiTagSurrounder("span"),
        VolkiTagSurrounder("section"),
        VolkiTagSurrounder("article"),
        VolkiTagSurrounder("header"),
        VolkiTagSurrounder("footer"),
        VolkiTagSurrounder("nav"),
        VolkiTagSurrounder("main"),
        VolkiTagSurrounder("p"),
        VolkiTagSurrounder("ul"),
        VolkiTagSurrounder("li"),
        VolkiTagSurrounder("form"),
    )

    override fun isExclusive(): Boolean = false
}

class VolkiTagSurrounder(private val tagName: String) : Surrounder {

    override fun getTemplateDescription(): String = "<$tagName>"

    override fun isApplicable(elements: Array<out PsiElement>): Boolean = elements.isNotEmpty()

    override fun surroundElements(project: Project, editor: Editor, elements: Array<out PsiElement>): TextRange? {
        if (elements.isEmpty()) return null

        val doc = editor.document
        val startOffset = elements.first().textRange.startOffset
        val endOffset = elements.last().textRange.endOffset

        val openTag = "<$tagName>"
        val closeTag = "</$tagName>"

        doc.insertString(endOffset, closeTag)
        doc.insertString(startOffset, openTag)

        // Place cursor right after the opening tag
        return TextRange(startOffset + openTag.length, startOffset + openTag.length)
    }
}
