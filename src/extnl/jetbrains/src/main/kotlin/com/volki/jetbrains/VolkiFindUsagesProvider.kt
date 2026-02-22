package com.volki.jetbrains

import com.intellij.lang.cacheBuilder.DefaultWordsScanner
import com.intellij.lang.cacheBuilder.WordsScanner
import com.intellij.lang.findUsages.FindUsagesProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.tree.TokenSet

class VolkiFindUsagesProvider : FindUsagesProvider {

    override fun getWordsScanner(): WordsScanner {
        return DefaultWordsScanner(
            VolkiLexer(),
            TokenSet.create(VolkiTokenTypes.IDENTIFIER, VolkiTokenTypes.TAG_NAME),
            TokenSet.create(VolkiTokenTypes.LINE_COMMENT, VolkiTokenTypes.DOC_COMMENT, VolkiTokenTypes.BLOCK_COMMENT),
            TokenSet.create(VolkiTokenTypes.STRING)
        )
    }

    override fun canFindUsagesFor(psiElement: PsiElement): Boolean {
        val type = psiElement.node?.elementType
        return type == VolkiTokenTypes.TAG_NAME ||
               type == VolkiTokenTypes.IDENTIFIER ||
               type == VolkiTokenTypes.KEYWORD
    }

    override fun getHelpId(psiElement: PsiElement): String? = null

    override fun getType(element: PsiElement): String {
        return when (element.node?.elementType) {
            VolkiTokenTypes.TAG_NAME -> {
                if (element.text.isNotEmpty() && element.text[0].isUpperCase()) "component"
                else "tag"
            }
            VolkiTokenTypes.IDENTIFIER -> "identifier"
            VolkiTokenTypes.KEYWORD -> "keyword"
            else -> "element"
        }
    }

    override fun getDescriptiveName(element: PsiElement): String = element.text

    override fun getNodeText(element: PsiElement, useFullName: Boolean): String = element.text
}
