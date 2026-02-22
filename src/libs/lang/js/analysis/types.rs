use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Import {
    pub source: String,
    pub symbols: ImportedSymbols,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum ImportedSymbols {
    Default(String),
    Named(Vec<String>),
    Namespace(String),
    SideEffect,
}

#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub kind: ExportKind,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum ExportKind {
    Named,
    Default,
    ReexportFrom(String),
}

#[derive(Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vvec;

    #[test]
    fn import_debug() {
        let imp = Import {
            source: crate::vstr!("react"),
            symbols: ImportedSymbols::Default(crate::vstr!("React")),
            line: 1,
        };
        let dbg = crate::vformat!("{imp:?}");
        assert!(dbg.contains("react"));
    }

    #[test]
    fn export_debug() {
        let exp = Export {
            name: crate::vstr!("foo"),
            kind: ExportKind::Named,
            line: 5,
        };
        let dbg = crate::vformat!("{exp:?}");
        assert!(dbg.contains("foo"));
    }

    #[test]
    fn file_info_debug() {
        let info = FileInfo {
            path: PathBuf::from("src/index.ts"),
            imports: vvec![],
            exports: vvec![],
        };
        let dbg = crate::vformat!("{info:?}");
        assert!(dbg.contains("index.ts"));
    }
}
