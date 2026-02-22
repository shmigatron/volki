package com.volki.jetbrains

import com.intellij.lang.*
import com.intellij.lexer.Lexer
import com.intellij.openapi.project.Project
import com.intellij.psi.FileViewProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.impl.source.tree.LeafPsiElement
import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.IFileElementType
import com.intellij.psi.tree.TokenSet

class VolkiParserDefinition : ParserDefinition {

    companion object {
        val FILE = IFileElementType(VolkiLanguage.INSTANCE)

        val WHITE_SPACES = TokenSet.WHITE_SPACE
        val COMMENTS = TokenSet.create(
            VolkiTokenTypes.LINE_COMMENT,
            VolkiTokenTypes.DOC_COMMENT,
            VolkiTokenTypes.BLOCK_COMMENT
        )
        val STRINGS = TokenSet.create(VolkiTokenTypes.STRING)
    }

    override fun createLexer(project: Project?): Lexer = VolkiLexer()

    override fun createParser(project: Project?): PsiParser {
        return object : PsiParser {
            override fun parse(root: IElementType, builder: PsiBuilder): ASTNode {
                val marker = builder.mark()
                while (!builder.eof()) {
                    builder.advanceLexer()
                }
                marker.done(root)
                return builder.treeBuilt
            }
        }
    }

    override fun getFileNodeType(): IFileElementType = FILE

    override fun getWhitespaceTokens(): TokenSet = WHITE_SPACES

    override fun getCommentTokens(): TokenSet = COMMENTS

    override fun getStringLiteralElements(): TokenSet = STRINGS

    override fun createElement(node: ASTNode): PsiElement = LeafPsiElement(node.elementType, node.text)

    override fun createFile(viewProvider: FileViewProvider): PsiFile = VolkiFile(viewProvider)
}
