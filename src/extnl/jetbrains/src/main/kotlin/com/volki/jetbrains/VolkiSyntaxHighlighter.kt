package com.volki.jetbrains

import com.intellij.lexer.Lexer
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.HighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.colors.TextAttributesKey.createTextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighterBase
import com.intellij.psi.tree.IElementType

class VolkiSyntaxHighlighter : SyntaxHighlighterBase() {

    companion object {
        val KEYWORD = createTextAttributesKey("VOLKI_KEYWORD", DefaultLanguageHighlighterColors.KEYWORD)
        val TYPE = createTextAttributesKey("VOLKI_TYPE", DefaultLanguageHighlighterColors.CLASS_NAME)
        val TAG_NAME = createTextAttributesKey("VOLKI_TAG_NAME", DefaultLanguageHighlighterColors.MARKUP_TAG)
        val TAG_BRACKET = createTextAttributesKey("VOLKI_TAG_BRACKET", DefaultLanguageHighlighterColors.MARKUP_TAG)
        val ATTRIBUTE = createTextAttributesKey("VOLKI_ATTRIBUTE", DefaultLanguageHighlighterColors.MARKUP_ATTRIBUTE)
        val STRING = createTextAttributesKey("VOLKI_STRING", DefaultLanguageHighlighterColors.STRING)
        val NUMBER = createTextAttributesKey("VOLKI_NUMBER", DefaultLanguageHighlighterColors.NUMBER)
        val LINE_COMMENT = createTextAttributesKey("VOLKI_LINE_COMMENT", DefaultLanguageHighlighterColors.LINE_COMMENT)
        val DOC_COMMENT = createTextAttributesKey("VOLKI_DOC_COMMENT", DefaultLanguageHighlighterColors.DOC_COMMENT)
        val BLOCK_COMMENT = createTextAttributesKey("VOLKI_BLOCK_COMMENT", DefaultLanguageHighlighterColors.BLOCK_COMMENT)
        val BRACES = createTextAttributesKey("VOLKI_BRACES", DefaultLanguageHighlighterColors.BRACES)
        val PARENS = createTextAttributesKey("VOLKI_PARENS", DefaultLanguageHighlighterColors.PARENTHESES)
        val BRACKETS = createTextAttributesKey("VOLKI_BRACKETS", DefaultLanguageHighlighterColors.BRACKETS)
        val OPERATOR = createTextAttributesKey("VOLKI_OPERATOR", DefaultLanguageHighlighterColors.OPERATION_SIGN)
        val IDENTIFIER = createTextAttributesKey("VOLKI_IDENTIFIER", DefaultLanguageHighlighterColors.IDENTIFIER)
        val FUNCTION_DECL = createTextAttributesKey("VOLKI_FUNCTION_DECL", DefaultLanguageHighlighterColors.FUNCTION_DECLARATION)
        val FUNCTION_CALL = createTextAttributesKey("VOLKI_FUNCTION_CALL", DefaultLanguageHighlighterColors.FUNCTION_CALL)
        val METHOD_CALL = createTextAttributesKey("VOLKI_METHOD_CALL", DefaultLanguageHighlighterColors.INSTANCE_METHOD)
        val VARIABLE = createTextAttributesKey("VOLKI_VARIABLE", DefaultLanguageHighlighterColors.LOCAL_VARIABLE)
        val TYPE_REFERENCE = createTextAttributesKey("VOLKI_TYPE_REFERENCE", DefaultLanguageHighlighterColors.CLASS_REFERENCE)
        val RETURN_TYPE = createTextAttributesKey("VOLKI_RETURN_TYPE", DefaultLanguageHighlighterColors.FUNCTION_DECLARATION)
        val RETURN_ARROW = createTextAttributesKey("VOLKI_RETURN_ARROW", DefaultLanguageHighlighterColors.OPERATION_SIGN)
        val BAD_CHARACTER = createTextAttributesKey("VOLKI_BAD_CHARACTER", HighlighterColors.BAD_CHARACTER)
        val HTML_TAG_NAME = createTextAttributesKey("VOLKI_HTML_TAG_NAME", DefaultLanguageHighlighterColors.KEYWORD)
        val CUSTOM_COMPONENT_NAME = createTextAttributesKey("VOLKI_CUSTOM_COMPONENT_NAME", DefaultLanguageHighlighterColors.FUNCTION_CALL)
        val USE_PATH = createTextAttributesKey("VOLKI_USE_PATH", DefaultLanguageHighlighterColors.CLASS_REFERENCE)
        val USE_SYMBOL = createTextAttributesKey("VOLKI_USE_SYMBOL", DefaultLanguageHighlighterColors.STATIC_METHOD)
        val USE_GLOB = createTextAttributesKey("VOLKI_USE_GLOB", DefaultLanguageHighlighterColors.KEYWORD)
        val TERNARY_OPERATOR = createTextAttributesKey("VOLKI_TERNARY_OPERATOR", DefaultLanguageHighlighterColors.OPERATION_SIGN)
        val CONDITIONAL_AND = createTextAttributesKey("VOLKI_CONDITIONAL_AND", DefaultLanguageHighlighterColors.OPERATION_SIGN)
    }

    override fun getHighlightingLexer(): Lexer = VolkiLexer()

    override fun getTokenHighlights(tokenType: IElementType?): Array<TextAttributesKey> {
        val key = when (tokenType) {
            VolkiTokenTypes.KEYWORD -> KEYWORD
            VolkiTokenTypes.TYPE -> TYPE
            VolkiTokenTypes.TAG_NAME -> TAG_NAME
            VolkiTokenTypes.TAG_BRACKET -> TAG_BRACKET
            VolkiTokenTypes.ATTRIBUTE -> ATTRIBUTE
            VolkiTokenTypes.STRING -> STRING
            VolkiTokenTypes.NUMBER -> NUMBER
            VolkiTokenTypes.LINE_COMMENT -> LINE_COMMENT
            VolkiTokenTypes.DOC_COMMENT -> DOC_COMMENT
            VolkiTokenTypes.BLOCK_COMMENT -> BLOCK_COMMENT
            VolkiTokenTypes.BRACE_OPEN, VolkiTokenTypes.BRACE_CLOSE -> BRACES
            VolkiTokenTypes.PAREN_OPEN, VolkiTokenTypes.PAREN_CLOSE -> PARENS
            VolkiTokenTypes.BRACKET_OPEN, VolkiTokenTypes.BRACKET_CLOSE -> BRACKETS
            VolkiTokenTypes.OPERATOR -> OPERATOR
            VolkiTokenTypes.IDENTIFIER -> IDENTIFIER
            VolkiTokenTypes.BAD_CHARACTER -> BAD_CHARACTER
            else -> return emptyArray()
        }
        return arrayOf(key)
    }
}
