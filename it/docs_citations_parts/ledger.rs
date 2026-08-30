//! The ledger half: keys, the resolution of every citation against
//! the ledger (the passes live in `passes`), and the ledger a bless
//! writes.
//!
//! A key is `citing|target|sha256(text)[..16]|nth` — the page, the
//! link's target as written, the window's hash, and how many earlier
//! citations of that page cite the same window of the same target.
//! Position is nowhere in it: insert a paragraph above, nothing
//! re-keys. The page's `#L` is a hint the passes read (present /
//! stale pointer), never the identity.

use super::anchor::Window;
use super::passes::Pass;
use super::{Citation, Ledger, LedgerEntry, label_lines};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

pub fn hash16(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn key(citing: &str, target: &str, text: &str, nth: usize) -> String {
    format!("{citing}|{target}|{}|{nth}", hash16(text))
}

/// Where a citation's fact sits today.
#[derive(Debug)]
pub struct Resolved {
    pub line: usize,
    pub end: Option<usize>,
    pub window: Window,
}

#[derive(Default)]
pub struct Resolution {
    /// Parallel to the citations; None where a hard red stopped one.
    pub resolved: Vec<Option<Resolved>>,
    /// Red under bless too.
    pub hard: Vec<String>,
    /// What a bless fixes: moves, seeds, drops, re-windowings.
    pub soft: Vec<String>,
    pub next: Ledger,
}

pub(super) fn first_line(text: &str) -> &str {
    text.split('\n').next().unwrap_or(text)
}

/// The end the PAGE states, honoured when its label starts where the
/// citation resolves (the human wrote the range against these lines);
/// the label's end first, then the link's own.
pub(super) fn stated_end(c: &Citation, line: usize) -> Option<usize> {
    match label_lines(&c.label) {
        Some((start, end)) if start == line => end.or(c.end),
        _ => None,
    }
}

/// The ledgered range carried to where the text sits now.
pub(super) fn shifted_end(entry: &LedgerEntry, line: usize) -> Option<usize> {
    entry.end.map(|e| line + (e - entry.line))
}

/// The ledger these resolutions spell, `nth` counting same-window
/// citations of one page to one target in page order.
fn rebuild(citations: &[Citation], resolved: &[Option<Resolved>]) -> Ledger {
    let mut seen: BTreeMap<(&str, &str, &str), usize> = BTreeMap::new();
    let mut next = Ledger::new();
    for (c, r) in citations.iter().zip(resolved) {
        let Some(r) = r else { continue };
        let slot = seen
            .entry((&c.citing, &c.target, &r.window.text))
            .or_insert(0);
        let nth = *slot;
        *slot += 1;
        next.insert(
            key(&c.citing, &c.target, &r.window.text, nth),
            LedgerEntry {
                target: c.target.clone(),
                line: r.line,
                end: r.end,
                head: r.window.head,
                text: r.window.text.clone(),
            },
        );
    }
    next
}

pub fn resolve(root: &Path, citations: &[Citation], ledger: &Ledger) -> Resolution {
    let mut pass = Pass::new(root, ledger);
    let mut out = Resolution::default();
    for c in citations {
        pass.resolve_one(c, &mut out);
    }
    pass.orphans(&mut out);
    out.next = rebuild(citations, &out.resolved);
    out
}

#[test]
fn keys_are_text_not_position() {
    let k = key("docs/a.md", "../x.rs", "fn one()", 0);
    assert_eq!(k, format!("docs/a.md|../x.rs|{}|0", hash16("fn one()")));
    assert_eq!(hash16("fn one()").len(), 16);
    assert_ne!(hash16("fn one()"), hash16("fn one() "));
}
