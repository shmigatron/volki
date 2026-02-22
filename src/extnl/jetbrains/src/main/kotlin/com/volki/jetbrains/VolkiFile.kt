package com.volki.jetbrains

import com.intellij.extapi.psi.PsiFileBase
import com.intellij.openapi.fileTypes.FileType
import com.intellij.psi.FileViewProvider

class VolkiFile(viewProvider: FileViewProvider) : PsiFileBase(viewProvider, VolkiLanguage.INSTANCE) {

    override fun getFileType(): FileType = VolkiFileType.INSTANCE

    override fun toString(): String = "Volki File"
}
