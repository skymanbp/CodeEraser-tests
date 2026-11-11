//! The measured features of one dead file (brief L3 (a)–(e)), composed
//! from the product's own primitives where it has one and from git
//! where it does not:
//! (a) the distinct named top-level node kinds of the file — the
//!     tree-sitter root's named children through the product's parser
//!     (`scan::ast::parse_lang`); none for a language without a grammar;
//! (b) kept out-edges — the graph wire's edges from the file's node whose
//!     kind is not inert (asset 3, unused reference definition 5: the
//!     two kinds the core excludes from liveness);
//! (c) how many OTHER files of the pinned tree spell the file's
//!     basename (`git grep -F -l`), and its stem as a word
//!     (`git grep -F -w -l`) — tracked files at the tip, the file itself
//!     not counted (the mention universe's base set, without its Terms);
//! (d) the convention segments of its path (mod.rs CONVENTION_SEGMENTS,
//!     `resources` only below a test segment);
//! (e) its document class: markdown / html / text, else null.

use super::CONVENTION_SEGMENTS;
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

/// (a): distinct named kinds of the root's children, sorted.
pub fn top_kinds(text: &str, lang: Option<Lang>) -> Vec<String> {
    let Some(tree) = lang.and_then(|l| codeeraser::scan::ast::parse_lang(text, l)) else {
        return Vec::new();
    };
    let root = tree.root_node();
    let kinds: BTreeSet<String> = (0..root.named_child_count())
        .filter_map(|i| root.named_child(i as u32))
        .map(|n| n.kind().to_string())
        .collect();
    kinds.into_iter().collect()
}

/// (c): the files of the tree at `tip` other than `path` that spell
/// `needle` (`word` = as a whole word).
pub fn spelled_by(repo: &str, tip: &str, path: &str, needle: &str, word: bool) -> u64 {
    if needle.is_empty() {
        return 0;
    }
    let mut args = vec!["-C", repo, "grep", "-I", "-l", "-z", "-F"];
    if word {
        args.push("-w");
    }
    args.extend(["-e", needle, tip]);
    let out = std::process::Command::new("git")
        .args(&args)
        .output()
        .expect("git grep");
    // 0 = matches, 1 = none; anything else is a broken instrument
    assert!(
        matches!(out.status.code(), Some(0 | 1)),
        "git grep {needle}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let own = format!("{tip}:{path}");
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|p| !p.is_empty() && *p != own)
        .count() as u64
}

/// (d): the convention segments of the path, in path order.
pub fn conventions(path: &str) -> Vec<String> {
    let segs: Vec<String> = path.split('/').map(str::to_ascii_lowercase).collect();
    let dirs = &segs[..segs.len().saturating_sub(1)];
    let mut out = Vec::new();
    for (i, s) in dirs.iter().enumerate() {
        let under_test = dirs[..i].iter().any(|d| d.starts_with("test"));
        if CONVENTION_SEGMENTS.contains(&s.as_str()) || (s == "resources" && under_test) {
            out.push(s.clone());
        }
    }
    out
}

/// (e): the document class by the product's language of the path.
pub fn doc_class(path: &str) -> Value {
    match Lang::from_path(Path::new(path)) {
        Some(Lang::Markdown) => json!("markdown"),
        Some(Lang::Html) => json!("html"),
        Some(Lang::Text) => json!("text"),
        _ => Value::Null,
    }
}

/// The basename and the stem (the basename less its last extension).
pub fn names(path: &str) -> (&str, &str) {
    let base = path.rsplit('/').next().unwrap_or(path);
    let stem = base.rsplit_once('.').map_or(base, |(s, _)| s);
    (base, stem)
}
