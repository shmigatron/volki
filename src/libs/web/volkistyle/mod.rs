//! volkistyle — Tailwind-like CSS utility classes compiled at build time.

pub mod collector;
pub mod config;
pub mod diagnostics;
pub mod escape;
pub mod palette;
pub mod preflight;
pub mod resolver;
pub mod variants;

use crate::core::volkiwithstds::collections::{String, Vec};

use config::{UnknownClassPolicy, VolkiStyleConfig};
use diagnostics::{GenerateCssReport, StyleDiagnostic, StyleDiagnosticKind};
use escape::escape_selector;
use variants::{parse_variants_with_config, CssRule, ResolvedUtility};

/// Backward-compatible CSS generation with default config.
pub fn generate_css(classes: &[String]) -> String {
    generate_css_with_config(classes, &VolkiStyleConfig::default()).css
}

/// Generate CSS + diagnostics using explicit style config.
pub fn generate_css_with_config(classes: &[String], config: &VolkiStyleConfig) -> GenerateCssReport {
    let mut unique = dedupe_classes(classes);
    for class in config.safelist.iter() {
        if !contains_str(&unique, class.as_str()) {
            unique.push(class.clone());
        }
    }

    let mut rules = Vec::<CssRule>::new();
    let mut bare_utilities = Vec::<String>::new();
    let mut diagnostics = Vec::<StyleDiagnostic>::new();

    let mut resolved_count = 0usize;
    let mut unresolved_count = 0usize;

    for class in unique.iter() {
        let full_class = class.as_str();
        if contains_str(&config.blocklist, full_class) {
            continue;
        }

        let parsed = parse_variants_with_config(full_class, config);

        // custom: prefix — pass-through class, skip resolution entirely
        if parsed.is_custom {
            continue;
        }

        let resolved = match resolver::resolve_declarations(parsed.utility.as_str()) {
            Some(r) => r,
            None => {
                unresolved_count += 1;
                if !has_unknown_diag(&diagnostics, full_class) {
                    diagnostics.push(StyleDiagnostic {
                        class_name: class.clone(),
                        kind: StyleDiagnosticKind::UnknownClass,
                        message: crate::vformat!("unresolved utility class '{}'", full_class),
                    });
                }
                continue;
            }
        };

        resolved_count += 1;
        bare_utilities.push(parsed.utility.clone());

        let escaped_full = escape_selector(full_class);
        match resolved {
            ResolvedUtility::Standard(decls) => {
                let mut selector = String::from(".");
                selector.push_str(escaped_full.as_str());

                for pc in parsed.pseudo_classes.iter() {
                    selector.push_str(pc.as_str());
                }
                for sfx in parsed.selector_suffixes.iter() {
                    selector.push_str(sfx.as_str());
                }
                for pref in parsed.selector_prefixes.iter().rev() {
                    let mut wrapped = pref.clone();
                    wrapped.push_str(selector.as_str());
                    selector = wrapped;
                }

                let final_decls = if parsed.important {
                    make_important(decls.as_str())
                } else {
                    decls
                };

                let media = combine_media_queries(&parsed.media_queries);
                rules.push(CssRule {
                    selector,
                    declarations: final_decls,
                    media: media.clone(),
                    layer: if media.is_some() { 1 } else { 0 },
                });
            }
            ResolvedUtility::Custom { selector_suffix, declarations } => {
                let mut selector = String::from(".");
                selector.push_str(escaped_full.as_str());

                for pc in parsed.pseudo_classes.iter() {
                    selector.push_str(pc.as_str());
                }
                selector.push_str(selector_suffix.as_str());
                for sfx in parsed.selector_suffixes.iter() {
                    selector.push_str(sfx.as_str());
                }
                for pref in parsed.selector_prefixes.iter().rev() {
                    let mut wrapped = pref.clone();
                    wrapped.push_str(selector.as_str());
                    selector = wrapped;
                }

                let final_decls = if parsed.important {
                    make_important(declarations.as_str())
                } else {
                    declarations
                };

                let media = combine_media_queries(&parsed.media_queries);
                rules.push(CssRule {
                    selector,
                    declarations: final_decls,
                    media: media.clone(),
                    layer: if media.is_some() { 1 } else { 0 },
                });
            }
        }
    }

    rules.sort();

    let mut out = String::new();
    if !rules.is_empty() {
        out.push_str(preflight::preflight_css());

        let mut media_groups = Vec::<(String, Vec<usize>)>::new();
        for (i, rule) in rules.iter().enumerate() {
            if let Some(ref mq) = rule.media {
                let mut found = false;
                for group in media_groups.iter_mut() {
                    if group.0.as_str() == mq.as_str() {
                        group.1.push(i);
                        found = true;
                        break;
                    }
                }
                if !found {
                    let mut idxs = Vec::new();
                    idxs.push(i);
                    media_groups.push((mq.clone(), idxs));
                }
            } else {
                out.push_str(rule.selector.as_str());
                out.push_str("{");
                out.push_str(rule.declarations.as_str());
                out.push_str("}");
            }
        }

        for (mq, indices) in media_groups.iter() {
            out.push_str("@media ");
            out.push_str(mq.as_str());
            out.push_str("{");
            for idx in indices.iter() {
                let rule = &rules[*idx];
                out.push_str(rule.selector.as_str());
                out.push_str("{");
                out.push_str(rule.declarations.as_str());
                out.push_str("}");
            }
            out.push_str("}");
        }

        let mut bare_refs = Vec::new();
        for u in bare_utilities.iter() {
            bare_refs.push(u.as_str());
        }
        let keyframes = resolver::transitions::keyframes_css(bare_refs.as_slice());
        if !keyframes.is_empty() {
            out.push_str(keyframes.as_str());
        }
    }

    match config.unknown_class_policy {
        UnknownClassPolicy::Warn | UnknownClassPolicy::Error => {}
        UnknownClassPolicy::Silent => diagnostics.clear(),
    }

    GenerateCssReport {
        css: out,
        diagnostics,
        resolved_count,
        unresolved_count,
    }
}

fn dedupe_classes(classes: &[String]) -> Vec<String> {
    let mut unique = Vec::<String>::new();
    for class in classes {
        if !contains_str(&unique, class.as_str()) {
            unique.push(class.clone());
        }
    }
    unique
}

fn contains_str(list: &[String], needle: &str) -> bool {
    for item in list {
        if item.as_str() == needle {
            return true;
        }
    }
    false
}

fn has_unknown_diag(diags: &[StyleDiagnostic], class_name: &str) -> bool {
    for d in diags {
        if d.kind == StyleDiagnosticKind::UnknownClass && d.class_name.as_str() == class_name {
            return true;
        }
    }
    false
}

fn combine_media_queries(list: &[String]) -> Option<String> {
    if list.is_empty() {
        return None;
    }
    let mut out = String::new();
    for (i, mq) in list.iter().enumerate() {
        if i > 0 {
            out.push_str(" and ");
        }
        out.push_str(mq.as_str());
    }
    Some(out)
}

fn make_important(decls: &str) -> String {
    let mut out = String::new();
    for part in decls.split(';') {
        if part.is_empty() {
            continue;
        }
        out.push_str(part);
        out.push_str(" !important;");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> String {
        String::from(v)
    }

    #[test]
    fn test_generate_css_basic() {
        let classes = crate::vvec![s("flex"), s("p-4")];
        let css = generate_css(&classes);
        assert!(css.as_str().contains(".flex{display:flex;}"));
        assert!(css.as_str().contains(".p-4{padding:1rem;}"));
    }

    #[test]
    fn test_generate_css_deduplicates() {
        let classes = crate::vvec![s("flex"), s("flex")];
        let css = generate_css(&classes);
        assert_eq!(css.as_str().matches(".flex{").count(), 1);
    }

    #[test]
    fn test_unresolved_diagnostic() {
        let classes = crate::vvec![s("definitely-not-real")];
        let report = generate_css_with_config(&classes, &VolkiStyleConfig::default());
        assert_eq!(report.unresolved_count, 1);
        assert_eq!(report.diagnostics.len(), 1);
    }

    #[test]
    fn test_custom_prefix_no_diagnostic() {
        let classes = crate::vvec![s("custom:sidebar-header"), s("custom:badge")];
        let report = generate_css_with_config(&classes, &VolkiStyleConfig::default());
        assert_eq!(report.unresolved_count, 0);
        assert_eq!(report.diagnostics.len(), 0);
    }

    #[test]
    fn test_arbitrary_hex_colors() {
        let classes = crate::vvec![s("bg-[#161b22]"), s("border-[#30363d]"), s("text-[#e6edf3]")];
        let report = generate_css_with_config(&classes, &VolkiStyleConfig::default());
        assert_eq!(report.unresolved_count, 0);
        assert_eq!(report.resolved_count, 3);
        assert!(report.css.as_str().contains("background-color:#161b22;"));
        assert!(report.css.as_str().contains("border-color:#30363d;"));
        assert!(report.css.as_str().contains("color:#e6edf3;"));
    }

    #[test]
    fn test_hover_arbitrary_color() {
        let classes = crate::vvec![s("hover:bg-[#30363d]")];
        let report = generate_css_with_config(&classes, &VolkiStyleConfig::default());
        assert_eq!(report.unresolved_count, 0);
        assert_eq!(report.resolved_count, 1);
        assert!(report.css.as_str().contains("background-color:#30363d;"));
    }
}
