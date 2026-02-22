package com.volki.jetbrains

import com.intellij.openapi.editor.markup.GutterIconRenderer
import java.awt.Color
import java.awt.Component
import java.awt.Graphics
import javax.swing.Icon

class VolkiStyleGutterIcon(
    private val color: Color,
    private val className: String
) : GutterIconRenderer() {

    override fun getIcon(): Icon = ColorIcon(12, color)

    override fun getTooltipText(): String = className

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is VolkiStyleGutterIcon) return false
        return color == other.color && className == other.className
    }

    override fun hashCode(): Int = 31 * color.hashCode() + className.hashCode()

    private class ColorIcon(private val size: Int, private val color: Color) : Icon {
        override fun paintIcon(c: Component?, g: Graphics, x: Int, y: Int) {
            g.color = color
            g.fillRect(x + 1, y + 1, size - 2, size - 2)
            g.color = Color(color.red / 2, color.green / 2, color.blue / 2)
            g.drawRect(x, y, size - 1, size - 1)
        }

        override fun getIconWidth(): Int = size
        override fun getIconHeight(): Int = size
    }
}
