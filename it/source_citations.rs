//! A `file:line` citation in Rust or Haskell source has to resolve,
//! same as one in a Markdown page.
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
//! What counts as a citation, and where the trees are walked, is the
//! scanner in source_citations_parts.
//!
//! Note the prose above spells the numbers out. This file is walked
//! by its own gate, so a citation written here in the real form would
//! be harvested as a real citation — of a page that does not exist.

use crate::common;
use crate::source_citations_parts::{cites_on, harvest, resolve};

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

/// What one spelling must scan to: a citation, a refusal, or nothing.
enum Want {
    C(&'static str, usize, usize, &'static str),
    Refused,
    Nothing,
}
use Want::{C, Nothing, Refused};

/// The shapes the scanner exists to separate, spelled with `·` for the
/// dot and `ˋ` for the backtick: this file is walked by the gate above,
/// and a real spelling here would be a real citation of a page that
/// does not exist.
const SHAPES: &[(&str, Want)] = &[
    // a page cited from a doc comment: span and anchor
    (
        "//! (docs/BENCH·md:31-40 ˋjoins onlyˋ)",
        C("docs/BENCH.md", 31, 40, "joins only"),
    ),
    // a single line is a span of one
    ("// (X·md:107 ˋpost/putˋ)", C("X.md", 107, 107, "post/put")),
    // no anchor: refused by name, not silently accepted
    ("// see PERF-BUDGET·md:82-84", Refused),
    // prose naming a page without a line makes no citation at all
    ("// recorded in CHANGELOG·md as usual", Nothing),
    // a source file cited from a comment, Rust or Haskell
    (
        "//! set (ladder/md·rs:86 ˋscope.filesˋ), so",
        C("ladder/md.rs", 86, 86, "scope.files"),
    ),
    (
        "-- (flags·rs:9 ˋsymbol factˋ)",
        C("flags.rs", 9, 9, "symbol fact"),
    ),
    // the same spelling as data in a code line is not a claim
    ("    (\"Cost·hs:42-55\", 44, Some(57)),", Nothing),
    // a comment citing a source file without an anchor is refused
    ("// see conn·rs:35", Refused),
    // a third-party file passes by name instead of failing for its anchor
    (
        "//!   src/os/windows/named_pipe/local_socket/stream·rs:53), and",
        Nothing,
    ),
];

#[test]
fn the_scanner_separates_the_shapes_it_exists_for() {
    for (spelt, want) in SHAPES {
        let line = spelt.replace('·', ".").replace('ˋ', "`");
        let got = cites_on("a.rs", 1, &line);
        match want {
            C(target, from, to, anchor) => {
                let c = got
                    .first()
                    .and_then(|r| r.as_ref().ok())
                    .unwrap_or_else(|| panic!("{line}: no citation"));
                assert_eq!(
                    (c.target.as_str(), c.from, c.to, c.anchor.as_str()),
                    (*target, *from, *to, *anchor),
                    "{line}"
                );
            }
            Refused => assert!(matches!(got.as_slice(), [Err(_)]), "{line}"),
            Nothing => assert!(got.is_empty(), "{line}"),
        }
    }
}
