package com.volki.jetbrains

object VolkiStyleVariants {

    data class ParsedClass(
        val utility: String,
        val variants: List<String>,
        val important: Boolean,
        val isCustom: Boolean
    )

    val VARIANT_PREFIXES = listOf(
        // Responsive
        "sm", "md", "lg", "xl", "2xl",
        // Max responsive
        "max-sm", "max-md", "max-lg", "max-xl", "max-2xl",
        // State
        "hover", "focus", "active", "visited", "disabled",
        "first", "last", "odd", "even",
        "focus-within", "focus-visible", "checked", "required", "empty", "open",
        // Pseudo-elements
        "placeholder", "before", "after", "selection", "marker", "file",
        // Dark mode
        "dark",
        // Group/peer
        "group-hover", "group-focus", "peer-hover", "peer-focus",
        // Media
        "motion-safe", "motion-reduce", "print",
        // Custom passthrough
        "custom"
    )

    private val VALID_VARIANT_SET = VARIANT_PREFIXES.toHashSet()

    fun parse(className: String): ParsedClass {
        val (important, rest) = if (className.startsWith('!')) {
            true to className.substring(1)
        } else {
            false to className
        }

        val parts = splitVariantChain(rest)
        if (parts.size <= 1) {
            return ParsedClass(
                utility = rest,
                variants = emptyList(),
                important = important,
                isCustom = false
            )
        }

        val variantParts = parts.subList(0, parts.size - 1)
        val utility = parts.last()
        var isCustom = false
        val variants = mutableListOf<String>()

        for (prefix in variantParts) {
            if (prefix == "custom") {
                isCustom = true
            } else {
                variants.add(prefix)
            }
        }

        return ParsedClass(
            utility = utility,
            variants = variants,
            important = important,
            isCustom = isCustom
        )
    }

    fun isValidVariant(prefix: String): Boolean {
        if (prefix in VALID_VARIANT_SET) return true
        // min-[...] / max-[...] arbitrary breakpoints
        if (prefix.startsWith("min-[") || prefix.startsWith("max-[")) return true
        // data-[...] / aria-[...]
        if (prefix.startsWith("data-[") || prefix.startsWith("aria-[")) return true
        // supports-[...]
        if (prefix.startsWith("supports-[")) return true
        // group-hover/name, group-focus/name, peer-hover/name, peer-focus/name
        if (prefix.startsWith("group-hover/") || prefix.startsWith("group-focus/")) return true
        if (prefix.startsWith("peer-hover/") || prefix.startsWith("peer-focus/")) return true
        return false
    }

    private fun splitVariantChain(input: String): List<String> {
        val out = mutableListOf<String>()
        var start = 0
        var depth = 0
        for ((i, ch) in input.withIndex()) {
            when (ch) {
                '[' -> depth++
                ']' -> if (depth > 0) depth--
                ':' -> if (depth == 0) {
                    out.add(input.substring(start, i))
                    start = i + 1
                }
            }
        }
        out.add(input.substring(start))
        return out
    }
}
