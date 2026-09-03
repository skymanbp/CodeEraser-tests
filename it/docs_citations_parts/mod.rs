//! The citation gate's shared shapes (plan v2.21 ③, the S6 flip).
//!
//! The anchor TEXT is the fact; every number a citation shows — the
//! `#L` in the link, the `file:a-b` in the label — is a rendering of
//! where that text sits today, rewritten under CE_BLESS=1. Before S6
//! the ledger was keyed by position (`citing:index`) and pinned the
//! text as a check on the number; a moved line was a red to re-aim
//! by hand, and a label's own number was unread until v2.14. Now the
//! ledger is keyed `citing|target|sha256(text)[..16]|nth`, so an
//! insertion elsewhere in the citing page disturbs no key, and the
//! numbers follow the text (`ledger`, `render`; the window rule lives
//! in `anchor`).
//!
//! Direction is deliberate: the leaf owns the shapes and the parent
//! references DOWNWARD, so there is no `use super::` web to become an
//! import cycle by the graph family's own axis-6 measure.

pub mod anchor;
pub mod ledger;
pub mod passes;
pub mod render;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Citation {
    pub citing: String,
    pub citing_line: usize,
    /// The human-readable half — `<file>:<a>-<b>` in
    /// `[<file>:<a>-<b>](…#L<a>)`; a rendering since S6.
    pub label: String,
    pub link: String,
    pub target: String,
    pub line: usize,
    /// The link's own `-L<end>`, when it carries one.
    pub end: Option<usize>,
    /// Byte span of the whole `[label](link)` in the citing file — the
    /// renderer splices exactly there, so the page's EOL survives.
    pub span: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LedgerEntry {
    pub target: String,
    pub line: usize,
    /// The range end a `file:a-b` label states — it exists nowhere
    /// but in the label, so the ledger holds it (migrated once at the
    /// S6 bless) and renders `a-b` from `(line, end)`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<usize>,
    /// Lines of the window ABOVE the cited line (anchor::seed grows
    /// upward only once EOF stops it); omitted when zero.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub head: usize,
    pub text: String,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

pub type Ledger = BTreeMap<String, LedgerEntry>;

/// The line numbers a label states: `file:a`, `file:a-b`, a bare `a`
/// (the range-end form `…–[141](…#L141)`), or the FIRST segment of a
/// comma list `file:a-b,c,d` — the later segments are human claims,
/// bounded by EOF (`label_claims`) and otherwise unread. None for a
/// label naming no line.
pub fn label_lines(label: &str) -> Option<(usize, Option<usize>)> {
    let tail = label.rsplit_once(':').map_or(label, |(_, t)| t);
    let first = tail.split(',').next().unwrap_or(tail).trim();
    let (start, end) = first
        .split_once('-')
        .map_or((first, None), |(a, b)| (a, Some(b)));
    let start = start.parse().ok()?;
    let end = match end {
        Some(e) => Some(e.parse().ok()?),
        None => None,
    };
    Some((start, end))
}

/// The numbers a comma-list label claims beyond its first segment —
/// human claims the renderer never touches, so the EOF bound reads
/// them here.
pub fn label_claims(label: &str) -> Vec<usize> {
    let tail = label.rsplit_once(':').map_or(label, |(_, t)| t);
    let rest = tail.split_once(',').map_or("", |(_, r)| r);
    rest.split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse().ok())
        .collect()
}

pub fn target_path(root: &Path, citing: &str, target: &str) -> PathBuf {
    root.join(citing)
        .parent()
        .expect("citing file parent")
        .join(target)
}

pub fn lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read target {}: {e}", path.display()))
        .lines()
        .map(str::to_string)
        .collect()
}

pub fn json(ledger: &Ledger) -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(ledger).expect("serialize citation ledger")
    )
}

#[test]
fn label_shapes() {
    assert_eq!(label_lines("Cost.hs:42-55"), Some((42, Some(55))));
    assert_eq!(label_lines("dedup/mod.rs:301"), Some((301, None)));
    assert_eq!(label_lines("141"), Some((141, None)));
    assert_eq!(label_lines("Split.hs:2-5,24,202"), Some((2, Some(5))));
    assert_eq!(label_lines("Cost.hs"), None);
    assert_eq!(label_lines("§4.2"), None);
    assert_eq!(label_claims("Split.hs:2-5,24,202"), vec![24, 202]);
    assert_eq!(label_claims("285"), Vec::<usize>::new());
}
