use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::libs::lang::js::analysis::parser::parse_imports_exports;
use crate::libs::lang::js::analysis::resolver::resolve_import;
use crate::libs::lang::js::analysis::types::{ExportKind, FileInfo, ImportedSymbols};
use crate::libs::lang::js::formatter::walker::{WalkConfig, walk_files};
use crate::libs::lang::shared::license::parsers::json::{JsonValue, extract_top_level};

#[derive(Debug)]
pub struct DeadCodeResult {
    pub unused_files: Vec<PathBuf>,
    pub unused_exports: Vec<UnusedExport>,
    pub unused_imports: Vec<UnusedImport>,
}

#[derive(Debug)]
pub struct UnusedExport {
    pub file: PathBuf,
    pub name: String,
    pub line: usize,
}

#[derive(Debug)]
pub struct UnusedImport {
    pub file: PathBuf,
    pub name: String,
    pub source: String,
    pub line: usize,
}

#[derive(Debug)]
pub enum DeadCodeError {
    Io(io::Error),
    NoSourceFiles(String),
}

impl fmt::Display for DeadCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeadCodeError::Io(e) => write!(f, "IO error: {e}"),
            DeadCodeError::NoSourceFiles(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<io::Error> for DeadCodeError {
    fn from(e: io::Error) -> Self {
        DeadCodeError::Io(e)
    }
}

// --- TypeScript config / path alias support ---

struct TsConfig {
    base_url: PathBuf,
    aliases: HashMap<String, Vec<String>>,
}

fn parse_tsconfig(root: &Path) -> TsConfig {
    let candidates = [
        "tsconfig.json",
        "tsconfig.app.json",
        "tsconfig.build.json",
        "tsconfig.base.json",
    ];

    for candidate in &candidates {
        let path = root.join(candidate);
        if let Some(config) = try_parse_tsconfig_file(root, &path, 0) {
            return config;
        }
    }

    TsConfig {
        base_url: root.to_path_buf(),
        aliases: HashMap::new(),
    }
}

/// Try to parse a single tsconfig file. Follows `extends` up to 3 levels deep.
fn try_parse_tsconfig_file(root: &Path, path: &Path, depth: u32) -> Option<TsConfig> {
    if depth > 3 {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let map = extract_top_level(&content);

    if let Some(compiler_opts) = map.get("compilerOptions").and_then(|v| v.as_object()) {
        let base_url = compiler_opts
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .map(|s| root.join(s))
            .unwrap_or_else(|| root.to_path_buf());

        let mut aliases = HashMap::new();
        if let Some(paths) = compiler_opts.get("paths").and_then(|v| v.as_object()) {
            for (key, value) in paths {
                if let Some(arr) = value.as_array() {
                    let replacements: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    if !replacements.is_empty() {
                        aliases.insert(key.clone(), replacements);
                    }
                }
            }
        }

        if !aliases.is_empty() {
            return Some(TsConfig { base_url, aliases });
        }
    }

    // Follow "extends" reference
    if let Some(extends) = map.get("extends").and_then(|v| v.as_str()) {
        let parent_dir = path.parent().unwrap_or(root);
        let extended_path = parent_dir.join(extends);
        if extended_path.exists() {
            return try_parse_tsconfig_file(root, &extended_path, depth + 1);
        }
        let with_json = extended_path.with_extension("json");
        if with_json.exists() {
            return try_parse_tsconfig_file(root, &with_json, depth + 1);
        }
    }

    None
}

fn resolve_with_aliases(
    source: &str,
    from_file: &Path,
    tsconfig: &TsConfig,
) -> Option<PathBuf> {
    // First try normal relative resolution
    if let Some(resolved) = resolve_import(source, from_file) {
        return Some(resolved);
    }

    // Try alias resolution
    for (pattern, replacements) in &tsconfig.aliases {
        if let Some(prefix) = pattern.strip_suffix('*') {
            if let Some(rest) = source.strip_prefix(prefix) {
                for replacement in replacements {
                    if let Some(rep_prefix) = replacement.strip_suffix('*') {
                        let target = tsconfig.base_url.join(rep_prefix).join(rest);
                        if let Some(resolved) = try_resolve_path(&target) {
                            return Some(resolved);
                        }
                    }
                }
            }
        } else if source == pattern.as_str() {
            for replacement in replacements {
                let target = tsconfig.base_url.join(replacement);
                if let Some(resolved) = try_resolve_path(&target) {
                    return Some(resolved);
                }
            }
        }
    }

    None
}

fn try_resolve_path(target: &Path) -> Option<PathBuf> {
    if target.is_file() {
        return Some(target.canonicalize().unwrap_or_else(|_| target.to_path_buf()));
    }

    for ext in &["ts", "tsx", "js", "jsx", "mjs", "cjs"] {
        let with_ext = target.with_extension(ext);
        if with_ext.is_file() {
            return Some(with_ext.canonicalize().unwrap_or_else(|_| with_ext));
        }
    }

    if target.is_dir() {
        for idx in &["index.ts", "index.tsx", "index.js", "index.jsx"] {
            let index_path = target.join(idx);
            if index_path.is_file() {
                return Some(index_path.canonicalize().unwrap_or_else(|_| index_path));
            }
        }
    }

    None
}

// --- Config file filtering ---

fn is_config_file(path: &Path) -> bool {
    let stem = match path.file_stem() {
        Some(s) => s.to_string_lossy(),
        None => return false,
    };
    stem.ends_with(".config") || stem.ends_with(".setup")
}

// --- Decorator detection ---

/// Check if an exported class has decorators — either class-level (e.g. @Module,
/// @Injectable) or property-level (e.g. @ApiProperty inside the class body).
fn is_decorated_export(source: &str, export_line: usize) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    if export_line == 0 || export_line > lines.len() {
        return false;
    }

    // Only filter decorated class exports
    let export_trimmed = lines[export_line - 1].trim();
    if !export_trimmed.contains("class ") {
        return false;
    }

    // --- Check BEFORE the export for class-level decorators ---
    let window = (export_line - 1).min(100);
    let start = export_line - 1 - window;
    let mut depth: i32 = 0;

    for i in (start..export_line - 1).rev() {
        let trimmed = lines[i].trim();

        for ch in trimmed.chars() {
            match ch {
                '(' | '{' | '[' => depth -= 1,
                ')' | '}' | ']' => depth += 1,
                _ => {}
            }
        }

        if depth <= 0 {
            if trimmed.starts_with('@') && !trimmed.starts_with("@ts-") {
                return true;
            }
            if trimmed.is_empty() {
                continue;
            }
            break;
        }
    }

    // --- Check INSIDE the class body for property decorators ---
    let end = (export_line + 50).min(lines.len());
    let mut brace_depth: i32 = 0;
    let mut entered_body = false;

    // Start from the export line itself (which has the opening `{`)
    for i in (export_line - 1)..end {
        let trimmed = lines[i].trim();
        for ch in trimmed.chars() {
            match ch {
                '{' => {
                    brace_depth += 1;
                    entered_body = true;
                }
                '}' => {
                    brace_depth -= 1;
                    if entered_body && brace_depth <= 0 {
                        return false; // Reached end of class body
                    }
                }
                _ => {}
            }
        }
        // Only check lines inside the class body (after opening brace)
        if entered_body && brace_depth > 0 && i > export_line - 1 {
            if trimmed.starts_with('@') && !trimmed.starts_with("@ts-") {
                return true;
            }
        }
    }

    false
}

fn filter_decorated_exports(exports: Vec<UnusedExport>) -> Vec<UnusedExport> {
    let mut by_file: HashMap<PathBuf, Vec<UnusedExport>> = HashMap::new();
    for exp in exports {
        by_file.entry(exp.file.clone()).or_default().push(exp);
    }

    let mut result = Vec::new();
    for (file, file_exports) in by_file {
        let source = match fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => {
                result.extend(file_exports);
                continue;
            }
        };

        for exp in file_exports {
            if !is_decorated_export(&source, exp.line) {
                result.push(exp);
            }
        }
    }

    result.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    result
}

// --- Re-export chain following ---

/// Trace an imported symbol through re-export chains (barrel files) to find
/// the file that originally defines it.
fn find_export_origin(
    file_infos: &HashMap<PathBuf, FileInfo>,
    file: &PathBuf,
    name: &str,
    tsconfig: &TsConfig,
    visited: &mut HashSet<PathBuf>,
) -> Option<PathBuf> {
    if !visited.insert(file.clone()) {
        return None; // cycle
    }

    let info = file_infos.get(file)?;

    // Check direct (non-reexport) exports
    for exp in &info.exports {
        if exp.name == name && !matches!(exp.kind, ExportKind::ReexportFrom(_)) {
            return Some(file.clone());
        }
    }

    // Follow wildcard and named re-exports
    for exp in &info.exports {
        if let ExportKind::ReexportFrom(ref source) = exp.kind {
            if exp.name == "*" || exp.name == name {
                if let Some(resolved) = resolve_with_aliases(source, &info.path, tsconfig) {
                    if let Some(origin) =
                        find_export_origin(file_infos, &resolved, name, tsconfig, visited)
                    {
                        return Some(origin);
                    }
                }
            }
        }
    }

    None
}

// --- Main detection ---

pub fn detect(root: &Path, entry_points: &[String]) -> Result<DeadCodeResult, DeadCodeError> {
    let config = WalkConfig::default();
    let files = walk_files(root, &config).map_err(DeadCodeError::Io)?;

    if files.is_empty() {
        return Err(DeadCodeError::NoSourceFiles(
            "No JS/TS source files found".to_string(),
        ));
    }

    // Filter out config files (*.config.ts, *.setup.js, etc.)
    let files: Vec<PathBuf> = files
        .into_iter()
        .filter(|f| !is_config_file(f))
        .collect();

    if files.is_empty() {
        return Err(DeadCodeError::NoSourceFiles(
            "No JS/TS source files found".to_string(),
        ));
    }

    // Parse tsconfig for path aliases
    let tsconfig = parse_tsconfig(root);

    // Parse all files
    let mut file_infos: HashMap<PathBuf, FileInfo> = HashMap::new();
    for file in &files {
        let source = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let (imports, exports) = parse_imports_exports(&source);
        let canonical = file.canonicalize().unwrap_or_else(|_| file.clone());
        file_infos.insert(
            canonical,
            FileInfo {
                path: file.clone(),
                imports,
                exports,
            },
        );
    }

    // Resolve entry points
    let mut entries: HashSet<PathBuf> = HashSet::new();
    if entry_points.is_empty() {
        if let Some(detected) = detect_entry_points(root) {
            for e in detected {
                let path = root.join(&e);
                if let Ok(canonical) = path.canonicalize() {
                    entries.insert(canonical);
                }
            }
        }
        // Fallback: if no entries found, treat all files as potentially reachable
        if entries.is_empty() {
            let unused_exports = find_unused_exports(&file_infos, &tsconfig);
            let unused_exports = filter_decorated_exports(unused_exports);
            return Ok(DeadCodeResult {
                unused_files: vec![],
                unused_exports,
                unused_imports: find_unused_imports(&file_infos),
            });
        }
    } else {
        for entry in entry_points {
            let path = root.join(entry);
            if let Ok(canonical) = path.canonicalize() {
                entries.insert(canonical);
            } else {
                let resolved = resolve_import(
                    &format!("./{entry}"),
                    &root.join("__dummy__"),
                );
                if let Some(p) = resolved {
                    entries.insert(p);
                }
            }
        }
    }

    // Build import graph: file -> set of files it imports
    let mut import_graph: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    for (canonical_path, info) in &file_infos {
        let mut deps = HashSet::new();
        for imp in &info.imports {
            if let Some(resolved) = resolve_with_aliases(&imp.source, &info.path, &tsconfig) {
                deps.insert(resolved);
            }
        }
        for exp in &info.exports {
            if let ExportKind::ReexportFrom(ref source) = exp.kind {
                if let Some(resolved) = resolve_with_aliases(source, &info.path, &tsconfig) {
                    deps.insert(resolved);
                }
            }
        }
        import_graph.insert(canonical_path.clone(), deps);
    }

    // BFS from entry points
    let mut reachable: HashSet<PathBuf> = HashSet::new();
    let mut queue: VecDeque<PathBuf> = VecDeque::new();
    for entry in &entries {
        if file_infos.contains_key(entry) {
            queue.push_back(entry.clone());
            reachable.insert(entry.clone());
        }
    }
    while let Some(current) = queue.pop_front() {
        if let Some(deps) = import_graph.get(&current) {
            for dep in deps {
                if reachable.insert(dep.clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }
    }

    // Find unused files
    let all_files: HashSet<PathBuf> = file_infos.keys().cloned().collect();
    let mut unused_files: Vec<PathBuf> = all_files
        .difference(&reachable)
        .filter_map(|p| file_infos.get(p).map(|info| info.path.clone()))
        .collect();
    unused_files.sort();

    // Find unused exports (only in reachable files), then filter decorated
    let unused_exports = find_unused_exports_in(&file_infos, &reachable, &tsconfig);
    let unused_exports = filter_decorated_exports(unused_exports);

    // Find unused imports
    let unused_imports = find_unused_imports(&file_infos);

    Ok(DeadCodeResult {
        unused_files,
        unused_exports,
        unused_imports,
    })
}

fn detect_entry_points(root: &Path) -> Option<Vec<String>> {
    let pkg_json = root.join("package.json");
    let content = fs::read_to_string(&pkg_json).ok()?;
    let map = extract_top_level(&content);

    let mut entries = Vec::new();

    if let Some(main) = map.get("main").and_then(|v| v.as_str()) {
        entries.push(main.to_string());
    }
    if let Some(module) = map.get("module").and_then(|v| v.as_str()) {
        entries.push(module.to_string());
    }
    if let Some(exports) = map.get("exports") {
        collect_export_entries(exports, &mut entries);
    }

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

fn collect_export_entries(value: &JsonValue, entries: &mut Vec<String>) {
    match value {
        JsonValue::Str(s) => entries.push(s.clone()),
        JsonValue::Object(map) => {
            for v in map.values() {
                collect_export_entries(v, entries);
            }
        }
        JsonValue::Array(arr) => {
            for v in arr {
                collect_export_entries(v, entries);
            }
        }
        _ => {}
    }
}

fn find_unused_exports(
    file_infos: &HashMap<PathBuf, FileInfo>,
    tsconfig: &TsConfig,
) -> Vec<UnusedExport> {
    let all_keys: HashSet<PathBuf> = file_infos.keys().cloned().collect();
    find_unused_exports_in(file_infos, &all_keys, tsconfig)
}

fn find_unused_exports_in(
    file_infos: &HashMap<PathBuf, FileInfo>,
    scope: &HashSet<PathBuf>,
    tsconfig: &TsConfig,
) -> Vec<UnusedExport> {
    // Build a map: (file, export_name) -> imported_count
    let mut usage: HashMap<(PathBuf, String), usize> = HashMap::new();

    // Initialize all exports in scope with 0
    for (path, info) in file_infos {
        if !scope.contains(path) {
            continue;
        }
        for exp in &info.exports {
            if matches!(exp.kind, ExportKind::ReexportFrom(_)) {
                continue;
            }
            usage.insert((path.clone(), exp.name.clone()), 0);
        }
    }

    // Count imports (using alias-aware resolution + re-export chain following)
    for info in file_infos.values() {
        for imp in &info.imports {
            let resolved = resolve_with_aliases(&imp.source, &info.path, tsconfig);
            if let Some(target) = resolved {
                match &imp.symbols {
                    ImportedSymbols::Default(_) => {
                        increment_usage_or_follow(
                            &mut usage,
                            file_infos,
                            &target,
                            "default",
                            tsconfig,
                        );
                    }
                    ImportedSymbols::Named(names) => {
                        for name in names {
                            increment_usage_or_follow(
                                &mut usage,
                                file_infos,
                                &target,
                                name,
                                tsconfig,
                            );
                        }
                    }
                    ImportedSymbols::Namespace(_) => {
                        // Namespace import uses all exports of target and its re-exports
                        mark_all_exports_used(&mut usage, file_infos, &target, tsconfig);
                    }
                    ImportedSymbols::SideEffect => {}
                }
            }
        }
    }

    let mut unused: Vec<UnusedExport> = usage
        .into_iter()
        .filter(|(_, count)| *count == 0)
        .filter_map(|((path, name), _)| {
            let info = file_infos.get(&path)?;
            let exp = info.exports.iter().find(|e| e.name == name)?;
            Some(UnusedExport {
                file: info.path.clone(),
                name: name.clone(),
                line: exp.line,
            })
        })
        .collect();

    unused.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    unused
}

/// Try to increment usage for (target, name). If not found directly,
/// follow re-export chains to find the original defining file.
fn increment_usage_or_follow(
    usage: &mut HashMap<(PathBuf, String), usize>,
    file_infos: &HashMap<PathBuf, FileInfo>,
    target: &PathBuf,
    name: &str,
    tsconfig: &TsConfig,
) {
    let key = (target.clone(), name.to_string());
    if let Some(count) = usage.get_mut(&key) {
        *count += 1;
    } else {
        // Follow re-export chain
        let mut visited = HashSet::new();
        if let Some(origin) = find_export_origin(file_infos, target, name, tsconfig, &mut visited) {
            if let Some(count) = usage.get_mut(&(origin, name.to_string())) {
                *count += 1;
            }
        }
    }
}

/// Mark all exports of a file (and transitively re-exported files) as used.
fn mark_all_exports_used(
    usage: &mut HashMap<(PathBuf, String), usize>,
    file_infos: &HashMap<PathBuf, FileInfo>,
    target: &PathBuf,
    tsconfig: &TsConfig,
) {
    let mut visited = HashSet::new();
    mark_all_exports_used_inner(usage, file_infos, target, tsconfig, &mut visited);
}

fn mark_all_exports_used_inner(
    usage: &mut HashMap<(PathBuf, String), usize>,
    file_infos: &HashMap<PathBuf, FileInfo>,
    target: &PathBuf,
    tsconfig: &TsConfig,
    visited: &mut HashSet<PathBuf>,
) {
    if !visited.insert(target.clone()) {
        return;
    }

    // Mark direct exports
    for key in usage.keys().cloned().collect::<Vec<_>>() {
        if key.0 == *target {
            if let Some(count) = usage.get_mut(&key) {
                *count += 1;
            }
        }
    }

    // Follow wildcard re-exports
    if let Some(info) = file_infos.get(target) {
        for exp in &info.exports {
            if let ExportKind::ReexportFrom(ref source) = exp.kind {
                if exp.name == "*" {
                    if let Some(resolved) = resolve_with_aliases(source, &info.path, tsconfig) {
                        mark_all_exports_used_inner(
                            usage, file_infos, &resolved, tsconfig, visited,
                        );
                    }
                }
            }
        }
    }
}

fn find_unused_imports(file_infos: &HashMap<PathBuf, FileInfo>) -> Vec<UnusedImport> {
    let mut unused = Vec::new();

    for info in file_infos.values() {
        let source = match fs::read_to_string(&info.path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for imp in &info.imports {
            let names = match &imp.symbols {
                ImportedSymbols::Default(name) => vec![name.clone()],
                ImportedSymbols::Named(names) => names.clone(),
                ImportedSymbols::Namespace(name) => vec![name.clone()],
                ImportedSymbols::SideEffect => continue,
            };

            for name in names {
                if !is_name_used_in_source(&name, &source, imp.line) {
                    unused.push(UnusedImport {
                        file: info.path.clone(),
                        name,
                        source: imp.source.clone(),
                        line: imp.line,
                    });
                }
            }
        }
    }

    unused.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    unused
}

/// Heuristic: check if an imported name appears as an identifier anywhere in the file
/// beyond the import line itself.
fn is_name_used_in_source(name: &str, source: &str, import_line: usize) -> bool {
    for (line_num, line) in source.lines().enumerate() {
        let line_number = line_num + 1;
        if line_number == import_line {
            continue;
        }
        if contains_identifier(line, name) {
            return true;
        }
    }
    false
}

fn contains_identifier(line: &str, name: &str) -> bool {
    let mut search_from = 0;
    let line_bytes = line.as_bytes();
    let name_bytes = name.as_bytes();

    while let Some(pos) = line[search_from..].find(name) {
        let abs_pos = search_from + pos;
        let before_ok = abs_pos == 0 || !is_ident_char(line_bytes[abs_pos - 1]);
        let after_pos = abs_pos + name_bytes.len();
        let after_ok = after_pos >= line_bytes.len() || !is_ident_char(line_bytes[after_pos]);

        if before_ok && after_ok {
            return true;
        }
        search_from = abs_pos + 1;
    }
    false
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_identifier_exact() {
        assert!(contains_identifier("useState()", "useState"));
    }

    #[test]
    fn contains_identifier_in_middle() {
        assert!(contains_identifier("const x = useState();", "useState"));
    }

    #[test]
    fn contains_identifier_not_substring() {
        assert!(!contains_identifier("useStateHook()", "useState"));
    }

    #[test]
    fn contains_identifier_at_start() {
        assert!(contains_identifier("foo.bar", "foo"));
    }

    #[test]
    fn contains_identifier_not_partial() {
        assert!(!contains_identifier("foobar", "foo"));
    }

    #[test]
    fn dead_code_error_display() {
        let err = DeadCodeError::NoSourceFiles("no files".to_string());
        assert_eq!(format!("{err}"), "no files");

        let err = DeadCodeError::Io(io::Error::new(io::ErrorKind::NotFound, "gone"));
        assert!(format!("{err}").contains("IO error"));
    }

    #[test]
    fn detect_entry_points_from_package_json() {
        let dir = std::env::temp_dir().join(format!(
            "volki_deadcode_entry_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("package.json"),
            r#"{"main": "src/index.js", "module": "src/index.mjs"}"#,
        )
        .unwrap();

        let entries = detect_entry_points(&dir).unwrap();
        assert!(entries.contains(&"src/index.js".to_string()));
        assert!(entries.contains(&"src/index.mjs".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_no_source_files() {
        let dir = std::env::temp_dir().join(format!(
            "volki_deadcode_empty_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = detect(&dir, &[]);
        assert!(matches!(result, Err(DeadCodeError::NoSourceFiles(_))));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_unused_file() {
        let dir = std::env::temp_dir().join(format!(
            "volki_deadcode_unused_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("index.ts"), r#"import { foo } from "./used";"#).unwrap();
        fs::write(dir.join("used.ts"), "export const foo = 1;").unwrap();
        fs::write(dir.join("unused.ts"), "export const bar = 2;").unwrap();

        let result = detect(&dir, &["index.ts".to_string()]).unwrap();
        assert!(result
            .unused_files
            .iter()
            .any(|f| f.file_name().unwrap() == "unused.ts"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_unused_import_heuristic() {
        let dir = std::env::temp_dir().join(format!(
            "volki_deadcode_unimp_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("index.ts"),
            "import { used, notUsed } from \"./lib\";\nconsole.log(used);",
        )
        .unwrap();
        fs::write(
            dir.join("lib.ts"),
            "export const used = 1;\nexport const notUsed = 2;",
        )
        .unwrap();

        let result = detect(&dir, &["index.ts".to_string()]).unwrap();
        assert!(result
            .unused_imports
            .iter()
            .any(|u| u.name == "notUsed"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn is_config_file_matches() {
        assert!(is_config_file(Path::new("jest.config.ts")));
        assert!(is_config_file(Path::new("vite.config.js")));
        assert!(is_config_file(Path::new("next.config.mjs")));
        assert!(is_config_file(Path::new("eslint.config.cjs")));
        assert!(is_config_file(Path::new("vitest.setup.ts")));
        assert!(!is_config_file(Path::new("config.ts")));
        assert!(!is_config_file(Path::new("app.service.ts")));
    }

    #[test]
    fn config_files_excluded_from_scan() {
        let dir = std::env::temp_dir().join(format!(
            "volki_deadcode_config_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("index.ts"), "export const a = 1;").unwrap();
        fs::write(dir.join("jest.config.ts"), "export default {};").unwrap();

        let result = detect(&dir, &[]).unwrap();
        assert!(!result
            .unused_exports
            .iter()
            .any(|e| e.file.to_string_lossy().contains("jest.config")));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn decorated_export_class_level() {
        let source = "@Injectable()\nexport class FooService {}";
        assert!(is_decorated_export(source, 2));
    }

    #[test]
    fn decorated_export_multiline() {
        let source =
            "@Module({\n  imports: [FooModule],\n  providers: [FooService],\n})\nexport class AppModule {}";
        assert!(is_decorated_export(source, 5));
    }

    #[test]
    fn decorated_export_property_decorators() {
        let source = "export class CreateInput {\n  @ApiProperty()\n  name: string;\n}";
        assert!(is_decorated_export(source, 1));
    }

    #[test]
    fn non_decorated_export() {
        let source = "// a comment\nexport class PlainClass {}";
        assert!(!is_decorated_export(source, 2));
    }

    #[test]
    fn decorated_non_class_not_filtered() {
        let source = "@deprecated\nexport const FOO = 1;";
        assert!(!is_decorated_export(source, 2));
    }

    #[test]
    fn tsconfig_alias_resolution() {
        let dir = std::env::temp_dir().join(format!(
            "volki_deadcode_alias_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src/utils")).unwrap();

        fs::write(
            dir.join("tsconfig.json"),
            r#"{"compilerOptions": {"baseUrl": ".", "paths": {"@/*": ["src/*"]}}}"#,
        )
        .unwrap();
        fs::write(
            dir.join("src/index.ts"),
            "import { helper } from \"@/utils/helper\";",
        )
        .unwrap();
        fs::write(
            dir.join("src/utils/helper.ts"),
            "export const helper = 1;",
        )
        .unwrap();

        let result = detect(&dir, &["src/index.ts".to_string()]).unwrap();
        assert!(!result
            .unused_files
            .iter()
            .any(|f| f.to_string_lossy().contains("helper")));
        assert!(!result
            .unused_exports
            .iter()
            .any(|e| e.name == "helper"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn tsconfig_extends_resolution() {
        let dir = std::env::temp_dir().join(format!(
            "volki_deadcode_extends_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();

        fs::write(
            dir.join("tsconfig.base.json"),
            r#"{"compilerOptions": {"baseUrl": ".", "paths": {"@/*": ["src/*"]}}}"#,
        )
        .unwrap();
        fs::write(
            dir.join("tsconfig.json"),
            r#"{"extends": "./tsconfig.base.json", "compilerOptions": {}}"#,
        )
        .unwrap();

        let config = parse_tsconfig(&dir);
        assert!(!config.aliases.is_empty());
        assert!(config.aliases.contains_key("@/*"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn barrel_reexport_counts_as_used() {
        let dir = std::env::temp_dir().join(format!(
            "volki_deadcode_barrel_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("lib")).unwrap();

        // actual.ts defines the export
        fs::write(dir.join("lib/actual.ts"), "export const foo = 1;").unwrap();
        // barrel re-exports everything
        fs::write(
            dir.join("lib/index.ts"),
            "export * from \"./actual\";",
        )
        .unwrap();
        // consumer imports through barrel
        fs::write(
            dir.join("index.ts"),
            "import { foo } from \"./lib\";\nconsole.log(foo);",
        )
        .unwrap();

        let result = detect(&dir, &["index.ts".to_string()]).unwrap();
        // foo should NOT be reported as unused
        assert!(
            !result.unused_exports.iter().any(|e| e.name == "foo"),
            "foo should be counted as used through barrel re-export"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
