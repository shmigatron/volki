package com.volki.jetbrains

import com.intellij.ide.FileIconProvider
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import javax.swing.Icon

class VolkiFileIconProvider : FileIconProvider {
    override fun getIcon(file: VirtualFile, flags: Int, project: Project?): Icon? {
        if (file.isDirectory && file.name == ".volki") {
            return VolkiIcons.FILE
        }
        if (!file.isDirectory && file.name == ".volki") {
            return VolkiIcons.FILE
        }
        if (!file.isDirectory && file.extension == "volki") {
            return VolkiIcons.FILE
        }
        return null
    }
}
