package com.volki.jetbrains

object VolkiStyleResolver {

    sealed class ResolvedUtility {
        data class Standard(val declarations: String) : ResolvedUtility()
        data class Custom(val selectorSuffix: String, val declarations: String) : ResolvedUtility()

        fun declarationsText(): String = when (this) {
            is Standard -> declarations
            is Custom -> declarations
        }
    }

    fun resolve(className: String): ResolvedUtility? {
        resolveLayout(className)?.let { return it }
        resolveFlexbox(className)?.let { return it }
        resolveGrid(className)?.let { return it }
        resolveSpacing(className)?.let { return it }
        resolveSizing(className)?.let { return it }
        resolveTypography(className)?.let { return it }
        resolveBackgrounds(className)?.let { return it }
        resolveBorders(className)?.let { return it }
        resolveEffects(className)?.let { return it }
        resolveTransforms(className)?.let { return it }
        resolveFilters(className)?.let { return it }
        resolveTransitions(className)?.let { return it }
        resolveInteractivity(className)?.let { return it }
        resolveTables(className)?.let { return it }
        resolveSvg(className)?.let { return it }
        resolveInset(className)?.let { return it }
        return null
    }

    fun category(className: String): String? {
        if (resolveLayout(className) != null) return "Layout"
        if (resolveFlexbox(className) != null) return "Flexbox & Grid"
        if (resolveGrid(className) != null) return "Grid"
        if (resolveSpacing(className) != null) return "Spacing"
        if (resolveSizing(className) != null) return "Sizing"
        if (resolveTypography(className) != null) return "Typography"
        if (resolveBackgrounds(className) != null) return "Backgrounds"
        if (resolveBorders(className) != null) return "Borders"
        if (resolveEffects(className) != null) return "Effects"
        if (resolveTransforms(className) != null) return "Transforms"
        if (resolveFilters(className) != null) return "Filters"
        if (resolveTransitions(className) != null) return "Transitions & Animation"
        if (resolveInteractivity(className) != null) return "Interactivity"
        if (resolveTables(className) != null) return "Tables"
        if (resolveSvg(className) != null) return "SVG"
        if (resolveInset(className) != null) return "Positioning"
        return null
    }

    fun extractColorName(className: String): String? {
        val prefixes = listOf("bg-", "text-", "border-", "ring-", "shadow-", "fill-", "stroke-",
            "accent-", "caret-", "decoration-", "from-", "via-", "to-",
            "border-t-", "border-r-", "border-b-", "border-l-",
            "outline-", "divide-", "ring-offset-")
        for (prefix in prefixes) {
            if (className.startsWith(prefix)) {
                val rest = className.removePrefix(prefix)
                val colorPart = rest.substringBefore('/')
                if (VolkiStylePalette.colorHex(colorPart) != null) return colorPart
            }
        }
        return null
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    private fun spacing(n: Int): String {
        if (n == 0) return "0px"
        val whole = n / 4
        return when (n % 4) {
            0 -> "${whole}rem"
            1 -> "${whole}.25rem"
            2 -> "${whole}.5rem"
            3 -> "${whole}.75rem"
            else -> "${whole}rem"
        }
    }

    private fun parseU32(s: String): Int? {
        if (s.isEmpty()) return null
        var n = 0
        for (ch in s) {
            if (ch !in '0'..'9') return null
            n = n * 10 + (ch - '0')
        }
        return n
    }

    private fun parseFractionalSpacing(s: String): String? {
        val dot = s.indexOf('.').takeIf { it >= 0 } ?: return null
        val wholePart = s.substring(0, dot)
        val fracPart = s.substring(dot + 1)
        if (fracPart != "5") return null
        val whole = parseU32(wholePart) ?: return null
        val v = whole * 2 + 1
        return "0.${v * 125}rem"
    }

    private fun parseFraction(s: String): String? {
        val slash = s.indexOf('/').takeIf { it >= 0 } ?: return null
        val num = parseU32(s.substring(0, slash)) ?: return null
        val den = parseU32(s.substring(slash + 1)) ?: return null
        if (den == 0 || num > den) return null
        if (num == den) return "100%"
        return when (num to den) {
            1 to 2 -> "50%"
            1 to 3 -> "33.333333%"; 2 to 3 -> "66.666667%"
            1 to 4 -> "25%"; 2 to 4 -> "50%"; 3 to 4 -> "75%"
            1 to 5 -> "20%"; 2 to 5 -> "40%"; 3 to 5 -> "60%"; 4 to 5 -> "80%"
            1 to 6 -> "16.666667%"; 2 to 6 -> "33.333333%"; 3 to 6 -> "50%"
            4 to 6 -> "66.666667%"; 5 to 6 -> "83.333333%"
            1 to 12 -> "8.333333%"; 2 to 12 -> "16.666667%"; 3 to 12 -> "25%"
            4 to 12 -> "33.333333%"; 5 to 12 -> "41.666667%"; 6 to 12 -> "50%"
            7 to 12 -> "58.333333%"; 8 to 12 -> "66.666667%"; 9 to 12 -> "75%"
            10 to 12 -> "83.333333%"; 11 to 12 -> "91.666667%"
            else -> null
        }
    }

    private fun parseArbitrary(s: String): String? {
        if (s.startsWith('[') && s.endsWith(']') && s.length > 2) {
            return s.substring(1, s.length - 1)
        }
        return null
    }

    private fun resolveColorWithOpacity(colorPart: String, property: String): String? {
        val slashPos = colorPart.indexOf('/')
        if (slashPos >= 0) {
            val colorName = colorPart.substring(0, slashPos)
            val opacityStr = colorPart.substring(slashPos + 1)
            val opacityVal = parseU32(opacityStr) ?: return null
            if (opacityVal > 100) return null
            val hex = VolkiStylePalette.colorHex(colorName) ?: return null
            if (hex == "transparent") return "$property:transparent;"
            val rgb = hexToRgb(hex) ?: return null
            val alpha = when {
                opacityVal == 100 -> "1"
                opacityVal == 0 -> "0"
                opacityVal % 10 == 0 -> "0.${opacityVal / 10}"
                else -> "0.$opacityVal"
            }
            return "$property:rgb(${rgb.first} ${rgb.second} ${rgb.third} / $alpha);"
        } else {
            val hex = VolkiStylePalette.colorHex(colorPart) ?: return null
            return "$property:$hex;"
        }
    }

    private fun hexToRgb(hex: String): Triple<Int, Int, Int>? {
        if (!hex.startsWith('#') || hex.length != 7) return null
        val r = hex.substring(1, 3).toIntOrNull(16) ?: return null
        val g = hex.substring(3, 5).toIntOrNull(16) ?: return null
        val b = hex.substring(5, 7).toIntOrNull(16) ?: return null
        return Triple(r, g, b)
    }

    private fun parseSpacingValue(s: String): String? {
        parseU32(s)?.let { return spacing(it) }
        parseFractionalSpacing(s)?.let { return it }
        parseArbitrary(s)?.let { return it }
        return null
    }

    // ── Layout ──────────────────────────────────────────────────────────────

    private fun resolveLayout(c: String): ResolvedUtility? {
        val decls = when (c) {
            "block" -> "display:block;"
            "inline" -> "display:inline;"
            "inline-block" -> "display:inline-block;"
            "flex" -> "display:flex;"
            "inline-flex" -> "display:inline-flex;"
            "grid" -> "display:grid;"
            "inline-grid" -> "display:inline-grid;"
            "hidden" -> "display:none;"
            "table" -> "display:table;"
            "table-row" -> "display:table-row;"
            "table-cell" -> "display:table-cell;"
            "table-caption" -> "display:table-caption;"
            "table-column" -> "display:table-column;"
            "table-column-group" -> "display:table-column-group;"
            "table-footer-group" -> "display:table-footer-group;"
            "table-header-group" -> "display:table-header-group;"
            "table-row-group" -> "display:table-row-group;"
            "contents" -> "display:contents;"
            "list-item" -> "display:list-item;"
            "flow-root" -> "display:flow-root;"
            "container" -> "width:100%;"
            "relative" -> "position:relative;"
            "absolute" -> "position:absolute;"
            "fixed" -> "position:fixed;"
            "sticky" -> "position:sticky;"
            "static" -> "position:static;"
            "float-right" -> "float:right;"
            "float-left" -> "float:left;"
            "float-none" -> "float:none;"
            "clear-left" -> "clear:left;"
            "clear-right" -> "clear:right;"
            "clear-both" -> "clear:both;"
            "clear-none" -> "clear:none;"
            "visible" -> "visibility:visible;"
            "invisible" -> "visibility:hidden;"
            "collapse" -> "visibility:collapse;"
            "box-border" -> "box-sizing:border-box;"
            "box-content" -> "box-sizing:content-box;"
            "isolate" -> "isolation:isolate;"
            "isolation-auto" -> "isolation:auto;"
            "aspect-auto" -> "aspect-ratio:auto;"
            "aspect-square" -> "aspect-ratio:1 / 1;"
            "aspect-video" -> "aspect-ratio:16 / 9;"
            "object-contain" -> "object-fit:contain;"
            "object-cover" -> "object-fit:cover;"
            "object-fill" -> "object-fit:fill;"
            "object-none" -> "object-fit:none;"
            "object-scale-down" -> "object-fit:scale-down;"
            "object-bottom" -> "object-position:bottom;"
            "object-center" -> "object-position:center;"
            "object-left" -> "object-position:left;"
            "object-left-bottom" -> "object-position:left bottom;"
            "object-left-top" -> "object-position:left top;"
            "object-right" -> "object-position:right;"
            "object-right-bottom" -> "object-position:right bottom;"
            "object-right-top" -> "object-position:right top;"
            "object-top" -> "object-position:top;"
            "overflow-hidden" -> "overflow:hidden;"
            "overflow-auto" -> "overflow:auto;"
            "overflow-scroll" -> "overflow:scroll;"
            "overflow-visible" -> "overflow:visible;"
            "overflow-clip" -> "overflow:clip;"
            "overflow-x-auto" -> "overflow-x:auto;"
            "overflow-y-auto" -> "overflow-y:auto;"
            "overflow-x-hidden" -> "overflow-x:hidden;"
            "overflow-y-hidden" -> "overflow-y:hidden;"
            "overflow-x-clip" -> "overflow-x:clip;"
            "overflow-y-clip" -> "overflow-y:clip;"
            "overflow-x-visible" -> "overflow-x:visible;"
            "overflow-y-visible" -> "overflow-y:visible;"
            "overflow-x-scroll" -> "overflow-x:scroll;"
            "overflow-y-scroll" -> "overflow-y:scroll;"
            "overscroll-auto" -> "overscroll-behavior:auto;"
            "overscroll-contain" -> "overscroll-behavior:contain;"
            "overscroll-none" -> "overscroll-behavior:none;"
            "overscroll-x-auto" -> "overscroll-behavior-x:auto;"
            "overscroll-x-contain" -> "overscroll-behavior-x:contain;"
            "overscroll-x-none" -> "overscroll-behavior-x:none;"
            "overscroll-y-auto" -> "overscroll-behavior-y:auto;"
            "overscroll-y-contain" -> "overscroll-behavior-y:contain;"
            "overscroll-y-none" -> "overscroll-behavior-y:none;"
            "break-after-auto" -> "break-after:auto;"
            "break-after-avoid" -> "break-after:avoid;"
            "break-after-all" -> "break-after:all;"
            "break-after-avoid-page" -> "break-after:avoid-page;"
            "break-after-page" -> "break-after:page;"
            "break-after-left" -> "break-after:left;"
            "break-after-right" -> "break-after:right;"
            "break-after-column" -> "break-after:column;"
            "break-before-auto" -> "break-before:auto;"
            "break-before-avoid" -> "break-before:avoid;"
            "break-before-all" -> "break-before:all;"
            "break-before-avoid-page" -> "break-before:avoid-page;"
            "break-before-page" -> "break-before:page;"
            "break-before-left" -> "break-before:left;"
            "break-before-right" -> "break-before:right;"
            "break-before-column" -> "break-before:column;"
            "break-inside-auto" -> "break-inside:auto;"
            "break-inside-avoid" -> "break-inside:avoid;"
            "break-inside-avoid-page" -> "break-inside:avoid-page;"
            "break-inside-avoid-column" -> "break-inside:avoid-column;"
            "sr-only" -> "position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border-width:0;"
            "not-sr-only" -> "position:static;width:auto;height:auto;padding:0;margin:0;overflow:visible;clip:auto;white-space:normal;"
            else -> {
                if (c.startsWith("columns-")) {
                    val rest = c.removePrefix("columns-")
                    val d = when (rest) {
                        "auto" -> "columns:auto;"
                        "3xs" -> "columns:16rem;"; "2xs" -> "columns:18rem;"
                        "xs" -> "columns:20rem;"; "sm" -> "columns:24rem;"
                        "md" -> "columns:28rem;"; "lg" -> "columns:32rem;"
                        "xl" -> "columns:36rem;"; "2xl" -> "columns:42rem;"
                        "3xl" -> "columns:48rem;"; "4xl" -> "columns:56rem;"
                        "5xl" -> "columns:64rem;"; "6xl" -> "columns:72rem;"
                        "7xl" -> "columns:80rem;"
                        else -> {
                            val n = parseU32(rest) ?: return null
                            if (n < 1 || n > 12) return null
                            "columns:$n;"
                        }
                    }
                    return ResolvedUtility.Standard(d)
                }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    // ── Flexbox ─────────────────────────────────────────────────────────────

    private fun resolveFlexbox(c: String): ResolvedUtility? {
        val decls = when (c) {
            "flex-row" -> "flex-direction:row;"
            "flex-col" -> "flex-direction:column;"
            "flex-row-reverse" -> "flex-direction:row-reverse;"
            "flex-col-reverse" -> "flex-direction:column-reverse;"
            "flex-wrap" -> "flex-wrap:wrap;"
            "flex-nowrap" -> "flex-wrap:nowrap;"
            "flex-wrap-reverse" -> "flex-wrap:wrap-reverse;"
            "flex-1" -> "flex:1 1 0%;"
            "flex-auto" -> "flex:1 1 auto;"
            "flex-initial" -> "flex:0 1 auto;"
            "flex-none" -> "flex:none;"
            "flex-grow" -> "flex-grow:1;"
            "flex-grow-0" -> "flex-grow:0;"
            "flex-shrink" -> "flex-shrink:1;"
            "flex-shrink-0" -> "flex-shrink:0;"
            "items-center" -> "align-items:center;"
            "items-start" -> "align-items:flex-start;"
            "items-end" -> "align-items:flex-end;"
            "items-stretch" -> "align-items:stretch;"
            "items-baseline" -> "align-items:baseline;"
            "justify-center" -> "justify-content:center;"
            "justify-between" -> "justify-content:space-between;"
            "justify-around" -> "justify-content:space-around;"
            "justify-evenly" -> "justify-content:space-evenly;"
            "justify-start" -> "justify-content:flex-start;"
            "justify-end" -> "justify-content:flex-end;"
            "justify-normal" -> "justify-content:normal;"
            "justify-stretch" -> "justify-content:stretch;"
            "justify-items-start" -> "justify-items:start;"
            "justify-items-end" -> "justify-items:end;"
            "justify-items-center" -> "justify-items:center;"
            "justify-items-stretch" -> "justify-items:stretch;"
            "justify-self-auto" -> "justify-self:auto;"
            "justify-self-start" -> "justify-self:start;"
            "justify-self-end" -> "justify-self:end;"
            "justify-self-center" -> "justify-self:center;"
            "justify-self-stretch" -> "justify-self:stretch;"
            "content-normal" -> "align-content:normal;"
            "content-center" -> "align-content:center;"
            "content-start" -> "align-content:flex-start;"
            "content-end" -> "align-content:flex-end;"
            "content-between" -> "align-content:space-between;"
            "content-around" -> "align-content:space-around;"
            "content-evenly" -> "align-content:space-evenly;"
            "content-baseline" -> "align-content:baseline;"
            "content-stretch" -> "align-content:stretch;"
            "self-auto" -> "align-self:auto;"
            "self-start" -> "align-self:flex-start;"
            "self-end" -> "align-self:flex-end;"
            "self-center" -> "align-self:center;"
            "self-stretch" -> "align-self:stretch;"
            "self-baseline" -> "align-self:baseline;"
            "place-content-center" -> "place-content:center;"
            "place-content-start" -> "place-content:start;"
            "place-content-end" -> "place-content:end;"
            "place-content-between" -> "place-content:space-between;"
            "place-content-around" -> "place-content:space-around;"
            "place-content-evenly" -> "place-content:space-evenly;"
            "place-content-baseline" -> "place-content:baseline;"
            "place-content-stretch" -> "place-content:stretch;"
            "place-items-start" -> "place-items:start;"
            "place-items-end" -> "place-items:end;"
            "place-items-center" -> "place-items:center;"
            "place-items-baseline" -> "place-items:baseline;"
            "place-items-stretch" -> "place-items:stretch;"
            "place-self-auto" -> "place-self:auto;"
            "place-self-start" -> "place-self:start;"
            "place-self-end" -> "place-self:end;"
            "place-self-center" -> "place-self:center;"
            "place-self-stretch" -> "place-self:stretch;"
            else -> {
                if (c.startsWith("grow-")) { val n = parseU32(c.removePrefix("grow-")) ?: return null; return ResolvedUtility.Standard("flex-grow:$n;") }
                if (c.startsWith("shrink-")) { val n = parseU32(c.removePrefix("shrink-")) ?: return null; return ResolvedUtility.Standard("flex-shrink:$n;") }
                if (c.startsWith("order-")) {
                    val rest = c.removePrefix("order-")
                    val d = when (rest) {
                        "first" -> "order:-9999;"; "last" -> "order:9999;"; "none" -> "order:0;"
                        else -> { val n = parseU32(rest) ?: return null; "order:$n;" }
                    }
                    return ResolvedUtility.Standard(d)
                }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    // ── Grid ────────────────────────────────────────────────────────────────

    private fun resolveGrid(c: String): ResolvedUtility? {
        val decls = when (c) {
            "grid-flow-row" -> "grid-auto-flow:row;"
            "grid-flow-col" -> "grid-auto-flow:column;"
            "grid-flow-dense" -> "grid-auto-flow:dense;"
            "grid-flow-row-dense" -> "grid-auto-flow:row dense;"
            "grid-flow-col-dense" -> "grid-auto-flow:column dense;"
            "auto-cols-auto" -> "grid-auto-columns:auto;"
            "auto-cols-min" -> "grid-auto-columns:min-content;"
            "auto-cols-max" -> "grid-auto-columns:max-content;"
            "auto-cols-fr" -> "grid-auto-columns:minmax(0,1fr);"
            "auto-rows-auto" -> "grid-auto-rows:auto;"
            "auto-rows-min" -> "grid-auto-rows:min-content;"
            "auto-rows-max" -> "grid-auto-rows:max-content;"
            "auto-rows-fr" -> "grid-auto-rows:minmax(0,1fr);"
            "col-auto" -> "grid-column:auto;"
            "col-span-full" -> "grid-column:1 / -1;"
            "row-auto" -> "grid-row:auto;"
            "row-span-full" -> "grid-row:1 / -1;"
            else -> {
                if (c.startsWith("grid-cols-")) {
                    val rest = c.removePrefix("grid-cols-")
                    val d = when (rest) {
                        "none" -> "grid-template-columns:none;"; "subgrid" -> "grid-template-columns:subgrid;"
                        else -> { val n = parseU32(rest) ?: return null; if (n < 1 || n > 12) return null; "grid-template-columns:repeat($n,minmax(0,1fr));" }
                    }
                    return ResolvedUtility.Standard(d)
                }
                if (c.startsWith("grid-rows-")) {
                    val rest = c.removePrefix("grid-rows-")
                    val d = when (rest) {
                        "none" -> "grid-template-rows:none;"; "subgrid" -> "grid-template-rows:subgrid;"
                        else -> { val n = parseU32(rest) ?: return null; if (n < 1 || n > 12) return null; "grid-template-rows:repeat($n,minmax(0,1fr));" }
                    }
                    return ResolvedUtility.Standard(d)
                }
                if (c.startsWith("col-span-")) { val n = parseU32(c.removePrefix("col-span-")) ?: return null; if (n < 1 || n > 12) return null; return ResolvedUtility.Standard("grid-column:span $n / span $n;") }
                if (c.startsWith("col-start-")) { val n = parseU32(c.removePrefix("col-start-")) ?: return null; return ResolvedUtility.Standard("grid-column-start:$n;") }
                if (c.startsWith("col-end-")) { val n = parseU32(c.removePrefix("col-end-")) ?: return null; return ResolvedUtility.Standard("grid-column-end:$n;") }
                if (c.startsWith("row-span-")) { val n = parseU32(c.removePrefix("row-span-")) ?: return null; if (n < 1 || n > 12) return null; return ResolvedUtility.Standard("grid-row:span $n / span $n;") }
                if (c.startsWith("row-start-")) { val n = parseU32(c.removePrefix("row-start-")) ?: return null; return ResolvedUtility.Standard("grid-row-start:$n;") }
                if (c.startsWith("row-end-")) { val n = parseU32(c.removePrefix("row-end-")) ?: return null; return ResolvedUtility.Standard("grid-row-end:$n;") }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    // ── Spacing ─────────────────────────────────────────────────────────────

    private fun resolveSpacing(c: String): ResolvedUtility? {
        val childSuffix = ">:not([hidden])~:not([hidden])"
        if (c == "space-x-reverse") return ResolvedUtility.Custom(childSuffix, "--tw-space-x-reverse:1;")
        if (c == "space-y-reverse") return ResolvedUtility.Custom(childSuffix, "--tw-space-y-reverse:1;")
        if (c.startsWith("space-x-")) { val v = parseSpacingValue(c.removePrefix("space-x-")) ?: return null; return ResolvedUtility.Custom(childSuffix, "margin-left:$v;") }
        if (c.startsWith("space-y-")) { val v = parseSpacingValue(c.removePrefix("space-y-")) ?: return null; return ResolvedUtility.Custom(childSuffix, "margin-top:$v;") }
        // Negative margins
        if (c.startsWith("-mx-")) { val v = parseSpacingValue(c.removePrefix("-mx-")) ?: return null; return ResolvedUtility.Standard("margin-left:-$v;margin-right:-$v;") }
        if (c.startsWith("-my-")) { val v = parseSpacingValue(c.removePrefix("-my-")) ?: return null; return ResolvedUtility.Standard("margin-top:-$v;margin-bottom:-$v;") }
        if (c.startsWith("-mt-")) { val v = parseSpacingValue(c.removePrefix("-mt-")) ?: return null; return ResolvedUtility.Standard("margin-top:-$v;") }
        if (c.startsWith("-mr-")) { val v = parseSpacingValue(c.removePrefix("-mr-")) ?: return null; return ResolvedUtility.Standard("margin-right:-$v;") }
        if (c.startsWith("-mb-")) { val v = parseSpacingValue(c.removePrefix("-mb-")) ?: return null; return ResolvedUtility.Standard("margin-bottom:-$v;") }
        if (c.startsWith("-ml-")) { val v = parseSpacingValue(c.removePrefix("-ml-")) ?: return null; return ResolvedUtility.Standard("margin-left:-$v;") }
        if (c.startsWith("-ms-")) { val v = parseSpacingValue(c.removePrefix("-ms-")) ?: return null; return ResolvedUtility.Standard("margin-inline-start:-$v;") }
        if (c.startsWith("-me-")) { val v = parseSpacingValue(c.removePrefix("-me-")) ?: return null; return ResolvedUtility.Standard("margin-inline-end:-$v;") }
        if (c.startsWith("-m-")) { val v = parseSpacingValue(c.removePrefix("-m-")) ?: return null; return ResolvedUtility.Standard("margin:-$v;") }
        // Padding
        if (c.startsWith("px-")) { val v = parseSpacingValue(c.removePrefix("px-")) ?: return null; return ResolvedUtility.Standard("padding-left:$v;padding-right:$v;") }
        if (c.startsWith("py-")) { val v = parseSpacingValue(c.removePrefix("py-")) ?: return null; return ResolvedUtility.Standard("padding-top:$v;padding-bottom:$v;") }
        if (c.startsWith("pt-")) { val v = parseSpacingValue(c.removePrefix("pt-")) ?: return null; return ResolvedUtility.Standard("padding-top:$v;") }
        if (c.startsWith("pr-")) { val v = parseSpacingValue(c.removePrefix("pr-")) ?: return null; return ResolvedUtility.Standard("padding-right:$v;") }
        if (c.startsWith("pb-")) { val v = parseSpacingValue(c.removePrefix("pb-")) ?: return null; return ResolvedUtility.Standard("padding-bottom:$v;") }
        if (c.startsWith("pl-")) { val v = parseSpacingValue(c.removePrefix("pl-")) ?: return null; return ResolvedUtility.Standard("padding-left:$v;") }
        if (c.startsWith("ps-")) { val v = parseSpacingValue(c.removePrefix("ps-")) ?: return null; return ResolvedUtility.Standard("padding-inline-start:$v;") }
        if (c.startsWith("pe-")) { val v = parseSpacingValue(c.removePrefix("pe-")) ?: return null; return ResolvedUtility.Standard("padding-inline-end:$v;") }
        if (c.startsWith("p-")) { val v = parseSpacingValue(c.removePrefix("p-")) ?: return null; return ResolvedUtility.Standard("padding:$v;") }
        // Margin
        if (c.startsWith("mx-")) { val r = c.removePrefix("mx-"); if (r == "auto") return ResolvedUtility.Standard("margin-left:auto;margin-right:auto;"); val v = parseSpacingValue(r) ?: return null; return ResolvedUtility.Standard("margin-left:$v;margin-right:$v;") }
        if (c.startsWith("my-")) { val r = c.removePrefix("my-"); if (r == "auto") return ResolvedUtility.Standard("margin-top:auto;margin-bottom:auto;"); val v = parseSpacingValue(r) ?: return null; return ResolvedUtility.Standard("margin-top:$v;margin-bottom:$v;") }
        if (c.startsWith("mt-")) { val r = c.removePrefix("mt-"); if (r == "auto") return ResolvedUtility.Standard("margin-top:auto;"); val v = parseSpacingValue(r) ?: return null; return ResolvedUtility.Standard("margin-top:$v;") }
        if (c.startsWith("mr-")) { val r = c.removePrefix("mr-"); if (r == "auto") return ResolvedUtility.Standard("margin-right:auto;"); val v = parseSpacingValue(r) ?: return null; return ResolvedUtility.Standard("margin-right:$v;") }
        if (c.startsWith("mb-")) { val r = c.removePrefix("mb-"); if (r == "auto") return ResolvedUtility.Standard("margin-bottom:auto;"); val v = parseSpacingValue(r) ?: return null; return ResolvedUtility.Standard("margin-bottom:$v;") }
        if (c.startsWith("ml-")) { val r = c.removePrefix("ml-"); if (r == "auto") return ResolvedUtility.Standard("margin-left:auto;"); val v = parseSpacingValue(r) ?: return null; return ResolvedUtility.Standard("margin-left:$v;") }
        if (c.startsWith("ms-")) { val r = c.removePrefix("ms-"); if (r == "auto") return ResolvedUtility.Standard("margin-inline-start:auto;"); val v = parseSpacingValue(r) ?: return null; return ResolvedUtility.Standard("margin-inline-start:$v;") }
        if (c.startsWith("me-")) { val r = c.removePrefix("me-"); if (r == "auto") return ResolvedUtility.Standard("margin-inline-end:auto;"); val v = parseSpacingValue(r) ?: return null; return ResolvedUtility.Standard("margin-inline-end:$v;") }
        if (c.startsWith("m-")) { val r = c.removePrefix("m-"); if (r == "auto") return ResolvedUtility.Standard("margin:auto;"); val v = parseSpacingValue(r) ?: return null; return ResolvedUtility.Standard("margin:$v;") }
        // Gap
        if (c.startsWith("gap-x-")) { val v = parseSpacingValue(c.removePrefix("gap-x-")) ?: return null; return ResolvedUtility.Standard("column-gap:$v;") }
        if (c.startsWith("gap-y-")) { val v = parseSpacingValue(c.removePrefix("gap-y-")) ?: return null; return ResolvedUtility.Standard("row-gap:$v;") }
        if (c.startsWith("gap-")) { val v = parseSpacingValue(c.removePrefix("gap-")) ?: return null; return ResolvedUtility.Standard("gap:$v;") }
        return null
    }

    // ── Sizing ──────────────────────────────────────────────────────────────

    private fun resolveDimension(rest: String, property: String): ResolvedUtility? {
        val d = when (rest) {
            "auto" -> "$property:auto;"
            "full" -> "$property:100%;"
            "screen" -> { val u = if (property == "width" || property.contains("width")) "vw" else "vh"; "$property:100$u;" }
            "svw" -> "$property:100svw;"; "svh" -> "$property:100svh;"
            "lvw" -> "$property:100lvw;"; "lvh" -> "$property:100lvh;"
            "dvw" -> "$property:100dvw;"; "dvh" -> "$property:100dvh;"
            "min" -> "$property:min-content;"; "max" -> "$property:max-content;"
            "fit" -> "$property:fit-content;"; "px" -> "$property:1px;"
            else -> {
                parseFraction(rest)?.let { return ResolvedUtility.Standard("$property:$it;") }
                parseSpacingValue(rest)?.let { return ResolvedUtility.Standard("$property:$it;") }
                return null
            }
        }
        return ResolvedUtility.Standard(d)
    }

    private fun resolveSizing(c: String): ResolvedUtility? {
        if (c.startsWith("basis-")) {
            val rest = c.removePrefix("basis-")
            val d = when (rest) {
                "auto" -> "flex-basis:auto;"; "full" -> "flex-basis:100%;"; "px" -> "flex-basis:1px;"
                else -> {
                    parseFraction(rest)?.let { return ResolvedUtility.Standard("flex-basis:$it;") }
                    parseSpacingValue(rest)?.let { return ResolvedUtility.Standard("flex-basis:$it;") }
                    return null
                }
            }
            return ResolvedUtility.Standard(d)
        }
        if (c.startsWith("size-")) {
            val rest = c.removePrefix("size-")
            val v = when (rest) {
                "auto" -> "auto"; "full" -> "100%"; "min" -> "min-content"; "max" -> "max-content"
                "fit" -> "fit-content"; "px" -> "1px"
                else -> parseFraction(rest) ?: parseSpacingValue(rest) ?: return null
            }
            return ResolvedUtility.Standard("width:$v;height:$v;")
        }
        if (c.startsWith("max-w-")) {
            val rest = c.removePrefix("max-w-")
            val d = when (rest) {
                "none" -> "max-width:none;"; "0" -> "max-width:0rem;"
                "xs" -> "max-width:20rem;"; "sm" -> "max-width:24rem;"; "md" -> "max-width:28rem;"
                "lg" -> "max-width:32rem;"; "xl" -> "max-width:36rem;"; "2xl" -> "max-width:42rem;"
                "3xl" -> "max-width:48rem;"; "4xl" -> "max-width:56rem;"; "5xl" -> "max-width:64rem;"
                "6xl" -> "max-width:72rem;"; "7xl" -> "max-width:80rem;"
                "full" -> "max-width:100%;"; "min" -> "max-width:min-content;"; "max" -> "max-width:max-content;"
                "fit" -> "max-width:fit-content;"; "prose" -> "max-width:65ch;"
                "screen-sm" -> "max-width:640px;"; "screen-md" -> "max-width:768px;"
                "screen-lg" -> "max-width:1024px;"; "screen-xl" -> "max-width:1280px;"
                "screen-2xl" -> "max-width:1536px;"; "screen" -> "max-width:100vw;"
                else -> { parseSpacingValue(rest)?.let { return ResolvedUtility.Standard("max-width:$it;") }; return null }
            }
            return ResolvedUtility.Standard(d)
        }
        if (c.startsWith("max-h-")) {
            val rest = c.removePrefix("max-h-")
            val d = when (rest) {
                "none" -> "max-height:none;"; "full" -> "max-height:100%;"; "screen" -> "max-height:100vh;"
                "min" -> "max-height:min-content;"; "max" -> "max-height:max-content;"; "fit" -> "max-height:fit-content;"
                else -> { parseSpacingValue(rest)?.let { return ResolvedUtility.Standard("max-height:$it;") }; return null }
            }
            return ResolvedUtility.Standard(d)
        }
        if (c.startsWith("min-w-")) {
            val rest = c.removePrefix("min-w-")
            val d = when (rest) {
                "0" -> "min-width:0px;"; "full" -> "min-width:100%;"; "min" -> "min-width:min-content;"
                "max" -> "min-width:max-content;"; "fit" -> "min-width:fit-content;"
                else -> { parseSpacingValue(rest)?.let { return ResolvedUtility.Standard("min-width:$it;") }; return null }
            }
            return ResolvedUtility.Standard(d)
        }
        if (c.startsWith("min-h-")) {
            val rest = c.removePrefix("min-h-")
            val d = when (rest) {
                "0" -> "min-height:0px;"; "full" -> "min-height:100%;"; "screen" -> "min-height:100vh;"
                "svh" -> "min-height:100svh;"; "lvh" -> "min-height:100lvh;"; "dvh" -> "min-height:100dvh;"
                "min" -> "min-height:min-content;"; "max" -> "min-height:max-content;"; "fit" -> "min-height:fit-content;"
                else -> { parseSpacingValue(rest)?.let { return ResolvedUtility.Standard("min-height:$it;") }; return null }
            }
            return ResolvedUtility.Standard(d)
        }
        if (c.startsWith("w-")) return resolveDimension(c.removePrefix("w-"), "width")
        if (c.startsWith("h-")) return resolveDimension(c.removePrefix("h-"), "height")
        return null
    }

    // ── Typography ──────────────────────────────────────────────────────────

    private fun resolveTypography(c: String): ResolvedUtility? {
        val decls = when (c) {
            "text-left" -> "text-align:left;"; "text-center" -> "text-align:center;"
            "text-right" -> "text-align:right;"; "text-justify" -> "text-align:justify;"
            "text-start" -> "text-align:start;"; "text-end" -> "text-align:end;"
            "uppercase" -> "text-transform:uppercase;"; "lowercase" -> "text-transform:lowercase;"
            "capitalize" -> "text-transform:capitalize;"; "normal-case" -> "text-transform:none;"
            "italic" -> "font-style:italic;"; "not-italic" -> "font-style:normal;"
            "underline" -> "text-decoration-line:underline;"; "no-underline" -> "text-decoration-line:none;"
            "line-through" -> "text-decoration-line:line-through;"; "overline" -> "text-decoration-line:overline;"
            "decoration-solid" -> "text-decoration-style:solid;"; "decoration-dashed" -> "text-decoration-style:dashed;"
            "decoration-dotted" -> "text-decoration-style:dotted;"; "decoration-double" -> "text-decoration-style:double;"
            "decoration-wavy" -> "text-decoration-style:wavy;"
            "decoration-auto" -> "text-decoration-thickness:auto;"; "decoration-from-font" -> "text-decoration-thickness:from-font;"
            "truncate" -> "overflow:hidden;text-overflow:ellipsis;white-space:nowrap;"
            "text-ellipsis" -> "text-overflow:ellipsis;"; "text-clip" -> "text-overflow:clip;"
            "whitespace-normal" -> "white-space:normal;"; "whitespace-nowrap" -> "white-space:nowrap;"
            "whitespace-pre" -> "white-space:pre;"; "whitespace-pre-line" -> "white-space:pre-line;"
            "whitespace-pre-wrap" -> "white-space:pre-wrap;"; "whitespace-break-spaces" -> "white-space:break-spaces;"
            "break-normal" -> "overflow-wrap:normal;word-break:normal;"; "break-all" -> "word-break:break-all;"
            "break-keep" -> "word-break:keep-all;"; "break-words" -> "overflow-wrap:break-word;"
            "text-wrap" -> "text-wrap:wrap;"; "text-nowrap" -> "text-wrap:nowrap;"
            "text-balance" -> "text-wrap:balance;"; "text-pretty" -> "text-wrap:pretty;"
            "font-sans" -> "font-family:ui-sans-serif,system-ui,sans-serif,\"Apple Color Emoji\",\"Segoe UI Emoji\",\"Segoe UI Symbol\",\"Noto Color Emoji\";"
            "font-serif" -> "font-family:ui-serif,Georgia,Cambria,\"Times New Roman\",Times,serif;"
            "font-mono" -> "font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,\"Liberation Mono\",\"Courier New\",monospace;"
            "list-none" -> "list-style-type:none;"; "list-disc" -> "list-style-type:disc;"; "list-decimal" -> "list-style-type:decimal;"
            "list-inside" -> "list-style-position:inside;"; "list-outside" -> "list-style-position:outside;"
            "align-baseline" -> "vertical-align:baseline;"; "align-top" -> "vertical-align:top;"
            "align-middle" -> "vertical-align:middle;"; "align-bottom" -> "vertical-align:bottom;"
            "align-text-top" -> "vertical-align:text-top;"; "align-text-bottom" -> "vertical-align:text-bottom;"
            "align-sub" -> "vertical-align:sub;"; "align-super" -> "vertical-align:super;"
            "hyphens-none" -> "hyphens:none;"; "hyphens-manual" -> "hyphens:manual;"; "hyphens-auto" -> "hyphens:auto;"
            "content-none" -> "content:none;"
            else -> return resolveTypographyPrefix(c)
        }
        return ResolvedUtility.Standard(decls)
    }

    private fun resolveTypographyPrefix(c: String): ResolvedUtility? {
        if (c.startsWith("text-")) {
            val rest = c.removePrefix("text-")
            val sizeDecl = when (rest) {
                "xs" -> "font-size:0.75rem;line-height:1rem;"; "sm" -> "font-size:0.875rem;line-height:1.25rem;"
                "base" -> "font-size:1rem;line-height:1.5rem;"; "lg" -> "font-size:1.125rem;line-height:1.75rem;"
                "xl" -> "font-size:1.25rem;line-height:1.75rem;"; "2xl" -> "font-size:1.5rem;line-height:2rem;"
                "3xl" -> "font-size:1.875rem;line-height:2.25rem;"; "4xl" -> "font-size:2.25rem;line-height:2.5rem;"
                "5xl" -> "font-size:3rem;line-height:1;"; "6xl" -> "font-size:3.75rem;line-height:1;"
                "7xl" -> "font-size:4.5rem;line-height:1;"; "8xl" -> "font-size:6rem;line-height:1;"
                "9xl" -> "font-size:8rem;line-height:1;"
                else -> null
            }
            if (sizeDecl != null) return ResolvedUtility.Standard(sizeDecl)
            resolveColorWithOpacity(rest, "color")?.let { return ResolvedUtility.Standard(it) }
            parseArbitrary(rest)?.let { return ResolvedUtility.Standard("color:$it;") }
            return null
        }
        if (c.startsWith("font-")) {
            val rest = c.removePrefix("font-")
            val d = when (rest) {
                "thin" -> "font-weight:100;"; "extralight" -> "font-weight:200;"; "light" -> "font-weight:300;"
                "normal" -> "font-weight:400;"; "medium" -> "font-weight:500;"; "semibold" -> "font-weight:600;"
                "bold" -> "font-weight:700;"; "extrabold" -> "font-weight:800;"; "black" -> "font-weight:900;"
                else -> return null
            }
            return ResolvedUtility.Standard(d)
        }
        if (c.startsWith("leading-")) {
            val rest = c.removePrefix("leading-")
            val d = when (rest) {
                "none" -> "line-height:1;"; "tight" -> "line-height:1.25;"; "snug" -> "line-height:1.375;"
                "normal" -> "line-height:1.5;"; "relaxed" -> "line-height:1.625;"; "loose" -> "line-height:2;"
                else -> { val n = parseU32(rest) ?: return null; "line-height:${spacing(n)};" }
            }
            return ResolvedUtility.Standard(d)
        }
        if (c.startsWith("tracking-")) {
            val rest = c.removePrefix("tracking-")
            val d = when (rest) {
                "tighter" -> "letter-spacing:-0.05em;"; "tight" -> "letter-spacing:-0.025em;"
                "normal" -> "letter-spacing:0em;"; "wide" -> "letter-spacing:0.025em;"
                "wider" -> "letter-spacing:0.05em;"; "widest" -> "letter-spacing:0.1em;"
                else -> return null
            }
            return ResolvedUtility.Standard(d)
        }
        if (c.startsWith("line-clamp-")) {
            val rest = c.removePrefix("line-clamp-")
            if (rest == "none") return ResolvedUtility.Standard("overflow:visible;display:block;-webkit-box-orient:horizontal;-webkit-line-clamp:none;")
            val n = parseU32(rest) ?: return null
            return ResolvedUtility.Standard("overflow:hidden;display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:$n;")
        }
        if (c.startsWith("decoration-")) {
            val rest = c.removePrefix("decoration-")
            parseU32(rest)?.let { return ResolvedUtility.Standard("text-decoration-thickness:${it}px;") }
            resolveColorWithOpacity(rest, "text-decoration-color")?.let { return ResolvedUtility.Standard(it) }
            return null
        }
        if (c.startsWith("underline-offset-")) {
            val rest = c.removePrefix("underline-offset-")
            if (rest == "auto") return ResolvedUtility.Standard("text-underline-offset:auto;")
            val n = parseU32(rest) ?: return null
            return ResolvedUtility.Standard("text-underline-offset:${n}px;")
        }
        if (c.startsWith("indent-")) {
            val v = parseSpacingValue(c.removePrefix("indent-")) ?: return null
            return ResolvedUtility.Standard("text-indent:$v;")
        }
        return null
    }

    // ── Backgrounds ─────────────────────────────────────────────────────────

    private fun resolveBackgrounds(c: String): ResolvedUtility? {
        val decls = when (c) {
            "bg-gradient-to-t" -> "background-image:linear-gradient(to top,var(--tw-gradient-stops));"
            "bg-gradient-to-tr" -> "background-image:linear-gradient(to top right,var(--tw-gradient-stops));"
            "bg-gradient-to-r" -> "background-image:linear-gradient(to right,var(--tw-gradient-stops));"
            "bg-gradient-to-br" -> "background-image:linear-gradient(to bottom right,var(--tw-gradient-stops));"
            "bg-gradient-to-b" -> "background-image:linear-gradient(to bottom,var(--tw-gradient-stops));"
            "bg-gradient-to-bl" -> "background-image:linear-gradient(to bottom left,var(--tw-gradient-stops));"
            "bg-gradient-to-l" -> "background-image:linear-gradient(to left,var(--tw-gradient-stops));"
            "bg-gradient-to-tl" -> "background-image:linear-gradient(to top left,var(--tw-gradient-stops));"
            "bg-none" -> "background-image:none;"
            "bg-auto" -> "background-size:auto;"; "bg-cover" -> "background-size:cover;"; "bg-contain" -> "background-size:contain;"
            "bg-center" -> "background-position:center;"; "bg-top" -> "background-position:top;"
            "bg-right" -> "background-position:right;"; "bg-bottom" -> "background-position:bottom;"
            "bg-left" -> "background-position:left;"
            "bg-left-bottom" -> "background-position:left bottom;"; "bg-left-top" -> "background-position:left top;"
            "bg-right-bottom" -> "background-position:right bottom;"; "bg-right-top" -> "background-position:right top;"
            "bg-repeat" -> "background-repeat:repeat;"; "bg-no-repeat" -> "background-repeat:no-repeat;"
            "bg-repeat-x" -> "background-repeat:repeat-x;"; "bg-repeat-y" -> "background-repeat:repeat-y;"
            "bg-repeat-round" -> "background-repeat:round;"; "bg-repeat-space" -> "background-repeat:space;"
            "bg-fixed" -> "background-attachment:fixed;"; "bg-local" -> "background-attachment:local;"; "bg-scroll" -> "background-attachment:scroll;"
            "bg-clip-border" -> "background-clip:border-box;"; "bg-clip-padding" -> "background-clip:padding-box;"
            "bg-clip-content" -> "background-clip:content-box;"; "bg-clip-text" -> "-webkit-background-clip:text;background-clip:text;"
            "bg-origin-border" -> "background-origin:border-box;"; "bg-origin-padding" -> "background-origin:padding-box;"
            "bg-origin-content" -> "background-origin:content-box;"
            else -> {
                if (c.startsWith("bg-")) {
                    val rest = c.removePrefix("bg-")
                    resolveColorWithOpacity(rest, "background-color")?.let { return ResolvedUtility.Standard(it) }
                    parseArbitrary(rest)?.let { return ResolvedUtility.Standard("background-color:$it;") }
                    return null
                }
                if (c.startsWith("from-")) { val hex = VolkiStylePalette.colorHex(c.removePrefix("from-")) ?: return null; return ResolvedUtility.Standard("--tw-gradient-from:$hex var(--tw-gradient-from-position);--tw-gradient-to:rgb(255 255 255 / 0) var(--tw-gradient-to-position);--tw-gradient-stops:var(--tw-gradient-from),var(--tw-gradient-to);") }
                if (c.startsWith("via-")) { val hex = VolkiStylePalette.colorHex(c.removePrefix("via-")) ?: return null; return ResolvedUtility.Standard("--tw-gradient-to:rgb(255 255 255 / 0) var(--tw-gradient-to-position);--tw-gradient-stops:var(--tw-gradient-from),$hex var(--tw-gradient-via-position),var(--tw-gradient-to);") }
                if (c.startsWith("to-")) { val hex = VolkiStylePalette.colorHex(c.removePrefix("to-")) ?: return null; return ResolvedUtility.Standard("--tw-gradient-to:$hex var(--tw-gradient-to-position);") }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    // ── Borders ─────────────────────────────────────────────────────────────

    private fun resolveBorders(c: String): ResolvedUtility? {
        val childSuffix = ">:not([hidden])~:not([hidden])"
        val decls = when (c) {
            "border" -> "border-width:1px;"; "border-0" -> "border-width:0px;"
            "border-2" -> "border-width:2px;"; "border-4" -> "border-width:4px;"; "border-8" -> "border-width:8px;"
            "border-t" -> "border-top-width:1px;"; "border-r" -> "border-right-width:1px;"
            "border-b" -> "border-bottom-width:1px;"; "border-l" -> "border-left-width:1px;"
            "border-x" -> "border-left-width:1px;border-right-width:1px;"; "border-y" -> "border-top-width:1px;border-bottom-width:1px;"
            "border-solid" -> "border-style:solid;"; "border-dashed" -> "border-style:dashed;"
            "border-dotted" -> "border-style:dotted;"; "border-double" -> "border-style:double;"
            "border-hidden" -> "border-style:hidden;"; "border-none" -> "border-style:none;"
            "rounded" -> "border-radius:0.25rem;"; "rounded-none" -> "border-radius:0px;"
            "rounded-sm" -> "border-radius:0.125rem;"; "rounded-md" -> "border-radius:0.375rem;"
            "rounded-lg" -> "border-radius:0.5rem;"; "rounded-xl" -> "border-radius:0.75rem;"
            "rounded-2xl" -> "border-radius:1rem;"; "rounded-3xl" -> "border-radius:1.5rem;"
            "rounded-full" -> "border-radius:9999px;"
            "rounded-t" -> "border-top-left-radius:0.25rem;border-top-right-radius:0.25rem;"
            "rounded-r" -> "border-top-right-radius:0.25rem;border-bottom-right-radius:0.25rem;"
            "rounded-b" -> "border-bottom-right-radius:0.25rem;border-bottom-left-radius:0.25rem;"
            "rounded-l" -> "border-top-left-radius:0.25rem;border-bottom-left-radius:0.25rem;"
            "outline-none" -> "outline:2px solid transparent;outline-offset:2px;"
            "outline" -> "outline-style:solid;"; "outline-dashed" -> "outline-style:dashed;"
            "outline-dotted" -> "outline-style:dotted;"; "outline-double" -> "outline-style:double;"
            "ring" -> "box-shadow:0 0 0 3px rgba(59,130,246,0.5);"
            "ring-0" -> "box-shadow:0 0 0 0px rgba(59,130,246,0.5);"
            "ring-1" -> "box-shadow:0 0 0 1px rgba(59,130,246,0.5);"
            "ring-2" -> "box-shadow:0 0 0 2px rgba(59,130,246,0.5);"
            "ring-4" -> "box-shadow:0 0 0 4px rgba(59,130,246,0.5);"
            "ring-8" -> "box-shadow:0 0 0 8px rgba(59,130,246,0.5);"
            "ring-inset" -> "--tw-ring-inset:inset;"
            "divide-x" -> return ResolvedUtility.Custom(childSuffix, "border-left-width:1px;")
            "divide-y" -> return ResolvedUtility.Custom(childSuffix, "border-top-width:1px;")
            "divide-x-0" -> return ResolvedUtility.Custom(childSuffix, "border-left-width:0px;")
            "divide-y-0" -> return ResolvedUtility.Custom(childSuffix, "border-top-width:0px;")
            "divide-x-2" -> return ResolvedUtility.Custom(childSuffix, "border-left-width:2px;")
            "divide-y-2" -> return ResolvedUtility.Custom(childSuffix, "border-top-width:2px;")
            "divide-x-4" -> return ResolvedUtility.Custom(childSuffix, "border-left-width:4px;")
            "divide-y-4" -> return ResolvedUtility.Custom(childSuffix, "border-top-width:4px;")
            "divide-x-8" -> return ResolvedUtility.Custom(childSuffix, "border-left-width:8px;")
            "divide-y-8" -> return ResolvedUtility.Custom(childSuffix, "border-top-width:8px;")
            "divide-solid" -> return ResolvedUtility.Custom(childSuffix, "border-style:solid;")
            "divide-dashed" -> return ResolvedUtility.Custom(childSuffix, "border-style:dashed;")
            "divide-dotted" -> return ResolvedUtility.Custom(childSuffix, "border-style:dotted;")
            "divide-double" -> return ResolvedUtility.Custom(childSuffix, "border-style:double;")
            "divide-none" -> return ResolvedUtility.Custom(childSuffix, "border-style:none;")
            else -> return resolveBordersPrefix(c)
        }
        return ResolvedUtility.Standard(decls)
    }

    private fun resolveBordersPrefix(c: String): ResolvedUtility? {
        val childSuffix = ">:not([hidden])~:not([hidden])"
        for ((prefix, prop) in listOf("border-t-" to "border-top", "border-r-" to "border-right", "border-b-" to "border-bottom", "border-l-" to "border-left")) {
            if (c.startsWith(prefix)) {
                val rest = c.removePrefix(prefix)
                parseU32(rest)?.let { return ResolvedUtility.Standard("$prop-width:${it}px;") }
                resolveColorWithOpacity(rest, "$prop-color")?.let { return ResolvedUtility.Standard(it) }
                return null
            }
        }
        if (c.startsWith("border-x-")) { val rest = c.removePrefix("border-x-"); val n = parseU32(rest) ?: return null; return ResolvedUtility.Standard("border-left-width:${n}px;border-right-width:${n}px;") }
        if (c.startsWith("border-y-")) { val rest = c.removePrefix("border-y-"); val n = parseU32(rest) ?: return null; return ResolvedUtility.Standard("border-top-width:${n}px;border-bottom-width:${n}px;") }
        if (c.startsWith("border-")) {
            val rest = c.removePrefix("border-")
            parseU32(rest)?.let { return ResolvedUtility.Standard("border-width:${it}px;") }
            resolveColorWithOpacity(rest, "border-color")?.let { return ResolvedUtility.Standard(it) }
            parseArbitrary(rest)?.let { return ResolvedUtility.Standard("border-color:$it;") }
            return null
        }
        // Rounded per-side/corner with size
        for ((prefix, props) in listOf(
            "rounded-t-" to "border-top-left-radius:%s;border-top-right-radius:%s;",
            "rounded-r-" to "border-top-right-radius:%s;border-bottom-right-radius:%s;",
            "rounded-b-" to "border-bottom-right-radius:%s;border-bottom-left-radius:%s;",
            "rounded-l-" to "border-top-left-radius:%s;border-bottom-left-radius:%s;",
            "rounded-tl-" to "border-top-left-radius:%s;",
            "rounded-tr-" to "border-top-right-radius:%s;",
            "rounded-bl-" to "border-bottom-left-radius:%s;",
            "rounded-br-" to "border-bottom-right-radius:%s;"
        )) {
            if (c.startsWith(prefix)) {
                val v = radiusValue(c.removePrefix(prefix)) ?: return null
                return ResolvedUtility.Standard(props.replace("%s", v))
            }
        }
        // Outline width/color/offset
        if (c.startsWith("outline-offset-")) { val n = parseU32(c.removePrefix("outline-offset-")) ?: return null; return ResolvedUtility.Standard("outline-offset:${n}px;") }
        if (c.startsWith("outline-")) {
            val rest = c.removePrefix("outline-")
            parseU32(rest)?.let { return ResolvedUtility.Standard("outline-width:${it}px;") }
            resolveColorWithOpacity(rest, "outline-color")?.let { return ResolvedUtility.Standard(it) }
            return null
        }
        // Ring offset
        if (c.startsWith("ring-offset-")) {
            val rest = c.removePrefix("ring-offset-")
            parseU32(rest)?.let { return ResolvedUtility.Standard("--tw-ring-offset-width:${it}px;box-shadow:0 0 0 var(--tw-ring-offset-width) var(--tw-ring-offset-color),var(--tw-ring-shadow);") }
            VolkiStylePalette.colorHex(rest)?.let { return ResolvedUtility.Standard("--tw-ring-offset-color:$it;") }
            return null
        }
        if (c.startsWith("ring-")) {
            val rest = c.removePrefix("ring-")
            VolkiStylePalette.colorHex(rest)?.let { return ResolvedUtility.Standard("--tw-ring-color:$it;") }
            return null
        }
        // Divide color
        if (c.startsWith("divide-")) {
            val rest = c.removePrefix("divide-")
            VolkiStylePalette.colorHex(rest)?.let { return ResolvedUtility.Custom(childSuffix, "border-color:$it;") }
            return null
        }
        return null
    }

    private fun radiusValue(size: String): String? = when (size) {
        "none" -> "0px"; "sm" -> "0.125rem"; "md" -> "0.375rem"; "lg" -> "0.5rem"
        "xl" -> "0.75rem"; "2xl" -> "1rem"; "3xl" -> "1.5rem"; "full" -> "9999px"
        else -> null
    }

    // ── Effects ─────────────────────────────────────────────────────────────

    private fun resolveEffects(c: String): ResolvedUtility? {
        val decls = when (c) {
            "shadow" -> "box-shadow:0 1px 3px 0 rgba(0,0,0,0.1),0 1px 2px -1px rgba(0,0,0,0.1);"
            "shadow-sm" -> "box-shadow:0 1px 2px 0 rgba(0,0,0,0.05);"
            "shadow-md" -> "box-shadow:0 4px 6px -1px rgba(0,0,0,0.1),0 2px 4px -2px rgba(0,0,0,0.1);"
            "shadow-lg" -> "box-shadow:0 10px 15px -3px rgba(0,0,0,0.1),0 4px 6px -4px rgba(0,0,0,0.1);"
            "shadow-xl" -> "box-shadow:0 20px 25px -5px rgba(0,0,0,0.1),0 8px 10px -6px rgba(0,0,0,0.1);"
            "shadow-2xl" -> "box-shadow:0 25px 50px -12px rgba(0,0,0,0.25);"
            "shadow-inner" -> "box-shadow:inset 0 2px 4px 0 rgba(0,0,0,0.05);"
            "shadow-none" -> "box-shadow:0 0 #0000;"
            "mix-blend-normal" -> "mix-blend-mode:normal;"; "mix-blend-multiply" -> "mix-blend-mode:multiply;"
            "mix-blend-screen" -> "mix-blend-mode:screen;"; "mix-blend-overlay" -> "mix-blend-mode:overlay;"
            "mix-blend-darken" -> "mix-blend-mode:darken;"; "mix-blend-lighten" -> "mix-blend-mode:lighten;"
            "mix-blend-color-dodge" -> "mix-blend-mode:color-dodge;"; "mix-blend-color-burn" -> "mix-blend-mode:color-burn;"
            "mix-blend-hard-light" -> "mix-blend-mode:hard-light;"; "mix-blend-soft-light" -> "mix-blend-mode:soft-light;"
            "mix-blend-difference" -> "mix-blend-mode:difference;"; "mix-blend-exclusion" -> "mix-blend-mode:exclusion;"
            "mix-blend-hue" -> "mix-blend-mode:hue;"; "mix-blend-saturation" -> "mix-blend-mode:saturation;"
            "mix-blend-color" -> "mix-blend-mode:color;"; "mix-blend-luminosity" -> "mix-blend-mode:luminosity;"
            "mix-blend-plus-lighter" -> "mix-blend-mode:plus-lighter;"
            "bg-blend-normal" -> "background-blend-mode:normal;"; "bg-blend-multiply" -> "background-blend-mode:multiply;"
            "bg-blend-screen" -> "background-blend-mode:screen;"; "bg-blend-overlay" -> "background-blend-mode:overlay;"
            "bg-blend-darken" -> "background-blend-mode:darken;"; "bg-blend-lighten" -> "background-blend-mode:lighten;"
            "bg-blend-color-dodge" -> "background-blend-mode:color-dodge;"; "bg-blend-color-burn" -> "background-blend-mode:color-burn;"
            "bg-blend-hard-light" -> "background-blend-mode:hard-light;"; "bg-blend-soft-light" -> "background-blend-mode:soft-light;"
            "bg-blend-difference" -> "background-blend-mode:difference;"; "bg-blend-exclusion" -> "background-blend-mode:exclusion;"
            "bg-blend-hue" -> "background-blend-mode:hue;"; "bg-blend-saturation" -> "background-blend-mode:saturation;"
            "bg-blend-color" -> "background-blend-mode:color;"; "bg-blend-luminosity" -> "background-blend-mode:luminosity;"
            else -> {
                if (c.startsWith("opacity-")) {
                    val n = parseU32(c.removePrefix("opacity-")) ?: return null
                    if (n > 100) return null
                    val d = when { n == 0 -> "opacity:0;"; n == 100 -> "opacity:1;"; n % 10 == 0 -> "opacity:0.${n / 10};"; else -> "opacity:0.$n;" }
                    return ResolvedUtility.Standard(d)
                }
                if (c.startsWith("shadow-")) {
                    val hex = VolkiStylePalette.colorHex(c.removePrefix("shadow-")) ?: return null
                    return ResolvedUtility.Standard("--tw-shadow-color:$hex;")
                }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    // ── Transforms ──────────────────────────────────────────────────────────

    private fun resolveTransforms(c: String): ResolvedUtility? {
        val decls = when (c) {
            "origin-center" -> "transform-origin:center;"; "origin-top" -> "transform-origin:top;"
            "origin-top-right" -> "transform-origin:top right;"; "origin-right" -> "transform-origin:right;"
            "origin-bottom-right" -> "transform-origin:bottom right;"; "origin-bottom" -> "transform-origin:bottom;"
            "origin-bottom-left" -> "transform-origin:bottom left;"; "origin-left" -> "transform-origin:left;"
            "origin-top-left" -> "transform-origin:top left;"
            else -> {
                if (c.startsWith("scale-x-")) { val n = parseU32(c.removePrefix("scale-x-")) ?: return null; return ResolvedUtility.Standard("transform:scaleX(${scaleValue(n)});") }
                if (c.startsWith("scale-y-")) { val n = parseU32(c.removePrefix("scale-y-")) ?: return null; return ResolvedUtility.Standard("transform:scaleY(${scaleValue(n)});") }
                if (c.startsWith("scale-")) { val n = parseU32(c.removePrefix("scale-")) ?: return null; return ResolvedUtility.Standard("transform:scale(${scaleValue(n)});") }
                if (c.startsWith("-rotate-")) { val n = parseU32(c.removePrefix("-rotate-")) ?: return null; return ResolvedUtility.Standard("transform:rotate(-${n}deg);") }
                if (c.startsWith("rotate-")) { val n = parseU32(c.removePrefix("rotate-")) ?: return null; return ResolvedUtility.Standard("transform:rotate(${n}deg);") }
                if (c.startsWith("-translate-x-")) { val rest = c.removePrefix("-translate-x-"); parseFraction(rest)?.let { return ResolvedUtility.Standard("transform:translateX(-$it);") }; val v = parseSpacingValue(rest) ?: return null; return ResolvedUtility.Standard("transform:translateX(-$v);") }
                if (c.startsWith("-translate-y-")) { val rest = c.removePrefix("-translate-y-"); parseFraction(rest)?.let { return ResolvedUtility.Standard("transform:translateY(-$it);") }; val v = parseSpacingValue(rest) ?: return null; return ResolvedUtility.Standard("transform:translateY(-$v);") }
                if (c.startsWith("translate-x-")) { val rest = c.removePrefix("translate-x-"); if (rest == "full") return ResolvedUtility.Standard("transform:translateX(100%);"); parseFraction(rest)?.let { return ResolvedUtility.Standard("transform:translateX($it);") }; val v = parseSpacingValue(rest) ?: return null; return ResolvedUtility.Standard("transform:translateX($v);") }
                if (c.startsWith("translate-y-")) { val rest = c.removePrefix("translate-y-"); if (rest == "full") return ResolvedUtility.Standard("transform:translateY(100%);"); parseFraction(rest)?.let { return ResolvedUtility.Standard("transform:translateY($it);") }; val v = parseSpacingValue(rest) ?: return null; return ResolvedUtility.Standard("transform:translateY($v);") }
                if (c.startsWith("-skew-x-")) { val n = parseU32(c.removePrefix("-skew-x-")) ?: return null; return ResolvedUtility.Standard("transform:skewX(-${n}deg);") }
                if (c.startsWith("-skew-y-")) { val n = parseU32(c.removePrefix("-skew-y-")) ?: return null; return ResolvedUtility.Standard("transform:skewY(-${n}deg);") }
                if (c.startsWith("skew-x-")) { val n = parseU32(c.removePrefix("skew-x-")) ?: return null; return ResolvedUtility.Standard("transform:skewX(${n}deg);") }
                if (c.startsWith("skew-y-")) { val n = parseU32(c.removePrefix("skew-y-")) ?: return null; return ResolvedUtility.Standard("transform:skewY(${n}deg);") }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    private fun scaleValue(n: Int): String = when {
        n == 0 -> "0"; n == 100 -> "1"; n % 100 == 0 -> "${n / 100}"
        n % 10 == 0 -> "${n / 100}.${(n % 100) / 10}"; else -> "${n / 100}.${n % 100}"
    }

    // ── Filters ─────────────────────────────────────────────────────────────

    private fun resolveFilters(c: String): ResolvedUtility? {
        val decls = when (c) {
            "blur-none" -> "filter:blur(0);"; "blur-sm" -> "filter:blur(4px);"; "blur" -> "filter:blur(8px);"
            "blur-md" -> "filter:blur(12px);"; "blur-lg" -> "filter:blur(16px);"; "blur-xl" -> "filter:blur(24px);"
            "blur-2xl" -> "filter:blur(40px);"; "blur-3xl" -> "filter:blur(64px);"
            "grayscale" -> "filter:grayscale(100%);"; "grayscale-0" -> "filter:grayscale(0);"
            "invert" -> "filter:invert(100%);"; "invert-0" -> "filter:invert(0);"
            "sepia" -> "filter:sepia(100%);"; "sepia-0" -> "filter:sepia(0);"
            "drop-shadow-sm" -> "filter:drop-shadow(0 1px 1px rgba(0,0,0,0.05));"
            "drop-shadow" -> "filter:drop-shadow(0 1px 2px rgba(0,0,0,0.1)) drop-shadow(0 1px 1px rgba(0,0,0,0.06));"
            "drop-shadow-md" -> "filter:drop-shadow(0 4px 3px rgba(0,0,0,0.07)) drop-shadow(0 2px 2px rgba(0,0,0,0.06));"
            "drop-shadow-lg" -> "filter:drop-shadow(0 10px 8px rgba(0,0,0,0.04)) drop-shadow(0 4px 3px rgba(0,0,0,0.1));"
            "drop-shadow-xl" -> "filter:drop-shadow(0 20px 13px rgba(0,0,0,0.03)) drop-shadow(0 8px 5px rgba(0,0,0,0.08));"
            "drop-shadow-2xl" -> "filter:drop-shadow(0 25px 25px rgba(0,0,0,0.15));"
            "drop-shadow-none" -> "filter:drop-shadow(0 0 #0000);"
            "backdrop-blur-none" -> "backdrop-filter:blur(0);"; "backdrop-blur-sm" -> "backdrop-filter:blur(4px);"
            "backdrop-blur" -> "backdrop-filter:blur(8px);"; "backdrop-blur-md" -> "backdrop-filter:blur(12px);"
            "backdrop-blur-lg" -> "backdrop-filter:blur(16px);"; "backdrop-blur-xl" -> "backdrop-filter:blur(24px);"
            "backdrop-blur-2xl" -> "backdrop-filter:blur(40px);"; "backdrop-blur-3xl" -> "backdrop-filter:blur(64px);"
            "backdrop-grayscale" -> "backdrop-filter:grayscale(100%);"; "backdrop-grayscale-0" -> "backdrop-filter:grayscale(0);"
            "backdrop-invert" -> "backdrop-filter:invert(100%);"; "backdrop-invert-0" -> "backdrop-filter:invert(0);"
            "backdrop-sepia" -> "backdrop-filter:sepia(100%);"; "backdrop-sepia-0" -> "backdrop-filter:sepia(0);"
            "backdrop-opacity-0" -> "backdrop-filter:opacity(0);"; "backdrop-opacity-100" -> "backdrop-filter:opacity(1);"
            else -> {
                if (c.startsWith("brightness-")) { val n = parseU32(c.removePrefix("brightness-")) ?: return null; return ResolvedUtility.Standard("filter:brightness(${filterPercent(n)});") }
                if (c.startsWith("contrast-")) { val n = parseU32(c.removePrefix("contrast-")) ?: return null; return ResolvedUtility.Standard("filter:contrast(${filterPercent(n)});") }
                if (c.startsWith("saturate-")) { val n = parseU32(c.removePrefix("saturate-")) ?: return null; return ResolvedUtility.Standard("filter:saturate(${filterPercent(n)});") }
                if (c.startsWith("-hue-rotate-")) { val n = parseU32(c.removePrefix("-hue-rotate-")) ?: return null; return ResolvedUtility.Standard("filter:hue-rotate(-${n}deg);") }
                if (c.startsWith("hue-rotate-")) { val n = parseU32(c.removePrefix("hue-rotate-")) ?: return null; return ResolvedUtility.Standard("filter:hue-rotate(${n}deg);") }
                if (c.startsWith("backdrop-brightness-")) { val n = parseU32(c.removePrefix("backdrop-brightness-")) ?: return null; return ResolvedUtility.Standard("backdrop-filter:brightness(${filterPercent(n)});") }
                if (c.startsWith("backdrop-contrast-")) { val n = parseU32(c.removePrefix("backdrop-contrast-")) ?: return null; return ResolvedUtility.Standard("backdrop-filter:contrast(${filterPercent(n)});") }
                if (c.startsWith("backdrop-saturate-")) { val n = parseU32(c.removePrefix("backdrop-saturate-")) ?: return null; return ResolvedUtility.Standard("backdrop-filter:saturate(${filterPercent(n)});") }
                if (c.startsWith("backdrop-hue-rotate-")) { val n = parseU32(c.removePrefix("backdrop-hue-rotate-")) ?: return null; return ResolvedUtility.Standard("backdrop-filter:hue-rotate(${n}deg);") }
                if (c.startsWith("backdrop-opacity-")) { val n = parseU32(c.removePrefix("backdrop-opacity-")) ?: return null; if (n > 100) return null; return ResolvedUtility.Standard("backdrop-filter:opacity(${filterPercent(n)});") }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    private fun filterPercent(n: Int): String = when {
        n == 0 -> "0"; n == 100 -> "1"; n % 100 == 0 -> "${n / 100}"
        n % 10 == 0 -> "${n / 100}.${(n % 100) / 10}"; else -> "${n / 100}.${n % 100}"
    }

    // ── Transitions ─────────────────────────────────────────────────────────

    private fun resolveTransitions(c: String): ResolvedUtility? {
        val decls = when (c) {
            "transition" -> "transition-property:color,background-color,border-color,text-decoration-color,fill,stroke,opacity,box-shadow,transform,filter,backdrop-filter;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;"
            "transition-none" -> "transition-property:none;"
            "transition-all" -> "transition-property:all;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;"
            "transition-colors" -> "transition-property:color,background-color,border-color,text-decoration-color,fill,stroke;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;"
            "transition-opacity" -> "transition-property:opacity;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;"
            "transition-shadow" -> "transition-property:box-shadow;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;"
            "transition-transform" -> "transition-property:transform;transition-timing-function:cubic-bezier(0.4,0,0.2,1);transition-duration:150ms;"
            "ease-linear" -> "transition-timing-function:linear;"
            "ease-in" -> "transition-timing-function:cubic-bezier(0.4,0,1,1);"
            "ease-out" -> "transition-timing-function:cubic-bezier(0,0,0.2,1);"
            "ease-in-out" -> "transition-timing-function:cubic-bezier(0.4,0,0.2,1);"
            "animate-none" -> "animation:none;"
            "animate-spin" -> "animation:spin 1s linear infinite;"
            "animate-ping" -> "animation:ping 1s cubic-bezier(0,0,0.2,1) infinite;"
            "animate-pulse" -> "animation:pulse 2s cubic-bezier(0.4,0,0.6,1) infinite;"
            "animate-bounce" -> "animation:bounce 1s infinite;"
            else -> {
                if (c.startsWith("duration-")) { val n = parseU32(c.removePrefix("duration-")) ?: return null; return ResolvedUtility.Standard("transition-duration:${n}ms;") }
                if (c.startsWith("delay-")) { val n = parseU32(c.removePrefix("delay-")) ?: return null; return ResolvedUtility.Standard("transition-delay:${n}ms;") }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    // ── Interactivity ───────────────────────────────────────────────────────

    private fun resolveInteractivity(c: String): ResolvedUtility? {
        val decls = when (c) {
            "cursor-auto" -> "cursor:auto;"; "cursor-default" -> "cursor:default;"; "cursor-pointer" -> "cursor:pointer;"
            "cursor-wait" -> "cursor:wait;"; "cursor-text" -> "cursor:text;"; "cursor-move" -> "cursor:move;"
            "cursor-help" -> "cursor:help;"; "cursor-not-allowed" -> "cursor:not-allowed;"; "cursor-none" -> "cursor:none;"
            "cursor-context-menu" -> "cursor:context-menu;"; "cursor-progress" -> "cursor:progress;"
            "cursor-cell" -> "cursor:cell;"; "cursor-crosshair" -> "cursor:crosshair;"
            "cursor-vertical-text" -> "cursor:vertical-text;"; "cursor-alias" -> "cursor:alias;"
            "cursor-copy" -> "cursor:copy;"; "cursor-no-drop" -> "cursor:no-drop;"
            "cursor-grab" -> "cursor:grab;"; "cursor-grabbing" -> "cursor:grabbing;"
            "cursor-all-scroll" -> "cursor:all-scroll;"; "cursor-col-resize" -> "cursor:col-resize;"
            "cursor-row-resize" -> "cursor:row-resize;"; "cursor-n-resize" -> "cursor:n-resize;"
            "cursor-e-resize" -> "cursor:e-resize;"; "cursor-s-resize" -> "cursor:s-resize;"
            "cursor-w-resize" -> "cursor:w-resize;"; "cursor-ne-resize" -> "cursor:ne-resize;"
            "cursor-nw-resize" -> "cursor:nw-resize;"; "cursor-se-resize" -> "cursor:se-resize;"
            "cursor-sw-resize" -> "cursor:sw-resize;"; "cursor-ew-resize" -> "cursor:ew-resize;"
            "cursor-ns-resize" -> "cursor:ns-resize;"; "cursor-nesw-resize" -> "cursor:nesw-resize;"
            "cursor-nwse-resize" -> "cursor:nwse-resize;"; "cursor-zoom-in" -> "cursor:zoom-in;"
            "cursor-zoom-out" -> "cursor:zoom-out;"
            "resize-none" -> "resize:none;"; "resize-y" -> "resize:vertical;"
            "resize-x" -> "resize:horizontal;"; "resize" -> "resize:both;"
            "select-none" -> "user-select:none;"; "select-text" -> "user-select:text;"
            "select-all" -> "user-select:all;"; "select-auto" -> "user-select:auto;"
            "pointer-events-none" -> "pointer-events:none;"; "pointer-events-auto" -> "pointer-events:auto;"
            "scroll-auto" -> "scroll-behavior:auto;"; "scroll-smooth" -> "scroll-behavior:smooth;"
            "snap-none" -> "scroll-snap-type:none;"
            "snap-x" -> "scroll-snap-type:x var(--tw-scroll-snap-strictness);"
            "snap-y" -> "scroll-snap-type:y var(--tw-scroll-snap-strictness);"
            "snap-both" -> "scroll-snap-type:both var(--tw-scroll-snap-strictness);"
            "snap-mandatory" -> "--tw-scroll-snap-strictness:mandatory;"
            "snap-proximity" -> "--tw-scroll-snap-strictness:proximity;"
            "snap-start" -> "scroll-snap-align:start;"; "snap-end" -> "scroll-snap-align:end;"
            "snap-center" -> "scroll-snap-align:center;"; "snap-align-none" -> "scroll-snap-align:none;"
            "snap-normal" -> "scroll-snap-stop:normal;"; "snap-always" -> "scroll-snap-stop:always;"
            "touch-auto" -> "touch-action:auto;"; "touch-none" -> "touch-action:none;"
            "touch-pan-x" -> "touch-action:pan-x;"; "touch-pan-y" -> "touch-action:pan-y;"
            "touch-pan-left" -> "touch-action:pan-left;"; "touch-pan-right" -> "touch-action:pan-right;"
            "touch-pan-up" -> "touch-action:pan-up;"; "touch-pan-down" -> "touch-action:pan-down;"
            "touch-pinch-zoom" -> "touch-action:pinch-zoom;"; "touch-manipulation" -> "touch-action:manipulation;"
            "appearance-none" -> "appearance:none;"; "appearance-auto" -> "appearance:auto;"
            "will-change-auto" -> "will-change:auto;"; "will-change-scroll" -> "will-change:scroll-position;"
            "will-change-contents" -> "will-change:contents;"; "will-change-transform" -> "will-change:transform;"
            else -> {
                if (c.startsWith("accent-")) { val rest = c.removePrefix("accent-"); if (rest == "auto") return ResolvedUtility.Standard("accent-color:auto;"); resolveColorWithOpacity(rest, "accent-color")?.let { return ResolvedUtility.Standard(it) }; return null }
                if (c.startsWith("caret-")) { resolveColorWithOpacity(c.removePrefix("caret-"), "caret-color")?.let { return ResolvedUtility.Standard(it) }; return null }
                if (c.startsWith("scroll-mx-")) { val v = parseSpacingValue(c.removePrefix("scroll-mx-")) ?: return null; return ResolvedUtility.Standard("scroll-margin-left:$v;scroll-margin-right:$v;") }
                if (c.startsWith("scroll-my-")) { val v = parseSpacingValue(c.removePrefix("scroll-my-")) ?: return null; return ResolvedUtility.Standard("scroll-margin-top:$v;scroll-margin-bottom:$v;") }
                if (c.startsWith("scroll-mt-")) { val v = parseSpacingValue(c.removePrefix("scroll-mt-")) ?: return null; return ResolvedUtility.Standard("scroll-margin-top:$v;") }
                if (c.startsWith("scroll-mr-")) { val v = parseSpacingValue(c.removePrefix("scroll-mr-")) ?: return null; return ResolvedUtility.Standard("scroll-margin-right:$v;") }
                if (c.startsWith("scroll-mb-")) { val v = parseSpacingValue(c.removePrefix("scroll-mb-")) ?: return null; return ResolvedUtility.Standard("scroll-margin-bottom:$v;") }
                if (c.startsWith("scroll-ml-")) { val v = parseSpacingValue(c.removePrefix("scroll-ml-")) ?: return null; return ResolvedUtility.Standard("scroll-margin-left:$v;") }
                if (c.startsWith("scroll-m-")) { val v = parseSpacingValue(c.removePrefix("scroll-m-")) ?: return null; return ResolvedUtility.Standard("scroll-margin:$v;") }
                if (c.startsWith("scroll-px-")) { val v = parseSpacingValue(c.removePrefix("scroll-px-")) ?: return null; return ResolvedUtility.Standard("scroll-padding-left:$v;scroll-padding-right:$v;") }
                if (c.startsWith("scroll-py-")) { val v = parseSpacingValue(c.removePrefix("scroll-py-")) ?: return null; return ResolvedUtility.Standard("scroll-padding-top:$v;scroll-padding-bottom:$v;") }
                if (c.startsWith("scroll-pt-")) { val v = parseSpacingValue(c.removePrefix("scroll-pt-")) ?: return null; return ResolvedUtility.Standard("scroll-padding-top:$v;") }
                if (c.startsWith("scroll-pr-")) { val v = parseSpacingValue(c.removePrefix("scroll-pr-")) ?: return null; return ResolvedUtility.Standard("scroll-padding-right:$v;") }
                if (c.startsWith("scroll-pb-")) { val v = parseSpacingValue(c.removePrefix("scroll-pb-")) ?: return null; return ResolvedUtility.Standard("scroll-padding-bottom:$v;") }
                if (c.startsWith("scroll-pl-")) { val v = parseSpacingValue(c.removePrefix("scroll-pl-")) ?: return null; return ResolvedUtility.Standard("scroll-padding-left:$v;") }
                if (c.startsWith("scroll-p-")) { val v = parseSpacingValue(c.removePrefix("scroll-p-")) ?: return null; return ResolvedUtility.Standard("scroll-padding:$v;") }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    // ── Tables ──────────────────────────────────────────────────────────────

    private fun resolveTables(c: String): ResolvedUtility? {
        val decls = when (c) {
            "table-auto" -> "table-layout:auto;"; "table-fixed" -> "table-layout:fixed;"
            "border-collapse" -> "border-collapse:collapse;"; "border-separate" -> "border-collapse:separate;"
            "caption-top" -> "caption-side:top;"; "caption-bottom" -> "caption-side:bottom;"
            else -> {
                if (c.startsWith("border-spacing-x-")) { val v = parseSpacingValue(c.removePrefix("border-spacing-x-")) ?: return null; return ResolvedUtility.Standard("border-spacing:$v 0;") }
                if (c.startsWith("border-spacing-y-")) { val v = parseSpacingValue(c.removePrefix("border-spacing-y-")) ?: return null; return ResolvedUtility.Standard("border-spacing:0 $v;") }
                if (c.startsWith("border-spacing-")) { val v = parseSpacingValue(c.removePrefix("border-spacing-")) ?: return null; return ResolvedUtility.Standard("border-spacing:$v;") }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    // ── SVG ─────────────────────────────────────────────────────────────────

    private fun resolveSvg(c: String): ResolvedUtility? {
        val decls = when (c) {
            "fill-none" -> "fill:none;"; "fill-current" -> "fill:currentColor;"; "fill-inherit" -> "fill:inherit;"
            "stroke-none" -> "stroke:none;"; "stroke-current" -> "stroke:currentColor;"; "stroke-inherit" -> "stroke:inherit;"
            else -> {
                if (c.startsWith("fill-")) { val hex = VolkiStylePalette.colorHex(c.removePrefix("fill-")) ?: return null; return ResolvedUtility.Standard("fill:$hex;") }
                if (c.startsWith("stroke-")) {
                    val rest = c.removePrefix("stroke-")
                    parseU32(rest)?.let { return ResolvedUtility.Standard("stroke-width:$it;") }
                    VolkiStylePalette.colorHex(rest)?.let { return ResolvedUtility.Standard("stroke:$it;") }
                    return null
                }
                return null
            }
        }
        return ResolvedUtility.Standard(decls)
    }

    // ── Inset ───────────────────────────────────────────────────────────────

    private fun resolveInset(c: String): ResolvedUtility? {
        // Negative inset
        if (c.startsWith("-inset-x-")) { val v = parseSpacingValue(c.removePrefix("-inset-x-")) ?: return null; return ResolvedUtility.Standard("left:-$v;right:-$v;") }
        if (c.startsWith("-inset-y-")) { val v = parseSpacingValue(c.removePrefix("-inset-y-")) ?: return null; return ResolvedUtility.Standard("top:-$v;bottom:-$v;") }
        if (c.startsWith("-inset-")) { val v = parseSpacingValue(c.removePrefix("-inset-")) ?: return null; return ResolvedUtility.Standard("inset:-$v;") }
        if (c.startsWith("-top-")) { val v = parseSpacingValue(c.removePrefix("-top-")) ?: return null; return ResolvedUtility.Standard("top:-$v;") }
        if (c.startsWith("-right-")) { val v = parseSpacingValue(c.removePrefix("-right-")) ?: return null; return ResolvedUtility.Standard("right:-$v;") }
        if (c.startsWith("-bottom-")) { val v = parseSpacingValue(c.removePrefix("-bottom-")) ?: return null; return ResolvedUtility.Standard("bottom:-$v;") }
        if (c.startsWith("-left-")) { val v = parseSpacingValue(c.removePrefix("-left-")) ?: return null; return ResolvedUtility.Standard("left:-$v;") }
        if (c.startsWith("-start-")) { val v = parseSpacingValue(c.removePrefix("-start-")) ?: return null; return ResolvedUtility.Standard("inset-inline-start:-$v;") }
        if (c.startsWith("-end-")) { val v = parseSpacingValue(c.removePrefix("-end-")) ?: return null; return ResolvedUtility.Standard("inset-inline-end:-$v;") }
        // Inset axis
        if (c.startsWith("inset-x-")) { val v = resolveInsetValue(c.removePrefix("inset-x-")) ?: return null; return ResolvedUtility.Standard("left:$v;right:$v;") }
        if (c.startsWith("inset-y-")) { val v = resolveInsetValue(c.removePrefix("inset-y-")) ?: return null; return ResolvedUtility.Standard("top:$v;bottom:$v;") }
        if (c.startsWith("inset-")) { val v = resolveInsetValue(c.removePrefix("inset-")) ?: return null; return ResolvedUtility.Standard("inset:$v;") }
        // Individual sides
        if (c.startsWith("top-")) { val v = resolveInsetValue(c.removePrefix("top-")) ?: return null; return ResolvedUtility.Standard("top:$v;") }
        if (c.startsWith("right-")) { val v = resolveInsetValue(c.removePrefix("right-")) ?: return null; return ResolvedUtility.Standard("right:$v;") }
        if (c.startsWith("bottom-")) { val v = resolveInsetValue(c.removePrefix("bottom-")) ?: return null; return ResolvedUtility.Standard("bottom:$v;") }
        if (c.startsWith("left-")) { val v = resolveInsetValue(c.removePrefix("left-")) ?: return null; return ResolvedUtility.Standard("left:$v;") }
        if (c.startsWith("start-")) { val v = resolveInsetValue(c.removePrefix("start-")) ?: return null; return ResolvedUtility.Standard("inset-inline-start:$v;") }
        if (c.startsWith("end-")) { val v = resolveInsetValue(c.removePrefix("end-")) ?: return null; return ResolvedUtility.Standard("inset-inline-end:$v;") }
        // Z-index
        if (c.startsWith("z-")) { val rest = c.removePrefix("z-"); if (rest == "auto") return ResolvedUtility.Standard("z-index:auto;"); val n = parseU32(rest) ?: return null; return ResolvedUtility.Standard("z-index:$n;") }
        return null
    }

    private fun resolveInsetValue(s: String): String? {
        if (s == "auto") return "auto"
        if (s == "full") return "100%"
        if (s == "px") return "1px"
        parseFraction(s)?.let { return it }
        parseSpacingValue(s)?.let { return it }
        return null
    }
}
