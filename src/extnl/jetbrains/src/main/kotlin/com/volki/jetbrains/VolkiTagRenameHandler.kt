package com.volki.jetbrains

import com.intellij.openapi.Disposable
import com.intellij.openapi.editor.event.DocumentEvent
import com.intellij.openapi.editor.event.DocumentListener
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerListener
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.ProjectActivity
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.PsiDocumentManager
import com.intellij.psi.PsiElement
import com.intellij.psi.TokenType

class VolkiTagRenameStartup : ProjectActivity {
    override suspend fun execute(project: Project) {
        project.messageBus.connect().subscribe(
            FileEditorManagerListener.FILE_EDITOR_MANAGER,
            object : FileEditorManagerListener {
                override fun fileOpened(source: FileEditorManager, file: VirtualFile) {
                    if (file.extension != "volki") return
                    val editor = source.selectedTextEditor ?: return
                    val document = editor.document

                    val disposable = Disposer.newDisposable("VolkiTagRename:${file.name}")
                    Disposer.register(project, disposable)
                    document.addDocumentListener(object : DocumentListener {
                        private var isUpdating = false

                        override fun documentChanged(event: DocumentEvent) {
                            if (isUpdating) return

                            val doc = event.document
                            val psiFile = PsiDocumentManager.getInstance(project).getPsiFile(doc)
                            if (psiFile !is VolkiFile) return

                            // Commit doc so PSI is up to date for the NEXT change
                            // We work on the old PSI state + the text change
                            val offset = event.offset
                            val newText = event.newFragment.toString()
                            val oldText = event.oldFragment.toString()

                            // Only handle single-character edits (typing)
                            if (newText.length > 3 || oldText.length > 3) return

                            // Find if the edit is inside a tag name
                            PsiDocumentManager.getInstance(project).commitDocument(doc)
                            val updatedPsi = PsiDocumentManager.getInstance(project).getPsiFile(doc)
                                ?: return

                            val elementAtOffset = updatedPsi.findElementAt(offset) ?: return
                            if (elementAtOffset.node?.elementType != VolkiTokenTypes.TAG_NAME) return

                            val editedTagName = elementAtOffset.text
                            val isInClosingTag = isClosingTagName(elementAtOffset)

                            // Find the matching partner
                            val partner = if (isInClosingTag) {
                                findMatchingOpenTag(elementAtOffset)
                            } else {
                                findMatchingCloseTag(elementAtOffset)
                            }

                            if (partner == null) return

                            val partnerText = partner.text
                            if (partnerText == editedTagName) return // already in sync

                            isUpdating = true
                            try {
                                val partnerRange = partner.textRange
                                doc.replaceString(partnerRange.startOffset, partnerRange.endOffset, editedTagName)
                                PsiDocumentManager.getInstance(project).commitDocument(doc)
                            } finally {
                                isUpdating = false
                            }
                        }
                    }, disposable)
                }
            }
        )
    }

    private fun isClosingTagName(tagNameElement: PsiElement): Boolean {
        var prev = tagNameElement.prevSibling
        while (prev != null && prev.node?.elementType == TokenType.WHITE_SPACE) {
            prev = prev.prevSibling
        }
        return prev?.node?.elementType == VolkiTokenTypes.TAG_BRACKET && prev?.text == "</"
    }

    private fun findMatchingOpenTag(closingTagName: PsiElement): PsiElement? {
        val targetName = closingTagName.text
        var depth = 0
        var cur: PsiElement? = closingTagName.prevSibling

        while (cur != null) {
            if (cur.node?.elementType == VolkiTokenTypes.TAG_BRACKET) {
                when (cur.text) {
                    "</" -> {
                        val nameEl = nextTagName(cur)
                        if (nameEl != null && nameEl != closingTagName) {
                            // Check if this closes the same tag
                            if (nameEl.text == targetName) depth++
                        }
                    }
                    "<" -> {
                        val nameEl = nextTagName(cur)
                        if (nameEl?.node?.elementType == VolkiTokenTypes.TAG_NAME) {
                            // Only count non-self-closing, non-void
                            if (!isSelfClosingBracket(cur) && VolkiElementRegistry.getElement(nameEl.text)?.isVoid != true) {
                                if (depth == 0) return nameEl
                                depth--
                            }
                        }
                    }
                }
            }
            cur = cur.prevSibling
        }
        return null
    }

    private fun findMatchingCloseTag(openingTagName: PsiElement): PsiElement? {
        val targetName = openingTagName.text
        var depth = 0
        var cur: PsiElement? = openingTagName.nextSibling

        while (cur != null) {
            if (cur.node?.elementType == VolkiTokenTypes.TAG_BRACKET) {
                when (cur.text) {
                    "<" -> {
                        val nameEl = nextTagName(cur)
                        if (nameEl?.node?.elementType == VolkiTokenTypes.TAG_NAME && nameEl.text == targetName) {
                            if (!isSelfClosingBracket(cur)) depth++
                        }
                    }
                    "</" -> {
                        val nameEl = nextTagName(cur)
                        if (nameEl?.node?.elementType == VolkiTokenTypes.TAG_NAME && nameEl.text == targetName) {
                            if (depth == 0) return nameEl
                            depth--
                        }
                    }
                }
            }
            cur = cur.nextSibling
        }
        return null
    }

    private fun nextTagName(bracket: PsiElement): PsiElement? {
        var cur = bracket.nextSibling
        while (cur != null && cur.node?.elementType == TokenType.WHITE_SPACE) cur = cur.nextSibling
        return if (cur?.node?.elementType == VolkiTokenTypes.TAG_NAME) cur else null
    }

    private fun isSelfClosingBracket(openBracket: PsiElement): Boolean {
        var cur: PsiElement? = openBracket.nextSibling
        while (cur != null) {
            val type = cur.node?.elementType
            if (type == VolkiTokenTypes.TAG_BRACKET) {
                return cur.text == "/>"
            }
            if (type == VolkiTokenTypes.BRACE_OPEN) return false
            if (type == VolkiTokenTypes.TAG_BRACKET) return false
            cur = cur.nextSibling
        }
        return false
    }
}
