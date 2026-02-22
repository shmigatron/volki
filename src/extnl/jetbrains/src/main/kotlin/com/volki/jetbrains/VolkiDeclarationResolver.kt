package com.volki.jetbrains

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiManager
import com.intellij.psi.search.FilenameIndex
import com.intellij.psi.search.GlobalSearchScope

object VolkiDeclarationResolver {

    // Maps special compiler tags to their declaration file
    private val TAG_TO_DECL_FILE = mapOf(
        "Style" to "special.volki",
        "Head" to "special.volki",
        "Stylesheet" to "special.volki"
    )

    // Maps prelude types/structs to their declaration file
    private val TYPE_TO_DECL_FILE = mapOf(
        // types.volki
        "HtmlNode" to "types.volki",
        "HtmlElement" to "types.volki",
        "IntoChildren" to "types.volki",
        "Html" to "types.volki",
        "Fragment" to "types.volki",
        "Client" to "types.volki",
        "Component" to "types.volki",
        // metadata.volki
        "Metadata" to "metadata.volki",
        "MetadataFn" to "metadata.volki",
        "Robots" to "metadata.volki",
        "MetadataWarning" to "metadata.volki",
        // document.volki
        "HtmlDocument" to "document.volki",
        // http.volki
        "Method" to "http.volki",
        "StatusCode" to "http.volki",
        "Headers" to "http.volki",
        "Request" to "http.volki",
        "Response" to "http.volki",
        "Handler" to "http.volki",
        // special.volki
        "Server" to "special.volki",
        "FileRoute" to "special.volki",
        "PageHandler" to "special.volki",
        "Style" to "special.volki",
        "Head" to "special.volki",
        "Stylesheet" to "special.volki"
    )

    // All known HTML element constructor names (lowercase tags)
    private val ELEMENT_CONSTRUCTORS = setOf(
        "div", "span", "header", "footer", "main_el", "nav", "section", "article",
        "p", "h1", "h2", "h3", "h4", "h5", "h6", "strong", "em", "pre", "code",
        "blockquote", "small", "a", "img", "ul", "ol", "li", "table", "thead",
        "tbody", "tr", "th", "td", "form", "button", "label", "input", "textarea",
        "select", "option", "script", "style", "br", "hr", "meta", "link",
        "text", "raw_html"
    )

    private val SPECIAL_COMPILER_TAGS = setOf("Style", "Head", "Stylesheet")

    /**
     * Check whether a tag name is a special compiler element.
     */
    fun isSpecialCompilerTag(tagName: String): Boolean = tagName in SPECIAL_COMPILER_TAGS

    /**
     * Check whether a symbol name is a known prelude type/struct/enum/trait/fn.
     */
    fun isKnownPreludeSymbol(name: String): Boolean =
        name in TYPE_TO_DECL_FILE || name in ELEMENT_CONSTRUCTORS

    /**
     * Locate the `src/libs/web/declarations/` directory in the project.
     */
    fun findDeclarationsDir(project: Project): VirtualFile? {
        // Strategy 1: Search for a known declaration file and derive the directory
        val scope = GlobalSearchScope.allScope(project)
        val files = FilenameIndex.getVirtualFilesByName("elements.volki", scope)
        for (f in files) {
            val parent = f.parent ?: continue
            if (parent.name == "declarations") return parent
        }

        // Strategy 2: Search for types.volki as fallback
        val typeFiles = FilenameIndex.getVirtualFilesByName("types.volki", scope)
        for (f in typeFiles) {
            val parent = f.parent ?: continue
            if (parent.name == "declarations") return parent
        }

        return null
    }

    /**
     * Find a symbol (pub fn/struct/enum/type NAME) in the declaration files.
     * Returns the PsiElement at the declaration site, or null.
     */
    fun findSymbolInDeclarations(project: Project, name: String): PsiElement? {
        val declDir = findDeclarationsDir(project) ?: return null
        val psiManager = PsiManager.getInstance(project)

        // If we know which file it's in, search there first
        val targetFile = TYPE_TO_DECL_FILE[name]
        if (targetFile != null) {
            val vf = declDir.findChild(targetFile)
            if (vf != null) {
                val result = searchFileForSymbol(psiManager, vf, name)
                if (result != null) return result
            }
        }

        // Check if it's an element constructor
        if (name in ELEMENT_CONSTRUCTORS) {
            val vf = declDir.findChild("elements.volki")
            if (vf != null) {
                val result = searchFileForSymbol(psiManager, vf, name)
                if (result != null) return result
            }
        }

        // Fallback: search all declaration files
        for (child in declDir.children) {
            if (!child.name.endsWith(".volki")) continue
            val result = searchFileForSymbol(psiManager, child, name)
            if (result != null) return result
        }

        return null
    }

    /**
     * Find a tag's declaration. Resolves:
     *   - Built-in HTML tags -> elements.volki
     *   - Special compiler tags (Style, Head, Stylesheet) -> special.volki
     */
    fun findTagInDeclarations(project: Project, tagName: String): PsiElement? {
        val declDir = findDeclarationsDir(project) ?: return null
        val psiManager = PsiManager.getInstance(project)

        // Special compiler tags
        val specialFile = TAG_TO_DECL_FILE[tagName]
        if (specialFile != null) {
            val vf = declDir.findChild(specialFile)
            if (vf != null) {
                return searchFileForSymbol(psiManager, vf, tagName)
            }
        }

        // Built-in HTML elements -> look for "pub fn tagname()" in elements.volki
        if (VolkiElementRegistry.isBuiltinTag(tagName)) {
            val info = VolkiElementRegistry.getElement(tagName)
            val constructorName = info?.rustConstructor ?: tagName
            val vf = declDir.findChild("elements.volki")
            if (vf != null) {
                return searchFileForSymbol(psiManager, vf, constructorName)
            }
        }

        return null
    }

    /**
     * Search a single file for `pub fn NAME(`, `pub struct NAME`, `pub enum NAME`,
     * `pub type NAME`, or `pub trait NAME`.
     */
    private fun searchFileForSymbol(psiManager: PsiManager, file: VirtualFile, name: String): PsiElement? {
        val psiFile = psiManager.findFile(file) ?: return null
        val text = psiFile.text

        // Search patterns in priority order
        val patterns = listOf(
            "pub fn $name(",
            "pub struct $name",
            "pub enum $name",
            "pub type $name",
            "pub trait $name",
            "pub const $name"
        )

        for (pattern in patterns) {
            val idx = text.indexOf(pattern)
            if (idx >= 0) {
                val target = psiFile.findElementAt(idx)
                if (target != null) return target
            }
        }

        return null
    }
}
