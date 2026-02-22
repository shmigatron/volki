package com.volki.jetbrains

import com.intellij.formatting.*
import com.intellij.lang.ASTNode
import com.intellij.openapi.util.TextRange
import com.intellij.psi.TokenType
import com.intellij.psi.codeStyle.CodeStyleSettings
import com.intellij.psi.formatter.common.AbstractBlock

class VolkiFormattingModelBuilder : FormattingModelBuilder {
    override fun createModel(context: FormattingContext): FormattingModel {
        val settings = context.codeStyleSettings
        val spacingBuilder = createSpacingBuilder(settings)
        val block = VolkiBlock(
            context.node,
            null,
            Indent.getNoneIndent(),
            spacingBuilder
        )
        return FormattingModelProvider.createFormattingModelForPsiFile(
            context.containingFile,
            block,
            settings
        )
    }

    private fun createSpacingBuilder(settings: CodeStyleSettings): SpacingBuilder {
        return SpacingBuilder(settings, VolkiLanguage.INSTANCE)
            // Space after keywords
            .after(VolkiTokenTypes.KEYWORD).spaces(1)
            // Space around operators
            .before(VolkiTokenTypes.OPERATOR).spaces(1)
            .after(VolkiTokenTypes.OPERATOR).spaces(1)
            // No space after opening braces/parens or before closing
            .after(VolkiTokenTypes.BRACE_OPEN).spaces(0)
            .before(VolkiTokenTypes.BRACE_CLOSE).spaces(0)
            .after(VolkiTokenTypes.PAREN_OPEN).spaces(0)
            .before(VolkiTokenTypes.PAREN_CLOSE).spaces(0)
            // Space after commas (operator includes comma)
            // No space between tag bracket and tag name: <div not < div
            .between(VolkiTokenTypes.TAG_BRACKET, VolkiTokenTypes.TAG_NAME).spaces(0)
            // Space between tag name and attributes
            .between(VolkiTokenTypes.TAG_NAME, VolkiTokenTypes.ATTRIBUTE).spaces(1)
            .between(VolkiTokenTypes.STRING, VolkiTokenTypes.ATTRIBUTE).spaces(1)
            .between(VolkiTokenTypes.BRACE_CLOSE, VolkiTokenTypes.ATTRIBUTE).spaces(1)
    }
}

class VolkiBlock(
    node: ASTNode,
    private val myWrap: Wrap?,
    private val myIndent: Indent,
    private val spacingBuilder: SpacingBuilder
) : AbstractBlock(node, myWrap, null) {

    override fun buildChildren(): List<Block> {
        val blocks = mutableListOf<Block>()
        var child = myNode.firstChildNode

        while (child != null) {
            if (child.elementType != TokenType.WHITE_SPACE) {
                val indent = computeChildIndent(child)
                blocks.add(VolkiBlock(child, null, indent, spacingBuilder))
            }
            child = child.treeNext
        }
        return blocks
    }

    private fun computeChildIndent(@Suppress("UNUSED_PARAMETER") child: ASTNode): Indent {
        // Top-level file — no indent
        if (myNode.treeParent == null) {
            return Indent.getNoneIndent()
        }

        return Indent.getNoneIndent()
    }

    override fun getIndent(): Indent = myIndent

    override fun getSpacing(child1: Block?, child2: Block): Spacing? {
        return spacingBuilder.getSpacing(this, child1, child2)
    }

    override fun isLeaf(): Boolean = myNode.firstChildNode == null

    override fun getChildAttributes(newChildIndex: Int): ChildAttributes {
        return ChildAttributes(Indent.getNormalIndent(), null)
    }
}
