package com.volki.jetbrains

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.PsiManager
import com.intellij.psi.TokenType
import com.intellij.psi.util.CachedValueProvider
import com.intellij.psi.util.CachedValuesManager
import com.intellij.psi.util.PsiModificationTracker

object VolkiImportResolver {

    // --- Data classes ---

    data class PathSegment(
        val element: PsiElement,
        val text: String,
        val separator: PsiElement? // the `::` operator after this segment, if any
    )

    data class ImportedSymbol(
        val element: PsiElement?,
        val text: String,
        val isGlob: Boolean,
        val isSelf: Boolean
    )

    data class ParsedUseStatement(
        val useKeyword: PsiElement,
        val pathSegments: List<PathSegment>,
        val symbols: List<ImportedSymbol>,
        val semicolon: PsiElement?,
        val startOffset: Int,
        val endOffset: Int
    )

    enum class SymbolKind { FN, CONST, STRUCT, ENUM, TRAIT, TYPE, MOD, STATIC }

    data class ResolvedSymbol(
        val name: String,
        val kind: SymbolKind,
        val signature: String,
        val byteOffset: Int,
        val file: VirtualFile
    )

    // --- Parsing ---

    fun parseUseStatements(file: PsiFile): List<ParsedUseStatement> {
        return CachedValuesManager.getCachedValue(file) {
            CachedValueProvider.Result.create(
                doParseUseStatements(file),
                PsiModificationTracker.MODIFICATION_COUNT
            )
        }
    }

    private fun doParseUseStatements(file: PsiFile): List<ParsedUseStatement> {
        val results = mutableListOf<ParsedUseStatement>()
        var current: PsiElement? = file.firstChild

        while (current != null) {
            if (current.node.elementType == VolkiTokenTypes.KEYWORD && current.text == "use") {
                val stmt = parseOneUseStatement(current)
                if (stmt != null) results.add(stmt)
            }
            current = current.nextSibling
        }
        return results
    }

    private fun parseOneUseStatement(useKeyword: PsiElement): ParsedUseStatement? {
        val segments = mutableListOf<PathSegment>()
        val symbols = mutableListOf<ImportedSymbol>()
        var semicolon: PsiElement? = null
        var current: PsiElement? = useKeyword.nextSibling

        // Phase 1: collect all tokens until semicolon
        val tokens = mutableListOf<PsiElement>()
        while (current != null) {
            val type = current.node.elementType
            if (type == TokenType.WHITE_SPACE) {
                current = current.nextSibling
                continue
            }
            if (type == VolkiTokenTypes.OPERATOR && current.text == ";") {
                semicolon = current
                break
            }
            // Stop if we hit a statement keyword (malformed)
            if (type == VolkiTokenTypes.KEYWORD && current.text in STATEMENT_KEYWORDS) break
            tokens.add(current)
            current = current.nextSibling
        }

        if (tokens.isEmpty()) return null

        // Phase 2: classify tokens into path segments and symbols
        // Find the brace group if present
        val braceOpenIdx = tokens.indexOfFirst {
            it.node.elementType == VolkiTokenTypes.BRACE_OPEN
        }

        if (braceOpenIdx >= 0) {
            // Path segments are everything before the brace open, split by ::
            parsePath(tokens.subList(0, braceOpenIdx), segments)
            // Symbols are inside braces
            parseBraceSymbols(tokens.subList(braceOpenIdx, tokens.size), symbols)
        } else {
            // No braces — last token is the symbol (could be * glob or identifier)
            // Find the last :: separator
            val lastSepIdx = tokens.indexOfLast {
                it.node.elementType == VolkiTokenTypes.OPERATOR && it.text == "::"
            }

            if (lastSepIdx >= 0 && lastSepIdx + 1 < tokens.size) {
                parsePath(tokens.subList(0, lastSepIdx + 1), segments)
                val symbolToken = tokens[lastSepIdx + 1]
                val symbolText = symbolToken.text
                symbols.add(ImportedSymbol(
                    element = symbolToken,
                    text = symbolText,
                    isGlob = symbolText == "*",
                    isSelf = symbolText == "self"
                ))
            } else {
                // Single identifier like `use foo;`
                parsePath(tokens, segments)
            }
        }

        val endOffset = semicolon?.textRange?.endOffset
            ?: tokens.lastOrNull()?.textRange?.endOffset
            ?: useKeyword.textRange.endOffset

        return ParsedUseStatement(
            useKeyword = useKeyword,
            pathSegments = segments,
            symbols = symbols,
            semicolon = semicolon,
            startOffset = useKeyword.textRange.startOffset,
            endOffset = endOffset
        )
    }

    private fun parsePath(tokens: List<PsiElement>, segments: MutableList<PathSegment>) {
        var i = 0
        while (i < tokens.size) {
            val tok = tokens[i]
            val type = tok.node.elementType
            val text = tok.text

            if (type == VolkiTokenTypes.OPERATOR && text == "::") {
                // Separator — attach to previous segment if possible
                if (segments.isNotEmpty()) {
                    val last = segments.removeAt(segments.lastIndex)
                    segments.add(last.copy(separator = tok))
                }
                i++
                continue
            }

            if (type == VolkiTokenTypes.IDENTIFIER || type == VolkiTokenTypes.KEYWORD || type == VolkiTokenTypes.TYPE) {
                segments.add(PathSegment(element = tok, text = text, separator = null))
            }
            i++
        }
    }

    private fun parseBraceSymbols(tokens: List<PsiElement>, symbols: MutableList<ImportedSymbol>) {
        for (tok in tokens) {
            val type = tok.node.elementType
            val text = tok.text
            if (type == VolkiTokenTypes.BRACE_OPEN || type == VolkiTokenTypes.BRACE_CLOSE) continue
            if (type == VolkiTokenTypes.OPERATOR && text == ",") continue
            if (type == VolkiTokenTypes.OPERATOR && text == "::") continue
            if (type == TokenType.WHITE_SPACE) continue

            if (type == VolkiTokenTypes.IDENTIFIER || type == VolkiTokenTypes.KEYWORD || type == VolkiTokenTypes.TYPE) {
                symbols.add(ImportedSymbol(
                    element = tok,
                    text = text,
                    isGlob = false,
                    isSelf = text == "self"
                ))
            } else if (type == VolkiTokenTypes.OPERATOR && text == "*") {
                symbols.add(ImportedSymbol(
                    element = tok,
                    text = "*",
                    isGlob = true,
                    isSelf = false
                ))
            }
        }
    }

    // --- Module path resolution ---

    fun resolveModulePath(
        @Suppress("UNUSED_PARAMETER") project: Project,
        pathSegments: List<PathSegment>,
        sourceFile: PsiFile
    ): VirtualFile? {
        if (pathSegments.isEmpty()) return null
        val srcRoot = findSrcRoot(sourceFile) ?: return null

        var currentDir = srcRoot
        val startIdx = if (pathSegments[0].text == "crate") 1 else 0

        for (i in startIdx until pathSegments.size) {
            val seg = pathSegments[i].text
            currentDir = resolveSegment(currentDir, seg) ?: return null
        }
        return currentDir
    }

    fun resolvePathSegment(
        @Suppress("UNUSED_PARAMETER") project: Project,
        pathSegments: List<PathSegment>,
        upToIndex: Int,
        sourceFile: PsiFile
    ): VirtualFile? {
        if (pathSegments.isEmpty() || upToIndex < 0) return null
        val srcRoot = findSrcRoot(sourceFile) ?: return null

        if (upToIndex == 0 && pathSegments[0].text == "crate") {
            // `crate` -> src root, try lib.rs then main.rs
            return srcRoot.findChild("lib.rs") ?: srcRoot.findChild("main.rs") ?: srcRoot
        }

        var currentDir = srcRoot
        val startIdx = if (pathSegments[0].text == "crate") 1 else 0
        val endIdx = minOf(upToIndex + 1, pathSegments.size)

        for (i in startIdx until endIdx) {
            val seg = pathSegments[i].text
            currentDir = resolveSegment(currentDir, seg) ?: return null
        }
        return currentDir
    }

    private fun resolveSegment(parent: VirtualFile, segment: String): VirtualFile? {
        // If parent is a file, use its parent directory
        val dir = if (parent.isDirectory) parent else parent.parent ?: return null

        // Try: {dir}/{seg}.volki
        dir.findChild("$segment.volki")?.let { return it }
        // Try: {dir}/{seg}/mod.rs
        dir.findChild(segment)?.findChild("mod.rs")?.let { return it }
        // Try: {dir}/{seg}.rs
        dir.findChild("$segment.rs")?.let { return it }
        // Try: {dir}/{seg}/mod.volki
        dir.findChild(segment)?.findChild("mod.volki")?.let { return it }
        // Try: {dir}/{seg}/ as directory
        dir.findChild(segment)?.let { if (it.isDirectory) return it }

        return null
    }

    private fun findSrcRoot(file: PsiFile): VirtualFile? {
        var vf = file.virtualFile ?: return null
        // Walk up to find `src/` directory
        var current: VirtualFile? = vf.parent
        while (current != null) {
            if (current.name == "src" && current.isDirectory) {
                return current
            }
            current = current.parent
        }
        return null
    }

    // --- Symbol finding ---

    fun findSymbolInFile(project: Project, file: VirtualFile, symbolName: String): ResolvedSymbol? {
        val psiFile = PsiManager.getInstance(project).findFile(file) ?: return null
        val text = psiFile.text

        // Pattern matchers for declarations
        val patterns = listOf(
            Regex("""(pub\s+)?fn\s+$symbolName\s*(<[^>]*>)?\s*\([^)]*\)(\s*->\s*\S+)?""") to SymbolKind.FN,
            Regex("""(pub\s+)?const\s+$symbolName\s*:\s*[^=;]+""") to SymbolKind.CONST,
            Regex("""(pub\s+)?static\s+$symbolName\s*:\s*[^=;]+""") to SymbolKind.STATIC,
            Regex("""(pub\s+)?struct\s+$symbolName\b""") to SymbolKind.STRUCT,
            Regex("""(pub\s+)?enum\s+$symbolName\b""") to SymbolKind.ENUM,
            Regex("""(pub\s+)?trait\s+$symbolName\b""") to SymbolKind.TRAIT,
            Regex("""(pub\s+)?type\s+$symbolName\b""") to SymbolKind.TYPE,
        )

        for ((pattern, kind) in patterns) {
            val match = pattern.find(text) ?: continue
            return ResolvedSymbol(
                name = symbolName,
                kind = kind,
                signature = match.value.trim(),
                byteOffset = match.range.first,
                file = file
            )
        }
        return null
    }

    fun findExportedSymbols(project: Project, file: VirtualFile): List<ResolvedSymbol> {
        val psiFile = PsiManager.getInstance(project).findFile(file) ?: return emptyList()
        return getCachedExports(psiFile, project)
    }

    private fun getCachedExports(file: PsiFile, project: Project): List<ResolvedSymbol> {
        return CachedValuesManager.getCachedValue(file) {
            CachedValueProvider.Result.create(
                doFindExportedSymbols(file, project),
                PsiModificationTracker.MODIFICATION_COUNT
            )
        }
    }

    private fun doFindExportedSymbols(psiFile: PsiFile, project: Project): List<ResolvedSymbol> {
        val text = psiFile.text
        val vf = psiFile.virtualFile ?: return emptyList()
        val results = mutableListOf<ResolvedSymbol>()

        // Find pub declarations
        val declPattern = Regex("""pub\s+(fn|const|static|struct|enum|trait|type)\s+(\w+)""")
        for (match in declPattern.findAll(text)) {
            val kindStr = match.groupValues[1]
            val name = match.groupValues[2]
            val kind = when (kindStr) {
                "fn" -> SymbolKind.FN
                "const" -> SymbolKind.CONST
                "static" -> SymbolKind.STATIC
                "struct" -> SymbolKind.STRUCT
                "enum" -> SymbolKind.ENUM
                "trait" -> SymbolKind.TRAIT
                "type" -> SymbolKind.TYPE
                else -> continue
            }
            // Get a more complete signature for functions
            val signature = if (kind == SymbolKind.FN) {
                val fnPattern = Regex("""pub\s+fn\s+$name\s*(<[^>]*>)?\s*\([^)]*\)(\s*->\s*\S+)?""")
                fnPattern.find(text, match.range.first)?.value?.trim() ?: match.value.trim()
            } else {
                match.value.trim()
            }
            results.add(ResolvedSymbol(name, kind, signature, match.range.first, vf))
        }

        // Follow one level of `pub use ...::*` re-exports
        val reExportPattern = Regex("""pub\s+use\s+([\w:]+)::\*\s*;""")
        for (match in reExportPattern.findAll(text)) {
            val path = match.groupValues[1]
            val segments = path.split("::")
            val reExportedFile = resolveReExportPath(project, segments, psiFile)
            if (reExportedFile != null) {
                val reExportedPsi = PsiManager.getInstance(project).findFile(reExportedFile)
                if (reExportedPsi != null) {
                    // Direct search (no recursion to avoid cycles)
                    val reText = reExportedPsi.text
                    for (reMatch in declPattern.findAll(reText)) {
                        val kindStr = reMatch.groupValues[1]
                        val name = reMatch.groupValues[2]
                        val kind = when (kindStr) {
                            "fn" -> SymbolKind.FN
                            "const" -> SymbolKind.CONST
                            "static" -> SymbolKind.STATIC
                            "struct" -> SymbolKind.STRUCT
                            "enum" -> SymbolKind.ENUM
                            "trait" -> SymbolKind.TRAIT
                            "type" -> SymbolKind.TYPE
                            else -> continue
                        }
                        val sig = reMatch.value.trim()
                        results.add(ResolvedSymbol(name, kind, sig, reMatch.range.first, reExportedFile))
                    }
                }
            }
        }

        return results
    }

    private fun resolveReExportPath(@Suppress("UNUSED_PARAMETER") project: Project, segments: List<String>, sourceFile: PsiFile): VirtualFile? {
        val srcRoot = findSrcRoot(sourceFile) ?: return null
        var current = srcRoot
        val startIdx = if (segments.firstOrNull() == "crate") 1 else 0
        for (i in startIdx until segments.size) {
            current = resolveSegment(current, segments[i]) ?: return null
        }
        return current
    }

    // --- Import-based identifier resolution ---

    fun findImportForIdentifier(file: PsiFile, identifierText: String): ResolvedSymbol? {
        val project = file.project
        val useStatements = parseUseStatements(file)

        for (stmt in useStatements) {
            // Check direct symbol imports: use ...::{ ident, ... }
            for (sym in stmt.symbols) {
                if (sym.text == identifierText && !sym.isGlob) {
                    val moduleFile = resolveModulePath(project, stmt.pathSegments, file) ?: continue
                    return findSymbolInFile(project, moduleFile, identifierText)
                }
            }

            // Check glob imports: use ...::*
            if (stmt.symbols.any { it.isGlob }) {
                val moduleFile = resolveModulePath(project, stmt.pathSegments, file) ?: continue
                val symbol = findSymbolInFile(project, moduleFile, identifierText)
                if (symbol != null) return symbol

                // Also check exported symbols (for re-exports)
                val exports = findExportedSymbols(project, moduleFile)
                val match = exports.find { it.name == identifierText }
                if (match != null) return match
            }
        }
        return null
    }

    // --- Context detection helpers ---

    fun findEnclosingUseKeyword(element: PsiElement): PsiElement? {
        var sibling: PsiElement? = element.prevSibling
        while (sibling != null) {
            val type = sibling.node?.elementType
            val text = sibling.text

            // Found use keyword
            if (type == VolkiTokenTypes.KEYWORD && text == "use") return sibling

            // Hit a statement boundary — not inside a use statement
            if (type == VolkiTokenTypes.OPERATOR && text == ";") return null
            if (type == VolkiTokenTypes.KEYWORD && text in STATEMENT_KEYWORDS) return null
            if (type == VolkiTokenTypes.BRACE_CLOSE) return null

            sibling = sibling.prevSibling
        }
        return null
    }

    fun isInsideUseBraces(element: PsiElement): Boolean {
        var sibling: PsiElement? = element.prevSibling
        while (sibling != null) {
            val type = sibling.node?.elementType
            if (type == VolkiTokenTypes.BRACE_OPEN) {
                // Verify there's a `use` keyword before the brace
                return findEnclosingUseKeyword(sibling) != null
            }
            if (type == VolkiTokenTypes.BRACE_CLOSE) return false
            if (type == VolkiTokenTypes.OPERATOR && sibling.text == ";") return false
            sibling = sibling.prevSibling
        }
        return false
    }

    fun isFollowedByPathSeparator(element: PsiElement): Boolean {
        var next: PsiElement? = element.nextSibling
        while (next != null && next.node.elementType == TokenType.WHITE_SPACE) {
            next = next.nextSibling
        }
        return next != null && next.node.elementType == VolkiTokenTypes.OPERATOR && next.text == "::"
    }

    fun isPrecededByPathSeparator(element: PsiElement): Boolean {
        var prev: PsiElement? = element.prevSibling
        while (prev != null && prev.node.elementType == TokenType.WHITE_SPACE) {
            prev = prev.prevSibling
        }
        return prev != null && prev.node.elementType == VolkiTokenTypes.OPERATOR && prev.text == "::"
    }

    /**
     * Find the index of this element within the path segments of its enclosing use statement.
     * Returns -1 if not found.
     */
    fun findSegmentIndex(file: PsiFile, element: PsiElement): Int {
        val useStatements = parseUseStatements(file)
        val offset = element.textRange.startOffset
        for (stmt in useStatements) {
            for ((idx, seg) in stmt.pathSegments.withIndex()) {
                if (seg.element.textRange.startOffset == offset) return idx
            }
        }
        return -1
    }

    /**
     * Find the ParsedUseStatement that contains the given element.
     */
    fun findContainingUseStatement(file: PsiFile, element: PsiElement): ParsedUseStatement? {
        val offset = element.textRange.startOffset
        val useStatements = parseUseStatements(file)
        for (stmt in useStatements) {
            if (offset in stmt.startOffset..stmt.endOffset) return stmt
        }
        return null
    }

    // --- Constants ---

    private val STATEMENT_KEYWORDS = setOf(
        "fn", "struct", "enum", "trait", "impl", "mod", "const", "static",
        "let", "pub", "type", "extern", "async", "unsafe"
    )
}
