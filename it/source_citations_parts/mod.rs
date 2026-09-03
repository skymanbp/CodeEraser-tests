//! The scanner behind `source_citations`: what counts as a citation in
//! Rust or Haskell source, which trees are walked, and how a target
//! resolves to one tracked file. The gate and the scanner's own shape
//! tests live in source_citations.rs.
//!
//! Two target shapes, harvested differently. A Markdown page is cited
//! anywhere on a line, panic strings included. A source file (`.rs`,
//! `.hs`) is cited from COMMENT lines only: the label-parsing tests
//! spell that shape as data (a file name, a colon and a span inside a
//! table row is an input, not a claim). The first draft of the gate
//! held only the Markdown shape, and three source-to-source citations
//! drifted through five releases while it stayed green — one of them
//! a quotation whose words had since been rewritten.
//!
//! The prose here spells its numbers out: this file is walked by the
//! gate it serves, so a citation written in the real form would be
//! harvested as a real one — of a file that does not exist.

use crate::common;

/// One citation: where it was written, what it points at, and the
/// anchor its lines must contain.
pub(crate) struct Cite {
    pub(crate) site: String,
    pub(crate) target: String,
    pub(crate) from: usize,
    pub(crate) to: usize,
    pub(crate) anchor: String,
}

/// The target shapes, and whether a shape counts only on a comment
/// line (see the module note for why source targets do).
const TARGETS: [(&str, bool); 3] = [(".md:", false), (".rs:", true), (".hs:", true)];

/// Files cited from source that live outside this tree — a
/// third-party crate's path, named with its version in the same
/// sentence. Nothing here can read them, so they pass by name.
const OUTSIDE_THE_TREE: &[&str] = &["src/os/windows/named_pipe/local_socket/stream.rs"];

/// What a path may be spelled with. The scan walks LEFT from the
/// extension over these, so the stop character is whatever precedes
/// the path — often `(`, but an em dash in the same sentence would
/// do, which is why the walk is over chars and never over bytes.
fn path_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "._/-".contains(c)
}

/// The number starting at `i`, and the index just past it.
fn number(s: &[u8], i: usize) -> (usize, usize) {
    let end = (i..s.len())
        .find(|&j| !s[j].is_ascii_digit())
        .unwrap_or(s.len());
    (
        std::str::from_utf8(&s[i..end])
            .unwrap_or("")
            .parse()
            .unwrap_or(0),
        end,
    )
}

/// The backtick-delimited anchor that must follow the line span, or
/// `None` when the citation was written without one.
fn anchor_at(line: &str, from: usize) -> Option<String> {
    let rest = line.get(from..)?.trim_start();
    let rest = rest.strip_prefix('`')?;
    rest.find('`').map(|e| rest[..e].to_string())
}

/// A Rust (`//`, doc forms included) or Haskell (`--`) comment line.
fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("--")
}

/// Every citation of one target shape on one line: a path, the
/// extension, a span, then a backticked anchor. The extension plus
/// digits is what makes a citation: a bare page name in prose has no
/// colon-number after it and is not one.
fn cites_of(path: &str, no: usize, line: &str, ext: &str) -> Vec<Result<Cite, String>> {
    let (b, mut out, mut at) = (line.as_bytes(), Vec::new(), 0);
    while let Some(hit) = line[at..].find(ext) {
        let dot = at + hit;
        at = dot + ext.len();
        let (from, mut end) = number(b, at);
        if from == 0 {
            continue;
        }
        let start = line[..dot]
            .char_indices()
            .rev()
            .find(|(_, c)| !path_char(*c))
            .map_or(0, |(i, c)| i + c.len_utf8());
        let target = line[start..dot + ext.len() - 1].to_string();
        let mut to = from;
        if b.get(end) == Some(&b'-') {
            (to, end) = number(b, end + 1);
        }
        at = end;
        if OUTSIDE_THE_TREE.contains(&target.as_str()) {
            continue;
        }
        let site = format!("{path}:{no}");
        out.push(match anchor_at(line, end) {
            Some(anchor) => Ok(Cite {
                site,
                target,
                from,
                to,
                anchor,
            }),
            None => Err(format!(
                "{site} cites {target}:{from}-{to} with no anchor — put the words \
                 those lines must contain in backticks after the span, or \
                 the citation cannot be told apart from a wrong one"
            )),
        });
    }
    out
}

/// Every citation on one line, over every target shape it may carry.
pub(crate) fn cites_on(path: &str, no: usize, line: &str) -> Vec<Result<Cite, String>> {
    TARGETS
        .iter()
        .filter(|(_, comment_only)| !comment_only || is_comment(line))
        .flat_map(|(ext, _)| cites_of(path, no, line, ext))
        .collect()
}

/// Every citation written in Rust or Haskell source, every tree that
/// carries either.
pub(crate) fn harvest(root: &std::path::Path) -> (Vec<Cite>, Vec<String>) {
    let mut files = Vec::new();
    for (dir, ext) in [
        ("cli/src", "rs"),
        ("cli/tests", "rs"),
        ("gui/src-tauri/src", "rs"),
        ("core/app", "hs"),
        ("core/test", "hs"),
    ] {
        common::files_with_ext(&root.join(dir), ext, &mut files);
    }
    assert!(files.len() > 100, "walked {} source files", files.len());
    let (mut ok, mut bad) = (Vec::new(), Vec::new());
    for f in &files {
        let rel = f.strip_prefix(root).unwrap_or(f).display().to_string();
        let rel = rel.replace('\\', "/");
        for (i, line) in std::fs::read_to_string(f)
            .expect("read")
            .lines()
            .enumerate()
        {
            for c in cites_on(&rel, i + 1, line) {
                match c {
                    Ok(c) => ok.push(c),
                    Err(e) => bad.push(e),
                }
            }
        }
    }
    (ok, bad)
}

/// The tracked file a citation names, by unique path suffix — the
/// four PERF-BUDGET sites write it bare, one writes `docs/BENCH.md`,
/// and the calibration one lives under contracts/. A source target
/// spells as much of its path as uniqueness needs: unit tests mirror
/// `cli/src` under `cli/tests/unit`, so a bare file name may be two.
pub(crate) fn resolve(root: &std::path::Path, target: &str) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .args(["ls-files"])
        .current_dir(root)
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files failed");
    let text = String::from_utf8_lossy(&out.stdout);
    let hits: Vec<&str> = text
        .lines()
        .filter(|p| *p == target || p.ends_with(&format!("/{target}")))
        .collect();
    match hits.len() {
        1 => Ok(hits[0].to_string()),
        n => Err(format!("{target} names {n} tracked files, want exactly 1")),
    }
}
