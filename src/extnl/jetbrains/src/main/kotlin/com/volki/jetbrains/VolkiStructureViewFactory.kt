package com.volki.jetbrains

import com.intellij.ide.structureView.*
import com.intellij.ide.util.treeView.smartTree.TreeElement
import com.intellij.lang.PsiStructureViewFactory
import com.intellij.navigation.ItemPresentation
import com.intellij.openapi.editor.Editor
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.TokenType
import javax.swing.Icon

class VolkiStructureViewFactory : PsiStructureViewFactory {
    override fun getStructureViewBuilder(psiFile: PsiFile): StructureViewBuilder {
        return object : TreeBasedStructureViewBuilder() {
            override fun createStructureViewModel(editor: Editor?): StructureViewModel {
                return VolkiStructureViewModel(psiFile)
            }
        }
    }
}

private class VolkiStructureViewModel(psiFile: PsiFile) :
    StructureViewModelBase(psiFile, VolkiStructureViewElement(psiFile)),
    StructureViewModel.ElementInfoProvider {

    override fun isAlwaysShowsPlus(element: StructureViewTreeElement): Boolean = false
    override fun isAlwaysLeaf(element: StructureViewTreeElement): Boolean = false
}

class VolkiStructureViewElement(private val element: PsiElement) : StructureViewTreeElement {

    override fun getValue(): Any = element

    override fun navigate(requestFocus: Boolean) {
        if (element is com.intellij.pom.Navigatable) {
            element.navigate(requestFocus)
        }
    }

    override fun canNavigate(): Boolean =
        element is com.intellij.pom.Navigatable && (element as com.intellij.pom.Navigatable).canNavigate()

    override fun canNavigateToSource(): Boolean = canNavigate()

    override fun getPresentation(): ItemPresentation {
        return when {
            element is PsiFile -> object : ItemPresentation {
                override fun getPresentableText(): String = element.name
                override fun getLocationString(): String? = null
                override fun getIcon(unused: Boolean): Icon = VolkiIcons.FILE
            }
            else -> {
                val info = element.getUserData(STRUCTURE_INFO_KEY)
                object : ItemPresentation {
                    override fun getPresentableText(): String = info?.name ?: element.text
                    override fun getLocationString(): String? = info?.detail
                    override fun getIcon(unused: Boolean): Icon = VolkiIcons.FILE
                }
            }
        }
    }

    override fun getChildren(): Array<TreeElement> {
        if (element !is PsiFile) return emptyArray()

        val result = mutableListOf<TreeElement>()
        val items = extractStructureItems(element)
        for (item in items) {
            val el = VolkiStructureViewElement(item.element)
            result.add(el)
        }
        return result.toTypedArray()
    }

    private fun extractStructureItems(file: PsiFile): List<StructureItem> {
        val items = mutableListOf<StructureItem>()
        var child: PsiElement? = file.firstChild

        while (child != null) {
            val type = child.node?.elementType

            if (type == VolkiTokenTypes.KEYWORD) {
                when (child.text) {
                    "fn" -> {
                        val fnItem = parseFunctionDecl(child)
                        if (fnItem != null) items.add(fnItem)
                    }
                    "struct" -> {
                        val structItem = parseStructDecl(child)
                        if (structItem != null) items.add(structItem)
                    }
                    "enum" -> {
                        val enumItem = parseEnumDecl(child)
                        if (enumItem != null) items.add(enumItem)
                    }
                    "impl" -> {
                        val implItem = parseImplDecl(child)
                        if (implItem != null) items.add(implItem)
                    }
                    "use" -> {
                        val useItem = parseUseDecl(child)
                        if (useItem != null) items.add(useItem)
                    }
                    "pub" -> {
                        // Check what follows pub
                        val next = nextNonWs(child)
                        if (next?.node?.elementType == VolkiTokenTypes.KEYWORD) {
                            when (next.text) {
                                "fn" -> {
                                    val fnItem = parseFunctionDecl(next)
                                    if (fnItem != null) {
                                        items.add(fnItem.copy(kind = "pub ${fnItem.kind}"))
                                    }
                                }
                                "struct" -> {
                                    val item = parseStructDecl(next)
                                    if (item != null) items.add(item)
                                }
                                "enum" -> {
                                    val item = parseEnumDecl(next)
                                    if (item != null) items.add(item)
                                }
                            }
                        }
                    }
                }
            }

            child = child.nextSibling
        }

        return items
    }

    private fun parseFunctionDecl(fnKeyword: PsiElement): StructureItem? {
        val nameEl = nextNonWs(fnKeyword) ?: return null
        val name = nameEl.text
        if (name == "fn") return null // double fn somehow

        // Try to extract params and return type
        var cur = nameEl.nextSibling
        val sig = StringBuilder("fn $name")
        while (cur != null) {
            val t = cur.node?.elementType
            if (t == VolkiTokenTypes.BRACE_OPEN) break // function body starts
            if (t == VolkiTokenTypes.TAG_BRACKET) break // RSX starts
            if (cur.text == "\n" && sig.contains("->")) break
            sig.append(cur.text)
            cur = cur.nextSibling
        }

        // Check if it's a component (returns Html)
        val isComponent = name.isNotEmpty() && name[0].isUpperCase()

        nameEl.putUserData(STRUCTURE_INFO_KEY, StructureInfo(
            name = name,
            detail = if (isComponent) "component" else "fn"
        ))

        return StructureItem(nameEl, name, if (isComponent) "component" else "fn")
    }

    private fun parseStructDecl(keyword: PsiElement): StructureItem? {
        val nameEl = nextNonWs(keyword) ?: return null
        val name = nameEl.text
        nameEl.putUserData(STRUCTURE_INFO_KEY, StructureInfo(name, "struct"))
        return StructureItem(nameEl, name, "struct")
    }

    private fun parseEnumDecl(keyword: PsiElement): StructureItem? {
        val nameEl = nextNonWs(keyword) ?: return null
        val name = nameEl.text
        nameEl.putUserData(STRUCTURE_INFO_KEY, StructureInfo(name, "enum"))
        return StructureItem(nameEl, name, "enum")
    }

    private fun parseImplDecl(keyword: PsiElement): StructureItem? {
        val nameEl = nextNonWs(keyword) ?: return null
        val name = nameEl.text
        nameEl.putUserData(STRUCTURE_INFO_KEY, StructureInfo("impl $name", "impl"))
        return StructureItem(nameEl, "impl $name", "impl")
    }

    private fun parseUseDecl(keyword: PsiElement): StructureItem? {
        val sb = StringBuilder()
        var cur = nextNonWs(keyword)
        while (cur != null) {
            val text = cur.text
            if (text == ";" || text == "\n") break
            if (cur.node?.elementType != TokenType.WHITE_SPACE) sb.append(text)
            cur = cur.nextSibling
        }
        if (sb.isEmpty()) return null
        val useText = sb.toString()
        keyword.putUserData(STRUCTURE_INFO_KEY, StructureInfo("use $useText", "use"))
        return StructureItem(keyword, "use $useText", "use")
    }

    private fun nextNonWs(el: PsiElement): PsiElement? {
        var cur = el.nextSibling
        while (cur != null && cur.node?.elementType == TokenType.WHITE_SPACE) cur = cur.nextSibling
        return cur
    }
}

private data class StructureItem(val element: PsiElement, val name: String, val kind: String)

data class StructureInfo(val name: String, val detail: String?)

private val STRUCTURE_INFO_KEY = com.intellij.openapi.util.Key.create<StructureInfo>("VOLKI_STRUCTURE_INFO")
