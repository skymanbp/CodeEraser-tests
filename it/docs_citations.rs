//! Machine-resolvable file:line citations across the documentation
//! (plan v2.21 ③): this half derives the surfaces, harvests every
//! `[label](path#L<n>[-L<m>])`, refuses what no ledger should hold,
//! and runs the resolution; the ledger, window and rendering halves
//! live in docs_citations_parts.
//!
//! Surfaces are derived, not listed: every tracked Markdown page under
//! docs/, the two READMEs, CHANGELOG, plugin/README and demo/README,
//! minus contracts/docs-citations-optout.json — each opt-out with its
//! reason, and each naming a page the derivation knows.

use crate::common::repo_root;
use crate::docs_citations_parts::ledger::{Resolution, Resolved, resolve};
use crate::docs_citations_parts::{
    Citation, Ledger, json, label_claims, lines, render, target_path,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

const FIXED: [&str; 5] = [
    "README.md",
    "README.zh.md",
    "CHANGELOG.md",
    "plugin/README.md",
    "demo/README.md",
];
const LEDGER: &str = "contracts/docs-citations.json";
const OPTOUT: &str = "contracts/docs-citations-optout.json";

/// A surface: its repo-relative path, its text, its citations.
type Page = (String, String, Vec<Citation>);

fn tracked_docs(root: &Path) -> Vec<String> {
    let out = Command::new("git")
        .args(["ls-files", "--", "docs"])
        .current_dir(root)
        .output()
        .expect("git ls-files");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|p| p.ends_with(".md"))
        .map(String::from)
        .collect()
}

fn surfaces(root: &Path) -> Vec<String> {
    let optout: BTreeMap<String, String> =
        serde_json::from_str(&crate::facts::read(root, OPTOUT)).expect("parse citation opt-outs");
    let mut all = tracked_docs(root);
    all.extend(FIXED.iter().map(|s| s.to_string()));
    all.sort();
    for (page, reason) in &optout {
        assert!(
            all.contains(page),
            "{OPTOUT}: {page} is not a citation surface ({reason})"
        );
        assert!(
            !reason.trim().is_empty(),
            "{OPTOUT}: {page} opts out without a reason"
        );
    }
    all.retain(|p| !optout.contains_key(p));
    all
}

fn parse_anchor(link: &str) -> Option<(&str, usize, Option<usize>)> {
    let hash = link.find("#L")?;
    let target = &link[..hash];
    let numbers = &link[hash + 2..];
    let (start, end) = numbers
        .split_once("-L")
        .map_or((numbers, None), |(a, b)| (a, Some(b)));
    if start.is_empty() || !start.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let line = start.parse().ok()?;
    let end = end.map(str::parse).transpose().ok()?;
    Some((target, line, end))
}

/// The citations of one line, spans absolute in the page.
fn citations_in_line(citing: &str, line_no: usize, at: usize, line: &str, out: &mut Vec<Citation>) {
    let mut from = 0;
    while let Some(open) = line[from..].find("](") {
        let open = from + open;
        let Some(close) = line[open + 2..].find(')') else {
            break;
        };
        let close = open + 2 + close;
        let link = &line[open + 2..close];
        let head = &line[..open];
        if let (Some(b), Some((target, target_line, end))) = (head.rfind('['), parse_anchor(link)) {
            out.push(Citation {
                citing: citing.to_string(),
                citing_line: line_no,
                label: head[b + 1..].to_string(),
                link: link.to_string(),
                target: target.to_string(),
                line: target_line,
                end,
                span: (at + b, at + close + 1),
            });
        }
        from = close + 1;
    }
}

fn harvest(citing: &str, text: &str) -> Vec<Citation> {
    let mut out = Vec::new();
    let mut at = 0;
    for (i, raw) in text.split_inclusive('\n').enumerate() {
        citations_in_line(
            citing,
            i + 1,
            at,
            raw.trim_end_matches(['\n', '\r']),
            &mut out,
        );
        at += raw.len();
    }
    out
}

/// What no harvest admits: a missing target, or a link whose own range
/// runs backwards. A blank or past-EOF pointer is judged where it
/// means something — a fresh seed refuses it, a ledgered citation
/// resolves through it (that is what "moved" looks like).
fn refusals(root: &Path, citations: &[Citation]) -> Vec<String> {
    let mut errors = Vec::new();
    for c in citations {
        let prefix = format!("{}:{}: {}", c.citing, c.citing_line, c.link);
        if !target_path(root, &c.citing, &c.target).is_file() {
            errors.push(format!("{prefix} -> target missing"));
        } else if c.end.is_some_and(|e| e < c.line) {
            errors.push(format!("{prefix} -> range end before start L{}", c.line));
        }
    }
    errors
}

fn saved_ledger(root: &Path) -> Ledger {
    match fs::read_to_string(root.join(LEDGER)) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            panic!("{LEDGER} is not an S6 ledger ({e}); delete it and CE_BLESS=1 to migrate")
        }),
        Err(_) => Ledger::new(),
    }
}

/// A page's path and text with its citations beside their resolutions.
type Paired<'a> = (&'a str, &'a str, Vec<(&'a Citation, &'a Resolved)>);

/// Each page's citations beside their resolutions, in page order —
/// the resolutions run parallel to the flattened citation list, and
/// a hard-stopped citation has none.
fn paired<'a>(pages: &'a [Page], res: &'a Resolution) -> Vec<Paired<'a>> {
    let mut i = 0;
    pages
        .iter()
        .map(|(citing, text, cites)| {
            let pairs = cites
                .iter()
                .filter_map(|c| {
                    let r = res.resolved[i].as_ref().map(|r| (c, r));
                    i += 1;
                    r
                })
                .collect();
            (citing.as_str(), text.as_str(), pairs)
        })
        .collect()
}

/// Every resolved end lies in [start, EOF], and every further number a
/// comma-list label claims (unrendered human claims) lies in the file.
fn range_errors(root: &Path, pairs: &[(&Citation, &Resolved)]) -> Vec<String> {
    let mut errors = Vec::new();
    for (c, r) in pairs {
        let count = lines(&target_path(root, &c.citing, &c.target)).len();
        let where_ = format!("{}:{}", c.citing, c.citing_line);
        if let Some(e) = r.end.filter(|&e| e < r.line || e > count) {
            errors.push(format!(
                "{where_}: range end L{e} outside [L{}, EOF {count}]",
                r.line
            ));
        }
        for n in label_claims(&c.label).into_iter().filter(|&n| n > count) {
            errors.push(format!(
                "{where_}: label claims L{n} past EOF (target has {count} lines)"
            ));
        }
    }
    errors
}

fn settle_ledger(root: &Path, saved: &Ledger, next: &Ledger) -> Vec<String> {
    if saved == next {
        return Vec::new();
    }
    if !crate::facts::blessing() {
        return vec![format!(
            "{LEDGER} drifted ({} -> {} entries) — CE_BLESS=1 to regenerate",
            saved.len(),
            next.len()
        )];
    }
    fs::write(root.join(LEDGER), json(next)).expect("bless citation ledger");
    println!("blessed {} citations at {LEDGER}", next.len());
    Vec::new()
}

#[test]
fn citations_resolve_to_their_text_and_render_their_numbers() {
    let root = repo_root();
    let pages: Vec<Page> = surfaces(&root)
        .into_iter()
        .map(|rel| {
            let text = crate::facts::read(&root, &rel);
            let cites = harvest(&rel, &text);
            (rel, text, cites)
        })
        .collect();
    let citations: Vec<Citation> = pages
        .iter()
        .flat_map(|(_, _, c)| c.iter())
        .cloned()
        .collect();
    // the harvest is real (the i18n gate's own guard): a broken parser
    // must not pass vacuously — the family carries ~1300 anchors
    assert!(
        citations.len() > 500,
        "only {} citations harvested — the parser is broken",
        citations.len()
    );
    let refused = refusals(&root, &citations);
    assert!(
        refused.is_empty(),
        "docs citation refusals:\n{}",
        refused.join("\n")
    );
    let saved = saved_ledger(&root);
    let res = resolve(&root, &citations, &saved);
    let paired = paired(&pages, &res);
    let mut hard = res.hard.clone();
    for (_, _, pairs) in &paired {
        hard.extend(range_errors(&root, pairs));
    }
    // a hard red stops everything BEFORE any page or the ledger is
    // written — a bless must not re-render around a vanished anchor
    assert!(
        hard.is_empty(),
        "docs citation hard errors (red under bless too):\n{}",
        hard.join("\n")
    );
    let mut soft: Vec<String> = paired
        .iter()
        .flat_map(|(citing, text, pairs)| render::settle(&root, citing, text, pairs))
        .collect();
    soft.extend(settle_ledger(&root, &saved, &res.next));
    if crate::facts::blessing() {
        for note in &res.soft {
            println!("{note}");
        }
        return;
    }
    soft.extend(res.soft);
    assert!(
        soft.is_empty(),
        "docs citation drift (CE_BLESS=1 re-renders and re-ledgers):\n{}",
        soft.join("\n")
    );
}
