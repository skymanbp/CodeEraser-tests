//! A `file:line` citation in Rust source has to resolve, same as one
//! in a Markdown page.
//!
//! `docs_citations` covers the documentation surfaces and keys every
//! entry on its anchor TEXT, rendering the line number, precisely
//! because line numbers drift under every edit above them. Source
//! comments were outside that gate: four sites pointed at PERF-BUDGET
//! lines 60 to 62 for the release-only rule, which had moved to 82-84
//! — the cited lines were a graph-cache table header by then, and
//! nothing anywhere could say so. They shipped that way in five
//! releases.
//!
//! So the same discipline, executed here: a citation names the lines
//! AND quotes an anchor those lines must still contain. The anchor is
//! mandatory — a span with none is refused, because there is no way
//! to tell a right one from a wrong one. It is delimited by
//! backticks, the one pair that reads naturally in a doc comment, in
//! a `panic!` string and in the Markdown these pages are written in.
//!
//! Note the prose above spells the numbers out. This file is walked
//! by its own gate, so a citation written here in the real form would
//! be harvested as a real citation — of a page that does not exist.

use crate::common;

/// One citation: where it was written, what it points at, and the
/// anchor its lines must contain.
struct Cite {
    site: String,
    target: String,
    from: usize,
    to: usize,
    anchor: String,
}

/// What a page path may be spelled with. The scan walks LEFT from
/// `.md:` over these, so the stop character is whatever precedes the
/// path — often `(`, but an em dash in the same sentence would do,
/// which is why the walk is over chars and never over bytes.
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

/// Every ``<page>.md:<a>[-<b>] `<anchor>` `` on one line. The `.md:` +
/// digits shape is the whole population: a bare `CHANGELOG.md` in
/// prose has no colon-number after it and is not a line citation.
fn cites_on(path: &str, no: usize, line: &str) -> Vec<Result<Cite, String>> {
    let (b, mut out, mut at) = (line.as_bytes(), Vec::new(), 0);
    while let Some(hit) = line[at..].find(".md:") {
        let dot = at + hit;
        at = dot + 4;
        let (from, mut end) = number(b, at);
        if from == 0 {
            continue;
        }
        let start = line[..dot]
            .char_indices()
            .rev()
            .find(|(_, c)| !path_char(*c))
            .map_or(0, |(i, c)| i + c.len_utf8());
        let mut to = from;
        if b.get(end) == Some(&b'-') {
            (to, end) = number(b, end + 1);
        }
        let site = format!("{path}:{no}");
        out.push(match anchor_at(line, end) {
            Some(anchor) => Ok(Cite {
                site,
                target: line[start..dot + 3].to_string(),
                from,
                to,
                anchor,
            }),
            None => Err(format!(
                "{site} cites {}:{from}-{to} with no anchor — put the words \
                 those lines must contain in backticks after the span, or \
                 the citation cannot be told apart from a wrong one",
                &line[start..dot + 3]
            )),
        });
        at = end;
    }
    out
}

/// Every citation written in Rust source, both trees.
fn harvest(root: &std::path::Path) -> (Vec<Cite>, Vec<String>) {
    let mut files = Vec::new();
    for dir in ["cli/src", "cli/tests"] {
        common::files_with_ext(&root.join(dir), "rs", &mut files);
    }
    assert!(files.len() > 100, "walked {} rust files", files.len());
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
/// and the calibration one lives under contracts/.
fn resolve(root: &std::path::Path, target: &str) -> Result<String, String> {
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

#[test]
fn every_source_citation_still_points_at_what_it_quotes() {
    let root = common::repo_root();
    let (cites, mut bad) = harvest(&root);
    assert!(
        !cites.is_empty(),
        "no source citations found — scanner broke"
    );
    for c in &cites {
        let path = match resolve(&root, &c.target) {
            Ok(p) => p,
            Err(e) => {
                bad.push(format!("{}: {e}", c.site));
                continue;
            }
        };
        let text = std::fs::read_to_string(root.join(&path)).expect("read target");
        let lines: Vec<&str> = text.lines().collect();
        if c.to < c.from || c.to > lines.len() {
            bad.push(format!(
                "{} cites {path}:{}-{}, which has {} lines",
                c.site,
                c.from,
                c.to,
                lines.len()
            ));
        } else if !lines[c.from - 1..c.to].join("\n").contains(&c.anchor) {
            bad.push(format!(
                "{} cites {path}:{}-{} for {:?}, which those lines no \
                 longer contain",
                c.site, c.from, c.to, c.anchor
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "source citations adrift:\n{}",
        bad.join("\n")
    );
}

/// The scanner on the shapes it exists to separate.
#[test]
fn the_scanner_reads_spans_and_refuses_bare_line_numbers() {
    // assembled, never written out: the gate above walks this file too
    let (dot, tick) = ('.', '\u{60}');
    let one = cites_on(
        "a.rs",
        7,
        &format!("//! rule (docs/BENCH{dot}md:31-40 {tick}joins only when{tick})"),
    );
    let c = one[0].as_ref().expect("a resolvable citation");
    assert_eq!(
        (c.target.as_str(), c.from, c.to, c.anchor.as_str()),
        ("docs/BENCH.md", 31, 40, "joins only when")
    );
    // a single line is a span of one
    let solo = cites_on(
        "a.rs",
        1,
        &format!("// (X{dot}md:107 {tick}post/put/patch{tick})"),
    );
    let c = solo[0].as_ref().expect("citation");
    assert_eq!((c.from, c.to), (107, 107));
    // no anchor: refused by name, not silently accepted
    assert!(cites_on("a.rs", 3, &format!("// see PERF-BUDGET{dot}md:82-84"))[0].is_err());
    // prose naming a page without a line makes no citation at all
    assert!(
        cites_on(
            "a.rs",
            4,
            &format!("// recorded in CHANGELOG{dot}md as usual")
        )
        .is_empty()
    );
}
