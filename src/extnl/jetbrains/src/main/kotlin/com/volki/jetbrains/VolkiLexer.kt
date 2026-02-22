package com.volki.jetbrains

import com.intellij.lexer.LexerBase
import com.intellij.psi.tree.IElementType

class VolkiLexer : LexerBase() {

    private var buffer: CharSequence = ""
    private var startOffset = 0
    private var endOffset = 0
    private var pos = 0
    private var tokenStart = 0
    private var tokenEnd = 0
    private var tokenType: IElementType? = null
    private var state = STATE_RUST

    companion object {
        const val STATE_RUST = 0
        const val STATE_TAG_OPEN = 1
        const val STATE_TAG_CLOSE = 2

        private val KEYWORDS = setOf(
            "pub", "fn", "let", "mut", "const", "for", "if", "else", "while",
            "loop", "match", "return", "break", "continue", "use", "unsafe",
            "mod", "crate", "self", "super", "struct", "enum", "impl", "trait",
            "where", "as", "in", "ref", "true", "false", "type", "async", "await",
            "move", "dyn", "static", "extern"
        )

        private val TYPES = setOf(
            "Html", "Fragment", "Client", "HtmlNode", "HtmlElement",
            "Request", "Response", "Metadata", "Style", "Head",
            "Vec", "Result", "Option", "String", "bool",
            "i8", "i16", "i32", "i64", "i128", "isize",
            "u8", "u16", "u32", "u64", "u128", "usize",
            "f32", "f64", "str", "char", "Box", "Rc", "Arc",
            "HashMap", "HashSet", "BTreeMap", "BTreeSet"
        )

        private val TAG_NAMES = setOf(
            "div", "span", "p", "a", "h1", "h2", "h3", "h4", "h5", "h6",
            "table", "tr", "td", "th", "thead", "tbody", "tfoot",
            "input", "button", "form", "textarea", "select", "option", "label",
            "nav", "ul", "ol", "li", "dl", "dt", "dd",
            "img", "br", "hr", "pre", "code", "blockquote",
            "header", "footer", "main", "section", "article", "aside",
            "strong", "em", "b", "i", "u", "small", "sub", "sup",
            "video", "audio", "source", "canvas", "svg", "path",
            "meta", "link", "script", "style", "title", "body", "html",
            "details", "summary", "dialog", "figure", "figcaption"
        )
    }

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
        this.buffer = buffer
        this.startOffset = startOffset
        this.endOffset = endOffset
        this.pos = startOffset
        this.state = initialState
        this.tokenType = null
        advance()
    }

    override fun getState(): Int = state

    override fun getTokenType(): IElementType? = tokenType

    override fun getTokenStart(): Int = tokenStart

    override fun getTokenEnd(): Int = tokenEnd

    override fun getBufferSequence(): CharSequence = buffer

    override fun getBufferEnd(): Int = endOffset

    override fun advance() {
        if (pos >= endOffset) {
            tokenType = null
            return
        }

        tokenStart = pos

        when (state) {
            STATE_RUST -> lexRust()
            STATE_TAG_OPEN -> lexTagOpen()
            STATE_TAG_CLOSE -> lexTagClose()
        }

        tokenEnd = pos
    }

    private fun lexRust() {
        val c = buffer[pos]

        // Whitespace
        if (c.isWhitespace()) {
            while (pos < endOffset && buffer[pos].isWhitespace()) pos++
            tokenType = com.intellij.psi.TokenType.WHITE_SPACE
            return
        }

        // Comments
        if (c == '/' && pos + 1 < endOffset) {
            val next = buffer[pos + 1]
            if (next == '/') {
                if (pos + 2 < endOffset && buffer[pos + 2] == '!') {
                    // Doc comment
                    while (pos < endOffset && buffer[pos] != '\n') pos++
                    tokenType = VolkiTokenTypes.DOC_COMMENT
                    return
                }
                // Line comment
                while (pos < endOffset && buffer[pos] != '\n') pos++
                tokenType = VolkiTokenTypes.LINE_COMMENT
                return
            }
            if (next == '*') {
                pos += 2
                var depth = 1
                while (pos < endOffset && depth > 0) {
                    if (buffer[pos] == '/' && pos + 1 < endOffset && buffer[pos + 1] == '*') {
                        depth++
                        pos += 2
                    } else if (buffer[pos] == '*' && pos + 1 < endOffset && buffer[pos + 1] == '/') {
                        depth--
                        pos += 2
                    } else {
                        pos++
                    }
                }
                tokenType = VolkiTokenTypes.BLOCK_COMMENT
                return
            }
        }

        // Raw string r#"..."#
        if (c == 'r' && pos + 1 < endOffset && buffer[pos + 1] == '#') {
            pos++
            var hashes = 0
            while (pos < endOffset && buffer[pos] == '#') { hashes++; pos++ }
            if (pos < endOffset && buffer[pos] == '"') {
                pos++ // skip opening quote
                while (pos < endOffset) {
                    if (buffer[pos] == '"') {
                        pos++
                        var closingHashes = 0
                        while (closingHashes < hashes && pos < endOffset && buffer[pos] == '#') {
                            closingHashes++
                            pos++
                        }
                        if (closingHashes == hashes) {
                            tokenType = VolkiTokenTypes.STRING
                            return
                        }
                    } else {
                        pos++
                    }
                }
                tokenType = VolkiTokenTypes.STRING
                return
            }
            // Not a raw string, backtrack and treat as identifier
            pos = tokenStart
        }

        // String literal
        if (c == '"') {
            pos++
            while (pos < endOffset && buffer[pos] != '"') {
                if (buffer[pos] == '\\' && pos + 1 < endOffset) pos++
                pos++
            }
            if (pos < endOffset) pos++ // closing quote
            tokenType = VolkiTokenTypes.STRING
            return
        }

        // Char literal
        if (c == '\'' && pos + 1 < endOffset && !buffer[pos + 1].isLetter().not()) {
            val saved = pos
            pos++
            if (pos < endOffset && buffer[pos] == '\\') pos++
            if (pos < endOffset) pos++
            if (pos < endOffset && buffer[pos] == '\'') {
                pos++
                tokenType = VolkiTokenTypes.STRING
                return
            }
            // Lifetime or not a char — backtrack
            pos = saved
        }

        // Number literal
        if (c.isDigit()) {
            if (c == '0' && pos + 1 < endOffset) {
                val prefix = buffer[pos + 1]
                if (prefix == 'x' || prefix == 'o' || prefix == 'b') {
                    pos += 2
                    while (pos < endOffset && (buffer[pos].isLetterOrDigit() || buffer[pos] == '_')) pos++
                    tokenType = VolkiTokenTypes.NUMBER
                    return
                }
            }
            while (pos < endOffset && (buffer[pos].isDigit() || buffer[pos] == '_')) pos++
            if (pos < endOffset && buffer[pos] == '.' && pos + 1 < endOffset && buffer[pos + 1].isDigit()) {
                pos++
                while (pos < endOffset && (buffer[pos].isDigit() || buffer[pos] == '_')) pos++
            }
            // Type suffix like i32, u64, f64
            if (pos < endOffset && (buffer[pos] == 'i' || buffer[pos] == 'u' || buffer[pos] == 'f')) {
                val suffixStart = pos
                pos++
                while (pos < endOffset && buffer[pos].isDigit()) pos++
                if (pos == suffixStart + 1) pos = suffixStart // no digits after prefix, backtrack
            }
            tokenType = VolkiTokenTypes.NUMBER
            return
        }

        // RSX: < followed by tag name or / (closing tag) or uppercase (component)
        if (c == '<') {
            if (pos + 1 < endOffset) {
                val next = buffer[pos + 1]
                if (next == '/') {
                    // Closing tag: </
                    pos += 2
                    tokenType = VolkiTokenTypes.TAG_BRACKET
                    state = STATE_TAG_CLOSE
                    return
                }
                if (next.isLetter()) {
                    val identStart = pos + 1
                    var identEnd = identStart
                    while (identEnd < endOffset && (buffer[identEnd].isLetterOrDigit() || buffer[identEnd] == '_')) identEnd++
                    val ident = buffer.subSequence(identStart, identEnd).toString()
                    if (ident in TAG_NAMES || ident[0].isUpperCase()) {
                        // RSX opening tag
                        pos++
                        tokenType = VolkiTokenTypes.TAG_BRACKET
                        state = STATE_TAG_OPEN
                        return
                    }
                }
            }
            // Generic angle bracket or comparison
            pos++
            if (pos < endOffset && buffer[pos] == '=') pos++
            tokenType = VolkiTokenTypes.OPERATOR
            return
        }

        // Braces, parens, brackets
        when (c) {
            '{' -> { pos++; tokenType = VolkiTokenTypes.BRACE_OPEN; return }
            '}' -> { pos++; tokenType = VolkiTokenTypes.BRACE_CLOSE; return }
            '(' -> { pos++; tokenType = VolkiTokenTypes.PAREN_OPEN; return }
            ')' -> { pos++; tokenType = VolkiTokenTypes.PAREN_CLOSE; return }
            '[' -> { pos++; tokenType = VolkiTokenTypes.BRACKET_OPEN; return }
            ']' -> { pos++; tokenType = VolkiTokenTypes.BRACKET_CLOSE; return }
        }

        // Operators
        if (c == '-' && pos + 1 < endOffset && buffer[pos + 1] == '>') {
            pos += 2; tokenType = VolkiTokenTypes.OPERATOR; return
        }
        if (c == '=' && pos + 1 < endOffset && buffer[pos + 1] == '>') {
            pos += 2; tokenType = VolkiTokenTypes.OPERATOR; return
        }
        if (c == ':' && pos + 1 < endOffset && buffer[pos + 1] == ':') {
            pos += 2; tokenType = VolkiTokenTypes.OPERATOR; return
        }
        if (c == '>' && pos + 1 < endOffset && buffer[pos + 1] == '=') {
            pos += 2; tokenType = VolkiTokenTypes.OPERATOR; return
        }
        if (c == '!' && pos + 1 < endOffset && buffer[pos + 1] == '=') {
            pos += 2; tokenType = VolkiTokenTypes.OPERATOR; return
        }
        if (c == '=' && pos + 1 < endOffset && buffer[pos + 1] == '=') {
            pos += 2; tokenType = VolkiTokenTypes.OPERATOR; return
        }
        if (c == '&' && pos + 1 < endOffset && buffer[pos + 1] == '&') {
            pos += 2; tokenType = VolkiTokenTypes.OPERATOR; return
        }
        if (c == '|' && pos + 1 < endOffset && buffer[pos + 1] == '|') {
            pos += 2; tokenType = VolkiTokenTypes.OPERATOR; return
        }
        if (c in ">.,:;=!&|?#@+*/%^~") {
            pos++; tokenType = VolkiTokenTypes.OPERATOR; return
        }

        // Identifier / keyword / type
        if (c.isLetter() || c == '_') {
            while (pos < endOffset && (buffer[pos].isLetterOrDigit() || buffer[pos] == '_')) pos++
            val word = buffer.subSequence(tokenStart, pos).toString()
            tokenType = when {
                word in KEYWORDS -> VolkiTokenTypes.KEYWORD
                word in TYPES -> VolkiTokenTypes.TYPE
                else -> VolkiTokenTypes.IDENTIFIER
            }
            return
        }

        // Bad character fallback
        pos++
        tokenType = VolkiTokenTypes.BAD_CHARACTER
    }

    private fun lexTagOpen() {
        val c = buffer[pos]

        // Whitespace inside tag
        if (c.isWhitespace()) {
            while (pos < endOffset && buffer[pos].isWhitespace()) pos++
            tokenType = com.intellij.psi.TokenType.WHITE_SPACE
            return
        }

        // Self-closing />
        if (c == '/' && pos + 1 < endOffset && buffer[pos + 1] == '>') {
            pos += 2
            tokenType = VolkiTokenTypes.TAG_BRACKET
            state = STATE_RUST
            return
        }

        // Closing >
        if (c == '>') {
            pos++
            tokenType = VolkiTokenTypes.TAG_BRACKET
            state = STATE_RUST
            return
        }

        // String attribute value
        if (c == '"') {
            pos++
            while (pos < endOffset && buffer[pos] != '"') {
                if (buffer[pos] == '\\' && pos + 1 < endOffset) pos++
                pos++
            }
            if (pos < endOffset) pos++
            tokenType = VolkiTokenTypes.STRING
            return
        }

        // Brace expression in attribute
        if (c == '{') {
            pos++
            tokenType = VolkiTokenTypes.BRACE_OPEN
            state = STATE_RUST
            return
        }

        // = sign in attributes
        if (c == '=') {
            pos++
            tokenType = VolkiTokenTypes.OPERATOR
            return
        }

        // Tag name or attribute identifier
        if (c.isLetter() || c == '_') {
            while (pos < endOffset && (buffer[pos].isLetterOrDigit() || buffer[pos] == '_' || buffer[pos] == '-')) pos++
            val word = buffer.subSequence(tokenStart, pos).toString()

            // Check if this is followed by = (attribute) or not (tag name)
            val peekPos = skipWhitespace(pos)
            if (peekPos < endOffset && buffer[peekPos] == '=') {
                tokenType = VolkiTokenTypes.ATTRIBUTE
            } else if (word in TAG_NAMES || word[0].isUpperCase()) {
                tokenType = VolkiTokenTypes.TAG_NAME
            } else {
                tokenType = VolkiTokenTypes.ATTRIBUTE
            }
            return
        }

        // Bad character
        pos++
        tokenType = VolkiTokenTypes.BAD_CHARACTER
    }

    private fun lexTagClose() {
        val c = buffer[pos]

        if (c.isWhitespace()) {
            while (pos < endOffset && buffer[pos].isWhitespace()) pos++
            tokenType = com.intellij.psi.TokenType.WHITE_SPACE
            return
        }

        if (c == '>') {
            pos++
            tokenType = VolkiTokenTypes.TAG_BRACKET
            state = STATE_RUST
            return
        }

        if (c.isLetter() || c == '_') {
            while (pos < endOffset && (buffer[pos].isLetterOrDigit() || buffer[pos] == '_' || buffer[pos] == '-')) pos++
            tokenType = VolkiTokenTypes.TAG_NAME
            return
        }

        pos++
        tokenType = VolkiTokenTypes.BAD_CHARACTER
    }

    private fun skipWhitespace(from: Int): Int {
        var i = from
        while (i < endOffset && buffer[i].isWhitespace()) i++
        return i
    }
}
