package com.volki.jetbrains

import com.intellij.lang.Language

class VolkiLanguage private constructor() : Language("Volki") {
    companion object {
        @JvmStatic
        val INSTANCE = VolkiLanguage()
    }
}
