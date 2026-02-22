use crate::core::config::parser::Table;
use crate::core::volkiwithstds::collections::{HashMap, String, Vec};
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownClassPolicy {
    Warn,
    Error,
    Silent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DarkModeStrategy {
    Media,
    Class,
}

#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub screens: HashMap<String, String>,
    pub colors: HashMap<String, String>,
    pub spacing: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct VariantConfig {
    pub enable_data_aria: bool,
    pub enable_supports: bool,
    pub enable_group_peer_named: bool,
}

#[derive(Debug, Clone)]
pub struct VolkiStyleConfig {
    pub unknown_class_policy: UnknownClassPolicy,
    pub dark_mode: DarkModeStrategy,
    pub safelist: Vec<String>,
    pub blocklist: Vec<String>,
    pub theme: ThemeConfig,
    pub variants: VariantConfig,
}

impl Default for VolkiStyleConfig {
    fn default() -> Self {
        let mut screens = HashMap::new();
        screens.insert(String::from("sm"), String::from("640px"));
        screens.insert(String::from("md"), String::from("768px"));
        screens.insert(String::from("lg"), String::from("1024px"));
        screens.insert(String::from("xl"), String::from("1280px"));
        screens.insert(String::from("2xl"), String::from("1536px"));

        Self {
            unknown_class_policy: UnknownClassPolicy::Warn,
            dark_mode: DarkModeStrategy::Media,
            safelist: Vec::new(),
            blocklist: Vec::new(),
            theme: ThemeConfig {
                screens,
                colors: HashMap::new(),
                spacing: HashMap::new(),
            },
            variants: VariantConfig {
                enable_data_aria: true,
                enable_supports: true,
                enable_group_peer_named: true,
            },
        }
    }
}

pub fn load_for_source_file(file: &Path) -> VolkiStyleConfig {
    let mut cfg = VolkiStyleConfig::default();
    if let Some(path) = find_volki_toml(file) {
        if let Ok(content) = fs::read_to_string(path.as_path()) {
            if let Ok(table) = crate::core::config::parser::parse(content.as_str()) {
                apply_table(&mut cfg, &table);
            }
        }
    }

    if let Some(v) = crate::core::volkiwithstds::env::var("VOLKI_WEB_STRICT_CLASSES") {
        if v == "1" || v.eq_ignore_ascii_case("true") {
            cfg.unknown_class_policy = UnknownClassPolicy::Error;
        }
    }

    cfg
}

fn find_volki_toml(file: &Path) -> Option<PathBuf> {
    let mut dir = if file.is_dir() {
        Some(file.to_path_buf())
    } else {
        file.parent().map(|p| p.to_path_buf())
    };

    while let Some(current) = dir {
        let candidate = current.join("volki.toml");
        if candidate.as_path().exists() {
            return Some(candidate);
        }
        dir = current.as_path().parent().map(|p| p.to_path_buf());
    }

    None
}

fn apply_table(cfg: &mut VolkiStyleConfig, table: &Table) {
    if let Some(v) = table.get("web.volkistyle", "unknown_class_policy").and_then(|v| v.as_str()) {
        cfg.unknown_class_policy = match v {
            "error" => UnknownClassPolicy::Error,
            "silent" => UnknownClassPolicy::Silent,
            _ => UnknownClassPolicy::Warn,
        };
    }

    if let Some(v) = table.get("web.volkistyle", "dark_mode").and_then(|v| v.as_str()) {
        cfg.dark_mode = match v {
            "class" => DarkModeStrategy::Class,
            _ => DarkModeStrategy::Media,
        };
    }

    if let Some(v) = table.get("web.volkistyle", "safelist").and_then(|v| v.as_str_array()) {
        let mut list = Vec::new();
        for item in v {
            list.push(String::from(item));
        }
        cfg.safelist = list;
    }

    if let Some(v) = table.get("web.volkistyle", "blocklist").and_then(|v| v.as_str_array()) {
        let mut list = Vec::new();
        for item in v {
            list.push(String::from(item));
        }
        cfg.blocklist = list;
    }

    if let Some(v) = table.get("web.volkistyle.variants", "data_aria").and_then(|v| v.as_bool()) {
        cfg.variants.enable_data_aria = v;
    }
    if let Some(v) = table.get("web.volkistyle.variants", "supports").and_then(|v| v.as_bool()) {
        cfg.variants.enable_supports = v;
    }
    if let Some(v) = table.get("web.volkistyle.variants", "group_peer_named").and_then(|v| v.as_bool()) {
        cfg.variants.enable_group_peer_named = v;
    }

    for (k, v) in table.entries_with_prefix("web.volkistyle.theme.screens") {
        cfg.theme.screens.insert(k, v);
    }
    for (k, v) in table.entries_with_prefix("web.volkistyle.theme.colors") {
        cfg.theme.colors.insert(k, v);
    }
    for (k, v) in table.entries_with_prefix("web.volkistyle.theme.spacing") {
        cfg.theme.spacing.insert(k, v);
    }
}
