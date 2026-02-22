package com.volki.jetbrains

import com.intellij.icons.AllIcons
import com.intellij.openapi.util.IconLoader
import javax.swing.Icon

object VolkiIcons {
    val FILE: Icon = IconLoader.findIcon("/assets/logo.svg", VolkiIcons::class.java)
        ?: IconLoader.findIcon("/icons/volki_file.svg", VolkiIcons::class.java)
        ?: IconLoader.findIcon("/META-INF/pluginIcon.svg", VolkiIcons::class.java)
        ?: AllIcons.FileTypes.Text
}
