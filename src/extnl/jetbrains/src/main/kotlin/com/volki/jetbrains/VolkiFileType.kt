package com.volki.jetbrains

import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

class VolkiFileType private constructor() : LanguageFileType(VolkiLanguage.INSTANCE) {

    override fun getName(): String = "Volki"

    override fun getDescription(): String = "Volki language file"

    override fun getDefaultExtension(): String = "volki"

    override fun getIcon(): Icon = VolkiIcons.FILE

    companion object {
        @JvmStatic
        val INSTANCE = VolkiFileType()
    }
}
