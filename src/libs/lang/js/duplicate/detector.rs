use crate::core::volkiwithstds::collections::HashMap;
use crate::core::volkiwithstds::collections::{String, Vec};
use crate::core::volkiwithstds::fmt;
use crate::core::volkiwithstds::fs;
use crate::core::volkiwithstds::io;
use crate::core::volkiwithstds::path::{Path, PathBuf};

use crate::libs::lang::js::formatter::tokenizer::{TokenKind, tokenize};
use crate::libs::lang::js::formatter::walker::{WalkConfig, walk_files};
use crate::vvec;

#[derive(Debug)]
pub struct DuplicateResult {
    pub clones: Vec<CloneGroup>,
    pub total_duplicated_lines: usize,
}

#[derive(Debug)]
pub struct CloneGroup {
    pub instances: Vec<CloneInstance>,
    pub token_count: usize,
}

#[derive(Debug, Clone)]
pub struct CloneInstance {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug)]
pub enum DuplicateError {
    Io(io::IoError),
    NoSourceFiles(String),
}

impl fmt::Display for DuplicateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DuplicateError::Io(e) => write!(f, "IO error: {e}"),
            DuplicateError::NoSourceFiles(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<io::IoError> for DuplicateError {
    fn from(e: io::IoError) -> Self {
        DuplicateError::Io(e)
    }
}

struct NormalizedToken {
    kind: TokenKind,
    line: usize,
}

pub fn detect(root: &Path, min_tokens: usize) -> Result<DuplicateResult, DuplicateError> {
    let config = WalkConfig::default();
    let files = walk_files(root, &config).map_err(DuplicateError::Io)?;

    if files.is_empty() {
        return Err(DuplicateError::NoSourceFiles(crate::vstr!(
            "No JS/TS source files found"
        )));
    }

    let mut all_sequences: Vec<(PathBuf, Vec<NormalizedToken>)> = Vec::new();

    for file in &files {
        let source = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let tokens = match tokenize(&source) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let normalized: Vec<NormalizedToken> = tokens
            .into_iter()
            .filter(|t| {
                !matches!(
                    t.kind,
                    TokenKind::Whitespace
                        | TokenKind::Newline
                        | TokenKind::LineComment
                        | TokenKind::BlockComment
                        | TokenKind::Eof
                )
            })
            .map(|t| NormalizedToken {
                kind: normalize_kind(&t.kind),
                line: t.line,
            })
            .collect();

        if normalized.len() >= min_tokens {
            all_sequences.push((file.clone(), normalized));
        }
    }

    // Build fingerprint index: hash -> Vec<(file_idx, token_start_idx)>
    let mut hash_index: HashMap<u64, Vec<(usize, usize)>> = HashMap::new();

    for (file_idx, (_, tokens)) in all_sequences.iter().enumerate() {
        if tokens.len() < min_tokens {
            continue;
        }
        for start in 0..=tokens.len() - min_tokens {
            let hash = compute_hash(&tokens.as_slice()[start..start + min_tokens]);
            hash_index.entry(hash).or_default().push((file_idx, start));
        }
    }

    let mut clone_groups: Vec<CloneGroup> = Vec::new();
    let mut seen_pairs: HashMap<(usize, usize, usize, usize), bool> = HashMap::new();

    for locations in hash_index.values() {
        if locations.len() < 2 {
            continue;
        }

        for i in 0..locations.len() {
            for j in (i + 1)..locations.len() {
                let (fi, si) = locations[i];
                let (fj, sj) = locations[j];

                if fi == fj && overlaps(si, sj, min_tokens) {
                    continue;
                }

                let key = if (fi, si) < (fj, sj) {
                    (fi, si, fj, sj)
                } else {
                    (fj, sj, fi, si)
                };
                if seen_pairs.contains_key(&key) {
                    continue;
                }
                seen_pairs.insert(key, true);

                let seq_i = &all_sequences[fi].1.as_slice()[si..si + min_tokens];
                let seq_j = &all_sequences[fj].1.as_slice()[sj..sj + min_tokens];
                if !sequences_match(seq_i, seq_j) {
                    continue;
                }

                let max_extend = extend_match(
                    &all_sequences[fi].1,
                    si,
                    &all_sequences[fj].1,
                    sj,
                    min_tokens,
                );

                let instance_i = CloneInstance {
                    file: all_sequences[fi].0.clone(),
                    start_line: all_sequences[fi].1[si].line,
                    end_line: all_sequences[fi].1[si + max_extend - 1].line,
                };
                let instance_j = CloneInstance {
                    file: all_sequences[fj].0.clone(),
                    start_line: all_sequences[fj].1[sj].line,
                    end_line: all_sequences[fj].1[sj + max_extend - 1].line,
                };

                clone_groups.push(CloneGroup {
                    instances: vvec![instance_i, instance_j],
                    token_count: max_extend,
                });
            }
        }
    }

    clone_groups = merge_overlapping(clone_groups);

    let total_duplicated_lines: usize = clone_groups
        .iter()
        .flat_map(|g| &g.instances)
        .map(|inst| inst.end_line.saturating_sub(inst.start_line) + 1)
        .sum();

    Ok(DuplicateResult {
        clones: clone_groups,
        total_duplicated_lines,
    })
}

fn normalize_kind(kind: &TokenKind) -> TokenKind {
    match kind {
        TokenKind::Identifier => TokenKind::Identifier,
        TokenKind::StringLiteral => TokenKind::StringLiteral,
        TokenKind::NumericLiteral => TokenKind::NumericLiteral,
        other => other.clone(),
    }
}

fn compute_hash(tokens: &[NormalizedToken]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for tok in tokens {
        let kind_byte = kind_to_byte(&tok.kind);
        hash ^= kind_byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

fn kind_to_byte(kind: &TokenKind) -> u8 {
    match kind {
        TokenKind::Identifier => 1,
        TokenKind::StringLiteral => 2,
        TokenKind::NumericLiteral => 3,
        TokenKind::OpenParen => 4,
        TokenKind::CloseParen => 5,
        TokenKind::OpenBrace => 6,
        TokenKind::CloseBrace => 7,
        TokenKind::OpenBracket => 8,
        TokenKind::CloseBracket => 9,
        TokenKind::Semicolon => 10,
        TokenKind::Comma => 11,
        TokenKind::Dot => 12,
        TokenKind::Colon => 13,
        TokenKind::Arrow => 14,
        TokenKind::Operator => 15,
        TokenKind::Assignment => 16,
        TokenKind::Spread => 17,
        TokenKind::QuestionMark => 18,
        TokenKind::TemplateLiteral => 19,
        TokenKind::TemplateHead => 20,
        TokenKind::TemplateMiddle => 21,
        TokenKind::TemplateTail => 22,
        TokenKind::RegexLiteral => 23,
        _ => 0,
    }
}

fn overlaps(start_a: usize, start_b: usize, len: usize) -> bool {
    let end_a = start_a + len;
    let end_b = start_b + len;
    start_a < end_b && start_b < end_a
}

fn sequences_match(a: &[NormalizedToken], b: &[NormalizedToken]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(ta, tb)| kind_to_byte(&ta.kind) == kind_to_byte(&tb.kind))
}

fn extend_match(
    seq_a: &[NormalizedToken],
    start_a: usize,
    seq_b: &[NormalizedToken],
    start_b: usize,
    min_len: usize,
) -> usize {
    let max_possible = (seq_a.len() - start_a).min(seq_b.len() - start_b);
    let mut len = min_len;
    while len < max_possible {
        if kind_to_byte(&seq_a[start_a + len].kind) != kind_to_byte(&seq_b[start_b + len].kind) {
            break;
        }
        len += 1;
    }
    len
}

fn merge_overlapping(mut groups: Vec<CloneGroup>) -> Vec<CloneGroup> {
    if groups.len() <= 1 {
        return groups;
    }

    groups.sort_by(|a, b| {
        let ai = &a.instances[0];
        let bi = &b.instances[0];
        ai.file
            .cmp(&bi.file)
            .then(ai.start_line.cmp(&bi.start_line))
    });

    let mut merged: Vec<CloneGroup> = Vec::new();

    for group in groups {
        let dominated = merged.iter().any(|existing| {
            existing.instances.iter().zip(group.instances.iter()).all(
                |(existing_inst, new_inst)| {
                    existing_inst.file == new_inst.file
                        && existing_inst.start_line <= new_inst.start_line
                        && existing_inst.end_line >= new_inst.end_line
                },
            )
        });
        if !dominated {
            merged.push(group);
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_error_display() {
        let err = DuplicateError::NoSourceFiles(crate::vstr!("no files"));
        assert_eq!(crate::vformat!("{err}"), "no files");
    }

    #[test]
    fn hash_deterministic() {
        let tokens = vvec![
            NormalizedToken {
                kind: TokenKind::Identifier,
                line: 1,
            },
            NormalizedToken {
                kind: TokenKind::OpenParen,
                line: 1,
            },
        ];
        let h1 = compute_hash(&tokens);
        let h2 = compute_hash(&tokens);
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_sequences_different_hash() {
        let a = vvec![NormalizedToken {
            kind: TokenKind::Identifier,
            line: 1,
        }];
        let b = vvec![NormalizedToken {
            kind: TokenKind::OpenParen,
            line: 1,
        }];
        assert_ne!(compute_hash(&a), compute_hash(&b));
    }

    #[test]
    fn overlaps_true() {
        assert!(overlaps(0, 5, 10));
    }

    #[test]
    fn overlaps_false() {
        assert!(!overlaps(0, 10, 5));
    }

    #[test]
    fn detect_no_source_files() {
        let dir = crate::core::volkiwithstds::env::temp_dir().join(&crate::vformat!(
            "volki_dup_empty_{}",
            crate::core::volkiwithstds::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = detect(&dir, 50);
        assert!(matches!(result, Err(DuplicateError::NoSourceFiles(_))));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_duplicate_blocks() {
        let dir = crate::core::volkiwithstds::env::temp_dir().join(&crate::vformat!(
            "volki_dup_blocks_{}",
            crate::core::volkiwithstds::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Two files with identical function bodies
        let code = "function process(data) { const result = data.map(item => item.value).filter(v => v > 0).reduce((a, b) => a + b, 0); return result; }";
        fs::write(dir.join("a.ts"), code).unwrap();
        fs::write(dir.join("b.ts"), code).unwrap();

        let result = detect(&dir, 10).unwrap();
        assert!(!result.clones.is_empty());
        assert!(result.total_duplicated_lines > 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_duplicates_different_code() {
        let dir = crate::core::volkiwithstds::env::temp_dir().join(&crate::vformat!(
            "volki_dup_diff_{}",
            crate::core::volkiwithstds::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("a.ts"), "const x = 1;").unwrap();
        fs::write(dir.join("b.ts"), "function hello() { return true; }").unwrap();

        let result = detect(&dir, 10).unwrap();
        assert!(result.clones.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }
}
