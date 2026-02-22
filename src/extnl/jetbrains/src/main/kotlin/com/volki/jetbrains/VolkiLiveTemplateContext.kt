package com.volki.jetbrains

import com.intellij.codeInsight.template.TemplateActionContext
import com.intellij.codeInsight.template.TemplateContextType

class VolkiLiveTemplateContext : TemplateContextType("Volki") {
    override fun isInContext(context: TemplateActionContext): Boolean {
        return context.file is VolkiFile
    }
}
