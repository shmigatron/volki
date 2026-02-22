//! Color palette — maps color names to hex values.
//!
//! Supports: white, black, transparent, current, inherit, and shades 50–950 for
//! all 22 Tailwind color families.

/// Resolve a color name (e.g. "red-500", "white") to a hex value.
pub fn color_hex(name: &str) -> Option<&'static str> {
    match name {
        // Absolute colors
        "white" => Some("#ffffff"),
        "black" => Some("#000000"),
        "transparent" => Some("transparent"),

        // Special values
        "current" => Some("currentColor"),
        "inherit" => Some("inherit"),

        // Slate
        "slate-50" => Some("#f8fafc"),
        "slate-100" => Some("#f1f5f9"),
        "slate-200" => Some("#e2e8f0"),
        "slate-300" => Some("#cbd5e1"),
        "slate-400" => Some("#94a3b8"),
        "slate-500" => Some("#64748b"),
        "slate-600" => Some("#475569"),
        "slate-700" => Some("#334155"),
        "slate-800" => Some("#1e293b"),
        "slate-900" => Some("#0f172a"),
        "slate-950" => Some("#020617"),

        // Gray
        "gray-50" => Some("#f9fafb"),
        "gray-100" => Some("#f3f4f6"),
        "gray-200" => Some("#e5e7eb"),
        "gray-300" => Some("#d1d5db"),
        "gray-400" => Some("#9ca3af"),
        "gray-500" => Some("#6b7280"),
        "gray-600" => Some("#4b5563"),
        "gray-700" => Some("#374151"),
        "gray-800" => Some("#1f2937"),
        "gray-900" => Some("#111827"),
        "gray-950" => Some("#030712"),

        // Zinc
        "zinc-50" => Some("#fafafa"),
        "zinc-100" => Some("#f4f4f5"),
        "zinc-200" => Some("#e4e4e7"),
        "zinc-300" => Some("#d4d4d8"),
        "zinc-400" => Some("#a1a1aa"),
        "zinc-500" => Some("#71717a"),
        "zinc-600" => Some("#52525b"),
        "zinc-700" => Some("#3f3f46"),
        "zinc-800" => Some("#27272a"),
        "zinc-900" => Some("#18181b"),
        "zinc-950" => Some("#09090b"),

        // Neutral
        "neutral-50" => Some("#fafafa"),
        "neutral-100" => Some("#f5f5f5"),
        "neutral-200" => Some("#e5e5e5"),
        "neutral-300" => Some("#d4d4d4"),
        "neutral-400" => Some("#a3a3a3"),
        "neutral-500" => Some("#737373"),
        "neutral-600" => Some("#525252"),
        "neutral-700" => Some("#404040"),
        "neutral-800" => Some("#262626"),
        "neutral-900" => Some("#171717"),
        "neutral-950" => Some("#0a0a0a"),

        // Stone
        "stone-50" => Some("#fafaf9"),
        "stone-100" => Some("#f5f5f4"),
        "stone-200" => Some("#e7e5e4"),
        "stone-300" => Some("#d6d3d1"),
        "stone-400" => Some("#a8a29e"),
        "stone-500" => Some("#78716c"),
        "stone-600" => Some("#57534e"),
        "stone-700" => Some("#44403c"),
        "stone-800" => Some("#292524"),
        "stone-900" => Some("#1c1917"),
        "stone-950" => Some("#0c0a09"),

        // Red
        "red-50" => Some("#fef2f2"),
        "red-100" => Some("#fee2e2"),
        "red-200" => Some("#fecaca"),
        "red-300" => Some("#fca5a5"),
        "red-400" => Some("#f87171"),
        "red-500" => Some("#ef4444"),
        "red-600" => Some("#dc2626"),
        "red-700" => Some("#b91c1c"),
        "red-800" => Some("#991b1b"),
        "red-900" => Some("#7f1d1d"),
        "red-950" => Some("#450a0a"),

        // Orange
        "orange-50" => Some("#fff7ed"),
        "orange-100" => Some("#ffedd5"),
        "orange-200" => Some("#fed7aa"),
        "orange-300" => Some("#fdba74"),
        "orange-400" => Some("#fb923c"),
        "orange-500" => Some("#f97316"),
        "orange-600" => Some("#ea580c"),
        "orange-700" => Some("#c2410c"),
        "orange-800" => Some("#9a3412"),
        "orange-900" => Some("#7c2d12"),
        "orange-950" => Some("#431407"),

        // Amber
        "amber-50" => Some("#fffbeb"),
        "amber-100" => Some("#fef3c7"),
        "amber-200" => Some("#fde68a"),
        "amber-300" => Some("#fcd34d"),
        "amber-400" => Some("#fbbf24"),
        "amber-500" => Some("#f59e0b"),
        "amber-600" => Some("#d97706"),
        "amber-700" => Some("#b45309"),
        "amber-800" => Some("#92400e"),
        "amber-900" => Some("#78350f"),
        "amber-950" => Some("#451a03"),

        // Yellow
        "yellow-50" => Some("#fefce8"),
        "yellow-100" => Some("#fef9c3"),
        "yellow-200" => Some("#fef08a"),
        "yellow-300" => Some("#fde047"),
        "yellow-400" => Some("#facc15"),
        "yellow-500" => Some("#eab308"),
        "yellow-600" => Some("#ca8a04"),
        "yellow-700" => Some("#a16207"),
        "yellow-800" => Some("#854d0e"),
        "yellow-900" => Some("#713f12"),
        "yellow-950" => Some("#422006"),

        // Lime
        "lime-50" => Some("#f7fee7"),
        "lime-100" => Some("#ecfccb"),
        "lime-200" => Some("#d9f99d"),
        "lime-300" => Some("#bef264"),
        "lime-400" => Some("#a3e635"),
        "lime-500" => Some("#84cc16"),
        "lime-600" => Some("#65a30d"),
        "lime-700" => Some("#4d7c0f"),
        "lime-800" => Some("#3f6212"),
        "lime-900" => Some("#365314"),
        "lime-950" => Some("#1a2e05"),

        // Green
        "green-50" => Some("#f0fdf4"),
        "green-100" => Some("#dcfce7"),
        "green-200" => Some("#bbf7d0"),
        "green-300" => Some("#86efac"),
        "green-400" => Some("#4ade80"),
        "green-500" => Some("#22c55e"),
        "green-600" => Some("#16a34a"),
        "green-700" => Some("#15803d"),
        "green-800" => Some("#166534"),
        "green-900" => Some("#14532d"),
        "green-950" => Some("#052e16"),

        // Emerald
        "emerald-50" => Some("#ecfdf5"),
        "emerald-100" => Some("#d1fae5"),
        "emerald-200" => Some("#a7f3d0"),
        "emerald-300" => Some("#6ee7b7"),
        "emerald-400" => Some("#34d399"),
        "emerald-500" => Some("#10b981"),
        "emerald-600" => Some("#059669"),
        "emerald-700" => Some("#047857"),
        "emerald-800" => Some("#065f46"),
        "emerald-900" => Some("#064e3b"),
        "emerald-950" => Some("#022c22"),

        // Teal
        "teal-50" => Some("#f0fdfa"),
        "teal-100" => Some("#ccfbf1"),
        "teal-200" => Some("#99f6e4"),
        "teal-300" => Some("#5eead4"),
        "teal-400" => Some("#2dd4bf"),
        "teal-500" => Some("#14b8a6"),
        "teal-600" => Some("#0d9488"),
        "teal-700" => Some("#0f766e"),
        "teal-800" => Some("#115e59"),
        "teal-900" => Some("#134e4a"),
        "teal-950" => Some("#042f2e"),

        // Cyan
        "cyan-50" => Some("#ecfeff"),
        "cyan-100" => Some("#cffafe"),
        "cyan-200" => Some("#a5f3fc"),
        "cyan-300" => Some("#67e8f9"),
        "cyan-400" => Some("#22d3ee"),
        "cyan-500" => Some("#06b6d4"),
        "cyan-600" => Some("#0891b2"),
        "cyan-700" => Some("#0e7490"),
        "cyan-800" => Some("#155e75"),
        "cyan-900" => Some("#164e63"),
        "cyan-950" => Some("#083344"),

        // Sky
        "sky-50" => Some("#f0f9ff"),
        "sky-100" => Some("#e0f2fe"),
        "sky-200" => Some("#bae6fd"),
        "sky-300" => Some("#7dd3fc"),
        "sky-400" => Some("#38bdf8"),
        "sky-500" => Some("#0ea5e9"),
        "sky-600" => Some("#0284c7"),
        "sky-700" => Some("#0369a1"),
        "sky-800" => Some("#075985"),
        "sky-900" => Some("#0c4a6e"),
        "sky-950" => Some("#082f49"),

        // Blue
        "blue-50" => Some("#eff6ff"),
        "blue-100" => Some("#dbeafe"),
        "blue-200" => Some("#bfdbfe"),
        "blue-300" => Some("#93c5fd"),
        "blue-400" => Some("#60a5fa"),
        "blue-500" => Some("#3b82f6"),
        "blue-600" => Some("#2563eb"),
        "blue-700" => Some("#1d4ed8"),
        "blue-800" => Some("#1e40af"),
        "blue-900" => Some("#1e3a8a"),
        "blue-950" => Some("#172554"),

        // Indigo
        "indigo-50" => Some("#eef2ff"),
        "indigo-100" => Some("#e0e7ff"),
        "indigo-200" => Some("#c7d2fe"),
        "indigo-300" => Some("#a5b4fc"),
        "indigo-400" => Some("#818cf8"),
        "indigo-500" => Some("#6366f1"),
        "indigo-600" => Some("#4f46e5"),
        "indigo-700" => Some("#4338ca"),
        "indigo-800" => Some("#3730a3"),
        "indigo-900" => Some("#312e81"),
        "indigo-950" => Some("#1e1b4b"),

        // Violet
        "violet-50" => Some("#f5f3ff"),
        "violet-100" => Some("#ede9fe"),
        "violet-200" => Some("#ddd6fe"),
        "violet-300" => Some("#c4b5fd"),
        "violet-400" => Some("#a78bfa"),
        "violet-500" => Some("#8b5cf6"),
        "violet-600" => Some("#7c3aed"),
        "violet-700" => Some("#6d28d9"),
        "violet-800" => Some("#5b21b6"),
        "violet-900" => Some("#4c1d95"),
        "violet-950" => Some("#2e1065"),

        // Purple
        "purple-50" => Some("#faf5ff"),
        "purple-100" => Some("#f3e8ff"),
        "purple-200" => Some("#e9d5ff"),
        "purple-300" => Some("#d8b4fe"),
        "purple-400" => Some("#c084fc"),
        "purple-500" => Some("#a855f7"),
        "purple-600" => Some("#9333ea"),
        "purple-700" => Some("#7e22ce"),
        "purple-800" => Some("#6b21a8"),
        "purple-900" => Some("#581c87"),
        "purple-950" => Some("#3b0764"),

        // Fuchsia
        "fuchsia-50" => Some("#fdf4ff"),
        "fuchsia-100" => Some("#fae8ff"),
        "fuchsia-200" => Some("#f5d0fe"),
        "fuchsia-300" => Some("#f0abfc"),
        "fuchsia-400" => Some("#e879f9"),
        "fuchsia-500" => Some("#d946ef"),
        "fuchsia-600" => Some("#c026d3"),
        "fuchsia-700" => Some("#a21caf"),
        "fuchsia-800" => Some("#86198f"),
        "fuchsia-900" => Some("#701a75"),
        "fuchsia-950" => Some("#4a044e"),

        // Pink
        "pink-50" => Some("#fdf2f8"),
        "pink-100" => Some("#fce7f3"),
        "pink-200" => Some("#fbcfe8"),
        "pink-300" => Some("#f9a8d4"),
        "pink-400" => Some("#f472b6"),
        "pink-500" => Some("#ec4899"),
        "pink-600" => Some("#db2777"),
        "pink-700" => Some("#be185d"),
        "pink-800" => Some("#9d174d"),
        "pink-900" => Some("#831843"),
        "pink-950" => Some("#500724"),

        // Rose
        "rose-50" => Some("#fff1f2"),
        "rose-100" => Some("#ffe4e6"),
        "rose-200" => Some("#fecdd3"),
        "rose-300" => Some("#fda4af"),
        "rose-400" => Some("#fb7185"),
        "rose-500" => Some("#f43f5e"),
        "rose-600" => Some("#e11d48"),
        "rose-700" => Some("#be123c"),
        "rose-800" => Some("#9f1239"),
        "rose-900" => Some("#881337"),
        "rose-950" => Some("#4c0519"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absolute_colors() {
        assert_eq!(color_hex("white"), Some("#ffffff"));
        assert_eq!(color_hex("black"), Some("#000000"));
        assert_eq!(color_hex("transparent"), Some("transparent"));
    }

    #[test]
    fn test_special_values() {
        assert_eq!(color_hex("current"), Some("currentColor"));
        assert_eq!(color_hex("inherit"), Some("inherit"));
    }

    #[test]
    fn test_shade_colors() {
        assert_eq!(color_hex("red-500"), Some("#ef4444"));
        assert_eq!(color_hex("blue-700"), Some("#1d4ed8"));
        assert_eq!(color_hex("gray-50"), Some("#f9fafb"));
        assert_eq!(color_hex("green-900"), Some("#14532d"));
        assert_eq!(color_hex("teal-400"), Some("#2dd4bf"));
    }

    #[test]
    fn test_shade_950() {
        assert_eq!(color_hex("gray-950"), Some("#030712"));
        assert_eq!(color_hex("red-950"), Some("#450a0a"));
        assert_eq!(color_hex("blue-950"), Some("#172554"));
        assert_eq!(color_hex("slate-950"), Some("#020617"));
    }

    #[test]
    fn test_new_families() {
        assert_eq!(color_hex("slate-500"), Some("#64748b"));
        assert_eq!(color_hex("zinc-500"), Some("#71717a"));
        assert_eq!(color_hex("neutral-500"), Some("#737373"));
        assert_eq!(color_hex("stone-500"), Some("#78716c"));
        assert_eq!(color_hex("amber-500"), Some("#f59e0b"));
        assert_eq!(color_hex("lime-500"), Some("#84cc16"));
        assert_eq!(color_hex("emerald-500"), Some("#10b981"));
        assert_eq!(color_hex("cyan-500"), Some("#06b6d4"));
        assert_eq!(color_hex("sky-500"), Some("#0ea5e9"));
        assert_eq!(color_hex("violet-500"), Some("#8b5cf6"));
        assert_eq!(color_hex("fuchsia-500"), Some("#d946ef"));
        assert_eq!(color_hex("rose-500"), Some("#f43f5e"));
    }

    #[test]
    fn test_unknown_color() {
        assert_eq!(color_hex("magenta-500"), None);
        assert_eq!(color_hex("red"), None);
        assert_eq!(color_hex(""), None);
    }
}
