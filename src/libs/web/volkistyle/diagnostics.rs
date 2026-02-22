use crate::core::volkiwithstds::collections::{String, Vec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleDiagnosticKind {
    UnknownClass,
}

#[derive(Debug, Clone)]
pub struct StyleDiagnostic {
    pub class_name: String,
    pub kind: StyleDiagnosticKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct GenerateCssReport {
    pub css: String,
    pub diagnostics: Vec<StyleDiagnostic>,
    pub resolved_count: usize,
    pub unresolved_count: usize,
}
