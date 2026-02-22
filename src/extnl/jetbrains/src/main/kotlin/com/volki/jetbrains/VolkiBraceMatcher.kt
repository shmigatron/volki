package com.volki.jetbrains

import com.intellij.lang.BracePair
import com.intellij.lang.PairedBraceMatcher
import com.intellij.psi.PsiFile
import com.intellij.psi.tree.IElementType

class VolkiBraceMatcher : PairedBraceMatcher {

    companion object {
        private val PAIRS = arrayOf(
            BracePair(VolkiTokenTypes.BRACE_OPEN, VolkiTokenTypes.BRACE_CLOSE, true),
            BracePair(VolkiTokenTypes.PAREN_OPEN, VolkiTokenTypes.PAREN_CLOSE, false),
            BracePair(VolkiTokenTypes.BRACKET_OPEN, VolkiTokenTypes.BRACKET_CLOSE, false),
        )
    }

    override fun getPairs(): Array<BracePair> = PAIRS

    override fun isPairedBracesAllowedBeforeType(lbraceType: IElementType, contextType: IElementType?): Boolean = true

    override fun getCodeConstructStart(file: PsiFile?, openingBraceOffset: Int): Int = openingBraceOffset
}
