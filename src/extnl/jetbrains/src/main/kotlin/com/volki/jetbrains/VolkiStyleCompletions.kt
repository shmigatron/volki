package com.volki.jetbrains

object VolkiStyleCompletions {

    data class CompletionEntry(val className: String, val category: String, val cssPreview: String)
    data class DynamicPrefix(val prefix: String, val category: String, val examples: List<String>, val cssProperty: String)

    val ALL_ENTRIES: List<CompletionEntry> by lazy { buildStaticEntries() }

    val DYNAMIC_PREFIXES: List<DynamicPrefix> by lazy { buildDynamicPrefixes() }

    val COLOR_PREFIXES = listOf("bg-", "text-", "border-", "ring-", "shadow-", "fill-", "stroke-",
        "accent-", "caret-", "decoration-", "from-", "via-", "to-", "divide-", "outline-", "ring-offset-")

    private fun buildStaticEntries(): List<CompletionEntry> {
        val entries = mutableListOf<CompletionEntry>()
        val candidates = listOf(
            // Layout
            "block", "inline", "inline-block", "flex", "inline-flex", "grid", "inline-grid",
            "hidden", "table", "table-row", "table-cell", "contents", "list-item", "flow-root", "container",
            "relative", "absolute", "fixed", "sticky", "static",
            "float-right", "float-left", "float-none", "clear-left", "clear-right", "clear-both", "clear-none",
            "visible", "invisible", "collapse",
            "box-border", "box-content", "isolate", "isolation-auto",
            "aspect-auto", "aspect-square", "aspect-video",
            "object-contain", "object-cover", "object-fill", "object-none", "object-scale-down",
            "object-center", "object-top", "object-bottom", "object-left", "object-right",
            "overflow-hidden", "overflow-auto", "overflow-scroll", "overflow-visible", "overflow-clip",
            "overflow-x-auto", "overflow-y-auto", "overflow-x-hidden", "overflow-y-hidden",
            "overscroll-auto", "overscroll-contain", "overscroll-none",
            "sr-only", "not-sr-only",
            // Flexbox
            "flex-row", "flex-col", "flex-row-reverse", "flex-col-reverse",
            "flex-wrap", "flex-nowrap", "flex-wrap-reverse",
            "flex-1", "flex-auto", "flex-initial", "flex-none",
            "flex-grow", "flex-grow-0", "flex-shrink", "flex-shrink-0",
            "items-center", "items-start", "items-end", "items-stretch", "items-baseline",
            "justify-center", "justify-between", "justify-around", "justify-evenly",
            "justify-start", "justify-end", "justify-normal", "justify-stretch",
            "self-auto", "self-start", "self-end", "self-center", "self-stretch", "self-baseline",
            "content-center", "content-start", "content-end", "content-between",
            "place-content-center", "place-items-center", "place-self-center",
            // Grid
            "grid-flow-row", "grid-flow-col", "grid-flow-dense",
            "auto-cols-auto", "auto-cols-min", "auto-cols-max", "auto-cols-fr",
            "auto-rows-auto", "auto-rows-min", "auto-rows-max", "auto-rows-fr",
            "col-auto", "col-span-full", "row-auto", "row-span-full",
            // Typography
            "text-left", "text-center", "text-right", "text-justify",
            "uppercase", "lowercase", "capitalize", "normal-case",
            "italic", "not-italic",
            "underline", "no-underline", "line-through", "overline",
            "decoration-solid", "decoration-dashed", "decoration-dotted", "decoration-double", "decoration-wavy",
            "truncate", "text-ellipsis", "text-clip",
            "whitespace-normal", "whitespace-nowrap", "whitespace-pre", "whitespace-pre-line", "whitespace-pre-wrap",
            "break-normal", "break-all", "break-keep", "break-words",
            "text-wrap", "text-nowrap", "text-balance", "text-pretty",
            "font-sans", "font-serif", "font-mono",
            "font-thin", "font-extralight", "font-light", "font-normal", "font-medium",
            "font-semibold", "font-bold", "font-extrabold", "font-black",
            "text-xs", "text-sm", "text-base", "text-lg", "text-xl",
            "text-2xl", "text-3xl", "text-4xl", "text-5xl", "text-6xl", "text-7xl", "text-8xl", "text-9xl",
            "leading-none", "leading-tight", "leading-snug", "leading-normal", "leading-relaxed", "leading-loose",
            "tracking-tighter", "tracking-tight", "tracking-normal", "tracking-wide", "tracking-wider", "tracking-widest",
            "list-none", "list-disc", "list-decimal", "list-inside", "list-outside",
            "align-baseline", "align-top", "align-middle", "align-bottom",
            // Backgrounds
            "bg-gradient-to-t", "bg-gradient-to-r", "bg-gradient-to-b", "bg-gradient-to-l",
            "bg-gradient-to-tr", "bg-gradient-to-br", "bg-gradient-to-bl", "bg-gradient-to-tl",
            "bg-none", "bg-auto", "bg-cover", "bg-contain",
            "bg-center", "bg-top", "bg-right", "bg-bottom", "bg-left",
            "bg-repeat", "bg-no-repeat", "bg-repeat-x", "bg-repeat-y",
            "bg-fixed", "bg-local", "bg-scroll",
            "bg-clip-border", "bg-clip-padding", "bg-clip-content", "bg-clip-text",
            // Borders
            "border", "border-0", "border-2", "border-4", "border-8",
            "border-t", "border-r", "border-b", "border-l", "border-x", "border-y",
            "border-solid", "border-dashed", "border-dotted", "border-double", "border-hidden", "border-none",
            "rounded", "rounded-none", "rounded-sm", "rounded-md", "rounded-lg", "rounded-xl",
            "rounded-2xl", "rounded-3xl", "rounded-full",
            "outline-none", "outline", "outline-dashed", "outline-dotted", "outline-double",
            "ring", "ring-0", "ring-1", "ring-2", "ring-4", "ring-8", "ring-inset",
            "divide-x", "divide-y", "divide-x-0", "divide-y-0", "divide-x-2", "divide-y-2",
            // Effects
            "shadow", "shadow-sm", "shadow-md", "shadow-lg", "shadow-xl", "shadow-2xl", "shadow-inner", "shadow-none",
            "mix-blend-normal", "mix-blend-multiply", "mix-blend-screen", "mix-blend-overlay",
            // Filters
            "blur", "blur-none", "blur-sm", "blur-md", "blur-lg", "blur-xl", "blur-2xl", "blur-3xl",
            "grayscale", "grayscale-0", "invert", "invert-0", "sepia", "sepia-0",
            "drop-shadow", "drop-shadow-sm", "drop-shadow-md", "drop-shadow-lg", "drop-shadow-xl", "drop-shadow-none",
            "backdrop-blur", "backdrop-blur-sm", "backdrop-blur-md", "backdrop-blur-lg",
            // Transitions
            "transition", "transition-none", "transition-all", "transition-colors", "transition-opacity",
            "transition-shadow", "transition-transform",
            "ease-linear", "ease-in", "ease-out", "ease-in-out",
            "animate-none", "animate-spin", "animate-ping", "animate-pulse", "animate-bounce",
            // Interactivity
            "cursor-auto", "cursor-default", "cursor-pointer", "cursor-wait", "cursor-text",
            "cursor-move", "cursor-help", "cursor-not-allowed", "cursor-none",
            "cursor-grab", "cursor-grabbing",
            "resize-none", "resize-y", "resize-x", "resize",
            "select-none", "select-text", "select-all", "select-auto",
            "pointer-events-none", "pointer-events-auto",
            "scroll-auto", "scroll-smooth",
            "snap-none", "snap-x", "snap-y", "snap-both",
            "snap-start", "snap-end", "snap-center",
            "touch-auto", "touch-none", "touch-manipulation",
            "appearance-none", "appearance-auto",
            "will-change-auto", "will-change-scroll", "will-change-contents", "will-change-transform",
            // Transforms
            "origin-center", "origin-top", "origin-bottom", "origin-left", "origin-right",
            "origin-top-left", "origin-top-right", "origin-bottom-left", "origin-bottom-right",
            // Tables
            "table-auto", "table-fixed", "border-collapse", "border-separate",
            "caption-top", "caption-bottom",
            // SVG
            "fill-none", "fill-current", "fill-inherit",
            "stroke-none", "stroke-current", "stroke-inherit",
        )
        for (name in candidates) {
            val resolved = VolkiStyleResolver.resolve(name) ?: continue
            val cat = VolkiStyleResolver.category(name) ?: "Other"
            entries.add(CompletionEntry(name, cat, resolved.declarationsText()))
        }
        return entries
    }

    private fun buildDynamicPrefixes(): List<DynamicPrefix> = listOf(
        DynamicPrefix("p-", "Spacing", listOf("0", "1", "2", "3", "4", "5", "6", "8", "10", "12", "16", "20"), "padding"),
        DynamicPrefix("px-", "Spacing", listOf("0", "1", "2", "4", "6", "8"), "padding-left/right"),
        DynamicPrefix("py-", "Spacing", listOf("0", "1", "2", "4", "6", "8"), "padding-top/bottom"),
        DynamicPrefix("pt-", "Spacing", listOf("0", "1", "2", "4", "8"), "padding-top"),
        DynamicPrefix("pr-", "Spacing", listOf("0", "1", "2", "4", "8"), "padding-right"),
        DynamicPrefix("pb-", "Spacing", listOf("0", "1", "2", "4", "8"), "padding-bottom"),
        DynamicPrefix("pl-", "Spacing", listOf("0", "1", "2", "4", "8"), "padding-left"),
        DynamicPrefix("m-", "Spacing", listOf("0", "1", "2", "4", "8", "auto"), "margin"),
        DynamicPrefix("mx-", "Spacing", listOf("0", "1", "2", "4", "auto"), "margin-left/right"),
        DynamicPrefix("my-", "Spacing", listOf("0", "1", "2", "4", "auto"), "margin-top/bottom"),
        DynamicPrefix("mt-", "Spacing", listOf("0", "1", "2", "4", "8", "auto"), "margin-top"),
        DynamicPrefix("mb-", "Spacing", listOf("0", "1", "2", "4", "8", "auto"), "margin-bottom"),
        DynamicPrefix("ml-", "Spacing", listOf("0", "1", "2", "4", "auto"), "margin-left"),
        DynamicPrefix("mr-", "Spacing", listOf("0", "1", "2", "4", "auto"), "margin-right"),
        DynamicPrefix("gap-", "Spacing", listOf("0", "1", "2", "4", "6", "8"), "gap"),
        DynamicPrefix("gap-x-", "Spacing", listOf("0", "1", "2", "4"), "column-gap"),
        DynamicPrefix("gap-y-", "Spacing", listOf("0", "1", "2", "4"), "row-gap"),
        DynamicPrefix("space-x-", "Spacing", listOf("0", "1", "2", "4", "8"), "margin-left (children)"),
        DynamicPrefix("space-y-", "Spacing", listOf("0", "1", "2", "4", "8"), "margin-top (children)"),
        DynamicPrefix("w-", "Sizing", listOf("0", "1", "4", "8", "12", "16", "24", "32", "48", "64", "full", "screen", "auto", "1/2", "1/3", "1/4"), "width"),
        DynamicPrefix("h-", "Sizing", listOf("0", "1", "4", "8", "12", "16", "24", "32", "48", "64", "full", "screen", "auto"), "height"),
        DynamicPrefix("size-", "Sizing", listOf("0", "1", "4", "8", "full", "auto"), "width + height"),
        DynamicPrefix("max-w-", "Sizing", listOf("none", "xs", "sm", "md", "lg", "xl", "2xl", "full", "prose"), "max-width"),
        DynamicPrefix("min-w-", "Sizing", listOf("0", "full", "min", "max", "fit"), "min-width"),
        DynamicPrefix("max-h-", "Sizing", listOf("full", "screen", "min", "max", "fit"), "max-height"),
        DynamicPrefix("min-h-", "Sizing", listOf("0", "full", "screen"), "min-height"),
        DynamicPrefix("grid-cols-", "Grid", listOf("1", "2", "3", "4", "5", "6", "12", "none"), "grid-template-columns"),
        DynamicPrefix("grid-rows-", "Grid", listOf("1", "2", "3", "6", "none"), "grid-template-rows"),
        DynamicPrefix("col-span-", "Grid", listOf("1", "2", "3", "4", "6", "12", "full"), "grid-column"),
        DynamicPrefix("row-span-", "Grid", listOf("1", "2", "3", "6", "full"), "grid-row"),
        DynamicPrefix("opacity-", "Effects", listOf("0", "5", "10", "20", "25", "50", "75", "100"), "opacity"),
        DynamicPrefix("duration-", "Transitions", listOf("75", "100", "150", "200", "300", "500", "700", "1000"), "transition-duration"),
        DynamicPrefix("delay-", "Transitions", listOf("75", "100", "150", "200", "300", "500"), "transition-delay"),
        DynamicPrefix("scale-", "Transforms", listOf("0", "50", "75", "90", "95", "100", "105", "110", "125", "150"), "transform:scale"),
        DynamicPrefix("rotate-", "Transforms", listOf("0", "1", "2", "3", "6", "12", "45", "90", "180"), "transform:rotate"),
        DynamicPrefix("translate-x-", "Transforms", listOf("0", "1", "2", "4", "8", "1/2", "full"), "transform:translateX"),
        DynamicPrefix("translate-y-", "Transforms", listOf("0", "1", "2", "4", "8", "1/2", "full"), "transform:translateY"),
        DynamicPrefix("z-", "Positioning", listOf("0", "10", "20", "30", "40", "50", "auto"), "z-index"),
        DynamicPrefix("inset-", "Positioning", listOf("0", "1", "2", "4", "auto", "full", "1/2"), "inset"),
        DynamicPrefix("top-", "Positioning", listOf("0", "1", "2", "4", "auto", "full", "1/2"), "top"),
        DynamicPrefix("right-", "Positioning", listOf("0", "1", "2", "4", "auto"), "right"),
        DynamicPrefix("bottom-", "Positioning", listOf("0", "1", "2", "4", "auto"), "bottom"),
        DynamicPrefix("left-", "Positioning", listOf("0", "1", "2", "4", "auto"), "left"),
        DynamicPrefix("brightness-", "Filters", listOf("0", "50", "75", "90", "95", "100", "105", "110", "125", "150"), "filter:brightness"),
        DynamicPrefix("contrast-", "Filters", listOf("0", "50", "75", "100", "125", "150", "200"), "filter:contrast"),
        DynamicPrefix("saturate-", "Filters", listOf("0", "50", "100", "150", "200"), "filter:saturate"),
        DynamicPrefix("hue-rotate-", "Filters", listOf("0", "15", "30", "60", "90", "180"), "filter:hue-rotate"),
        DynamicPrefix("basis-", "Sizing", listOf("0", "1", "2", "4", "8", "auto", "full", "1/2", "1/3"), "flex-basis"),
        DynamicPrefix("columns-", "Layout", listOf("1", "2", "3", "4", "auto", "sm", "md", "lg"), "columns"),
        DynamicPrefix("order-", "Flexbox", listOf("1", "2", "3", "4", "5", "first", "last", "none"), "order"),
        DynamicPrefix("leading-", "Typography", listOf("3", "4", "5", "6", "7", "8", "none", "tight", "snug", "normal", "relaxed", "loose"), "line-height"),
        DynamicPrefix("indent-", "Typography", listOf("0", "1", "2", "4", "8"), "text-indent"),
        DynamicPrefix("stroke-", "SVG", listOf("0", "1", "2"), "stroke-width"),
        DynamicPrefix("border-spacing-", "Tables", listOf("0", "1", "2", "4"), "border-spacing"),
        DynamicPrefix("line-clamp-", "Typography", listOf("1", "2", "3", "4", "5", "6", "none"), "-webkit-line-clamp"),
        DynamicPrefix("underline-offset-", "Typography", listOf("auto", "0", "1", "2", "4", "8"), "text-underline-offset"),
        DynamicPrefix("decoration-", "Typography", listOf("0", "1", "2", "4", "8"), "text-decoration-thickness"),
        DynamicPrefix("rounded-t-", "Borders", listOf("none", "sm", "md", "lg", "xl", "2xl", "full"), "border-top-radius"),
        DynamicPrefix("rounded-b-", "Borders", listOf("none", "sm", "md", "lg", "xl", "2xl", "full"), "border-bottom-radius"),
        DynamicPrefix("rounded-l-", "Borders", listOf("none", "sm", "md", "lg", "xl", "2xl", "full"), "border-left-radius"),
        DynamicPrefix("rounded-r-", "Borders", listOf("none", "sm", "md", "lg", "xl", "2xl", "full"), "border-right-radius"),
    )
}
