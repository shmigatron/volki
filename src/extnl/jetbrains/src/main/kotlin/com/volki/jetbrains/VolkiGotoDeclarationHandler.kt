package com.volki.jetbrains

import com.intellij.codeInsight.navigation.actions.GotoDeclarationHandler
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiManager
import com.intellij.psi.search.FilenameIndex
import com.intellij.psi.search.GlobalSearchScope

class VolkiGotoDeclarationHandler : GotoDeclarationHandler {

    override fun getGotoDeclarationTargets(
        sourceElement: PsiElement?,
        offset: Int,
        editor: Editor?
    ): Array<PsiElement>? {
        val element = sourceElement ?: return null
        val project = element.project
        val tokenType = element.node?.elementType ?: return null

        return when (tokenType) {
            VolkiTokenTypes.TAG_NAME -> handleTagName(element, project)
            VolkiTokenTypes.ATTRIBUTE -> handleAttribute(element, project)
            VolkiTokenTypes.IDENTIFIER -> handleIdentifierOrType(element, project)
            VolkiTokenTypes.TYPE -> handleIdentifierOrType(element, project)
            VolkiTokenTypes.KEYWORD -> handleKeyword(element, project)
            else -> null
        }
    }

    // --- Identifier / Type navigation ---

    private fun handleIdentifierOrType(element: PsiElement, project: Project): Array<PsiElement>? {
        val text = element.text
        val file = element.containingFile ?: return null

        // Case 1: Inside a use statement as a path segment (followed by ::)
        val useKeyword = VolkiImportResolver.findEnclosingUseKeyword(element)
        if (useKeyword != null) {
            if (VolkiImportResolver.isFollowedByPathSeparator(element)) {
                // Path segment — resolve up to this segment and navigate to the module
                return navigateToPathSegment(element, project, file)
            }

            // Case 2: Inside a use statement as an imported symbol
            if (VolkiImportResolver.isInsideUseBraces(element) || VolkiImportResolver.isPrecededByPathSeparator(element)) {
                return navigateToImportedSymbol(element, project, file, text)
            }
        }

        // Case 3: Not in a use statement — resolve through imports
        val resolved = VolkiImportResolver.findImportForIdentifier(file, text)
        if (resolved != null) {
            return navigateToResolvedSymbol(project, resolved)
        }

        // Case 4: Try declaration files (types, metadata, http, etc.)
        val declResult = VolkiDeclarationResolver.findSymbolInDeclarations(project, text)
        if (declResult != null) return arrayOf(declResult)

        // Fall back to custom component search for uppercase identifiers
        if (text.isNotEmpty() && text[0].isUpperCase()) {
            return findCustomComponent(project, text)
        }

        // Fall back to searching current file for local fn definition
        val localResult = findLocalDefinition(file, text, project)
        if (localResult != null) return localResult

        return null
    }

    private fun navigateToPathSegment(element: PsiElement, project: Project, @Suppress("UNUSED_PARAMETER") file: PsiElement): Array<PsiElement>? {
        val psiFile = element.containingFile ?: return null
        val stmt = VolkiImportResolver.findContainingUseStatement(psiFile, element) ?: return null
        val segIdx = VolkiImportResolver.findSegmentIndex(psiFile, element)
        if (segIdx < 0) return null

        val resolved = VolkiImportResolver.resolvePathSegment(project, stmt.pathSegments, segIdx, psiFile)
            ?: return null

        val targetFile = if (resolved.isDirectory) {
            resolved.findChild("mod.rs") ?: resolved.findChild("mod.volki") ?: resolved
        } else {
            resolved
        }

        val psi = PsiManager.getInstance(project).findFile(targetFile) ?: return null
        return arrayOf(psi)
    }

    private fun navigateToImportedSymbol(
        element: PsiElement,
        project: Project,
        @Suppress("UNUSED_PARAMETER") file: PsiElement,
        symbolName: String
    ): Array<PsiElement>? {
        val psiFile = element.containingFile ?: return null
        val stmt = VolkiImportResolver.findContainingUseStatement(psiFile, element) ?: return null
        val moduleFile = VolkiImportResolver.resolveModulePath(project, stmt.pathSegments, psiFile)
            ?: return null

        val resolved = VolkiImportResolver.findSymbolInFile(project, moduleFile, symbolName)
        if (resolved != null) {
            return navigateToResolvedSymbol(project, resolved)
        }

        // Check re-exports
        val exports = VolkiImportResolver.findExportedSymbols(project, moduleFile)
        val exported = exports.find { it.name == symbolName }
        if (exported != null) {
            return navigateToResolvedSymbol(project, exported)
        }

        return null
    }

    private fun navigateToResolvedSymbol(project: Project, resolved: VolkiImportResolver.ResolvedSymbol): Array<PsiElement>? {
        val psiFile = PsiManager.getInstance(project).findFile(resolved.file) ?: return null
        val targetElement = psiFile.findElementAt(resolved.byteOffset) ?: return null
        return arrayOf(targetElement)
    }

    private fun findLocalDefinition(file: PsiElement, name: String, @Suppress("UNUSED_PARAMETER") project: Project): Array<PsiElement>? {
        val psiFile = file as? com.intellij.psi.PsiFile ?: file.containingFile ?: return null
        val text = psiFile.text
        val pattern = "fn $name("
        val idx = text.indexOf(pattern)
        if (idx >= 0) {
            val target = psiFile.findElementAt(idx)
            if (target != null) return arrayOf(target)
        }
        return null
    }

    // --- Keyword navigation ---

    private fun handleKeyword(element: PsiElement, project: Project): Array<PsiElement>? {
        val text = element.text
        val useKeyword = VolkiImportResolver.findEnclosingUseKeyword(element)

        // Only handle crate/self/super in use statement paths
        if (useKeyword == null) return null

        when (text) {
            "crate" -> {
                // Navigate to src/lib.rs or src/main.rs
                val file = element.containingFile ?: return null
                val srcRoot = findSrcRoot(file) ?: return null
                val target = srcRoot.findChild("lib.rs")
                    ?: srcRoot.findChild("main.rs")
                    ?: return null
                val psi = PsiManager.getInstance(project).findFile(target) ?: return null
                return arrayOf(psi)
            }
            "self" -> {
                // Navigate to current module file
                val file = element.containingFile ?: return null
                return arrayOf(file)
            }
            "super" -> {
                // Navigate to parent module
                val file = element.containingFile ?: return null
                val vf = file.virtualFile ?: return null
                val parentDir = vf.parent ?: return null
                val target = parentDir.parent?.findChild("${parentDir.name}.rs")
                    ?: parentDir.findChild("mod.rs")
                    ?: return null
                val psi = PsiManager.getInstance(project).findFile(target) ?: return null
                return arrayOf(psi)
            }
        }
        return null
    }

    private fun findSrcRoot(file: com.intellij.psi.PsiFile): VirtualFile? {
        var current: VirtualFile? = file.virtualFile?.parent
        while (current != null) {
            if (current.name == "src" && current.isDirectory) return current
            current = current.parent
        }
        return null
    }

    // --- Existing tag/attribute handlers ---

    private fun handleTagName(element: PsiElement, project: Project): Array<PsiElement>? {
        val tag = element.text

        // Try declaration files first (elements.volki, special.volki)
        val declResult = VolkiDeclarationResolver.findTagInDeclarations(project, tag)
        if (declResult != null) return arrayOf(declResult)

        val info = VolkiElementRegistry.getElement(tag)
        if (info != null) {
            // Built-in HTML element — fall back to element.rs
            return findInElementRs(project, "pub fn ${info.rustConstructor}()")
        }

        // Custom component (uppercase) — search project files for fn definition
        if (tag.isNotEmpty() && tag[0].isUpperCase()) {
            return findCustomComponent(project, tag)
        }

        return null
    }

    private fun handleAttribute(element: PsiElement, project: Project): Array<PsiElement>? {
        val attrName = element.text

        // Navigate to the builder method in element.rs
        val methodName = when (attrName) {
            "class" -> "fn class("
            "id" -> "fn id("
            else -> "fn attr("
        }
        return findInElementRs(project, methodName)
    }

    private fun findInElementRs(project: Project, searchText: String): Array<PsiElement>? {
        val scope = GlobalSearchScope.allScope(project)
        val files = FilenameIndex.getVirtualFilesByName("element.rs", scope)

        for (vf in files) {
            val psiFile = PsiManager.getInstance(project).findFile(vf) ?: continue
            val text = psiFile.text
            val idx = text.indexOf(searchText)
            if (idx < 0) continue

            val targetElement = psiFile.findElementAt(idx)
            if (targetElement != null) {
                return arrayOf(targetElement)
            }
        }

        return null
    }

    private fun findCustomComponent(project: Project, componentName: String): Array<PsiElement>? {
        val results = mutableListOf<PsiElement>()
        val scope = GlobalSearchScope.allScope(project)
        val psiManager = PsiManager.getInstance(project)

        // Search .volki files for fn ComponentName(
        val volkiFiles = FilenameIndex.getAllFilesByExt(project, "volki", scope)
        searchFilesForPattern(volkiFiles, psiManager, "fn $componentName(", results)

        // Search .rs files for fn ComponentName(
        val rsFiles = FilenameIndex.getAllFilesByExt(project, "rs", scope)
        searchFilesForPattern(rsFiles, psiManager, "fn $componentName(", results)

        return if (results.isNotEmpty()) results.toTypedArray() else null
    }

    private fun searchFilesForPattern(
        files: Collection<VirtualFile>,
        psiManager: PsiManager,
        pattern: String,
        results: MutableList<PsiElement>
    ) {
        for (vf in files) {
            val psiFile = psiManager.findFile(vf) ?: continue
            val text = psiFile.text
            var idx = text.indexOf(pattern)
            while (idx >= 0) {
                val target = psiFile.findElementAt(idx)
                if (target != null) {
                    results.add(target)
                }
                idx = text.indexOf(pattern, idx + 1)
            }
        }
    }
}
