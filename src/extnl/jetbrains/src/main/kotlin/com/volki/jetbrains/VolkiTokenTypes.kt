package com.volki.jetbrains

import com.intellij.psi.tree.IElementType

class VolkiTokenType(debugName: String) : IElementType(debugName, VolkiLanguage.INSTANCE) {
    override fun toString(): String = "VolkiTokenType.${super.toString()}"
}

object VolkiTokenTypes {
    @JvmField val KEYWORD = VolkiTokenType("KEYWORD")
    @JvmField val TYPE = VolkiTokenType("TYPE")
    @JvmField val TAG_NAME = VolkiTokenType("TAG_NAME")
    @JvmField val TAG_BRACKET = VolkiTokenType("TAG_BRACKET")
    @JvmField val ATTRIBUTE = VolkiTokenType("ATTRIBUTE")
    @JvmField val STRING = VolkiTokenType("STRING")
    @JvmField val NUMBER = VolkiTokenType("NUMBER")
    @JvmField val LINE_COMMENT = VolkiTokenType("LINE_COMMENT")
    @JvmField val DOC_COMMENT = VolkiTokenType("DOC_COMMENT")
    @JvmField val BLOCK_COMMENT = VolkiTokenType("BLOCK_COMMENT")
    @JvmField val BRACE_OPEN = VolkiTokenType("BRACE_OPEN")
    @JvmField val BRACE_CLOSE = VolkiTokenType("BRACE_CLOSE")
    @JvmField val PAREN_OPEN = VolkiTokenType("PAREN_OPEN")
    @JvmField val PAREN_CLOSE = VolkiTokenType("PAREN_CLOSE")
    @JvmField val BRACKET_OPEN = VolkiTokenType("BRACKET_OPEN")
    @JvmField val BRACKET_CLOSE = VolkiTokenType("BRACKET_CLOSE")
    @JvmField val ANGLE_OPEN = VolkiTokenType("ANGLE_OPEN")
    @JvmField val ANGLE_CLOSE = VolkiTokenType("ANGLE_CLOSE")
    @JvmField val OPERATOR = VolkiTokenType("OPERATOR")
    @JvmField val IDENTIFIER = VolkiTokenType("IDENTIFIER")
    @JvmField val BAD_CHARACTER = VolkiTokenType("BAD_CHARACTER")
}
