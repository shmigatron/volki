package com.volki.jetbrains

import com.intellij.openapi.editor.ElementColorProvider
import com.intellij.psi.PsiElement
import java.awt.Color

class VolkiStyleColorProvider : ElementColorProvider {

    override fun getColorFrom(element: PsiElement): Color? {
        if (element.node?.elementType != VolkiTokenTypes.STRING) return null
        if (!VolkiStyleClassContext.isClassAttributeValue(element)) return null

        val spans = VolkiStyleClassContext.extractClassNames(element)
        for (span in spans) {
            val parsed = VolkiStyleVariants.parse(span.text)
            val colorName = VolkiStyleResolver.extractColorName(parsed.utility) ?: continue
            val color = VolkiStylePalette.colorToAwtColor(colorName) ?: continue
            return color
        }
        return null
    }

    override fun setColorTo(element: PsiElement, color: Color) {
        // Find the nearest palette color by RGB distance
        var bestName: String? = null
        var bestDist = Int.MAX_VALUE

        for (family in VolkiStylePalette.COLOR_FAMILIES) {
            for (shade in VolkiStylePalette.SHADES) {
                val name = "$family-$shade"
                val c = VolkiStylePalette.colorToAwtColor(name) ?: continue
                val dr = c.red - color.red
                val dg = c.green - color.green
                val db = c.blue - color.blue
                val dist = dr * dr + dg * dg + db * db
                if (dist < bestDist) {
                    bestDist = dist
                    bestName = name
                }
            }
        }
        // Also check white/black
        for (name in listOf("white", "black")) {
            val c = VolkiStylePalette.colorToAwtColor(name) ?: continue
            val dr = c.red - color.red
            val dg = c.green - color.green
            val db = c.blue - color.blue
            val dist = dr * dr + dg * dg + db * db
            if (dist < bestDist) {
                bestDist = dist
                bestName = name
            }
        }

        if (bestName == null) return

        // Find which class in the string uses a color and replace it
        val spans = VolkiStyleClassContext.extractClassNames(element)

        for (span in spans) {
            val parsed = VolkiStyleVariants.parse(span.text)
            val oldColor = VolkiStyleResolver.extractColorName(parsed.utility) ?: continue

            // Build the new class name by replacing the color portion
            val utility = parsed.utility
            val newUtility = utility.replace(oldColor, bestName)
            val newClassName = if (parsed.variants.isNotEmpty()) {
                parsed.variants.joinToString(":") + ":" + newUtility
            } else {
                newUtility
            }

            val doc = element.containingFile?.viewProvider?.document ?: return
            doc.replaceString(span.startOffset, span.endOffset, newClassName)
            return
        }
    }
}
