//! Every frozen point in `contracts/bench/bench.json` cites where its
//! number was established — `docs/X.md:A-B + contracts/eval/*.json` —
//! and until v2.25 nothing read those citations: `docs_citations`
//! harvests only `[label](path#L)` links and `source_citations` only
//! Rust comments, so a frozen row could point at lines that had moved
//! under it and ship green on BENCH.md, both READMEs and two site
//! pages. This gate resolves every segment (the file exists, the
//! range fits, a glob matches at least one file) and demands that
//! every number the row shows is spelled verbatim inside the cited
//! lines — the anchor a reader would go and check.

use crate::common::repo_root;
use serde_json::{Value, json};
use std::path::Path;

/// One cited place: a path, and the 1-based inclusive line range when
/// the segment carries one (a bare path or glob carries none).
type Segment = (String, Option<(usize, usize)>);

/// `<path>:<a>-<b>` segments joined with ` + `, plus bare paths or
/// one-star globs; a segment that starts with `:` is another range in
/// the previous segment's file. (The literals below are built at run
/// time on purpose: `source_citations` reads every `.rs` for
/// `file:line` spans and would take a fixture for a citation.)
fn segments(source: &str) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    for seg in source.split(" + ").map(str::trim) {
        if let Some(range) = seg.strip_prefix(':') {
            let path = out
                .last()
                .expect("a `:A-B` continuation follows a path")
                .0
                .clone();
            out.push((path, Some(parse_range(range))));
            continue;
        }
        match seg.rsplit_once(':') {
            Some((path, range)) if range.starts_with(|c: char| c.is_ascii_digit()) => {
                out.push((path.to_string(), Some(parse_range(range))));
            }
            _ => out.push((seg.to_string(), None)),
        }
    }
    out
}

fn parse_range(s: &str) -> (usize, usize) {
    let (a, b) = s.split_once('-').unwrap_or((s, s));
    let line = |x: &str| {
        x.parse::<usize>()
            .unwrap_or_else(|_| panic!("line number: {x:?}"))
    };
    (line(a), line(b))
}

/// Maximal runs of digits with the joiners a number wears on these
/// pages — `17/17`, `1.000`, `100%`, `0.90` — so a ratio is one token
/// and `0/600` cannot be satisfied by a lone `0`.
fn number_tokens(value: &str) -> Vec<&str> {
    value
        .split(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '/' | '%')))
        .filter(|t| t.chars().any(|c| c.is_ascii_digit()))
        .collect()
}

/// Files a bare segment names: an exact path, or a one-star glob in
/// the file name (`contracts/eval/t3-precision-*-v1.json`).
fn files_named(root: &Path, pattern: &str) -> usize {
    let (dir, name) = pattern.rsplit_once('/').unwrap_or(("", pattern));
    let Some((prefix, suffix)) = name.split_once('*') else {
        return usize::from(root.join(pattern).is_file());
    };
    let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with(prefix) && n.ends_with(suffix) && n.len() >= prefix.len() + suffix.len()
        })
        .count()
}

/// The verdict for one frozen row: every segment resolves and every
/// shown number is inside the cited lines, or the first failure named.
fn check_row(root: &Path, row: &Value) -> Result<(), String> {
    let field = |k: &str| {
        row[k]
            .as_str()
            .ok_or_else(|| format!("frozen row lacks a string `{k}`: {row}"))
    };
    let (metric, source, value) = (field("metric")?, field("source")?, field("value")?);
    let mut cited = String::new();
    let mut ranges = 0;
    for (path, range) in segments(source) {
        let Some((a, b)) = range else {
            if files_named(root, &path) == 0 {
                return Err(format!("{metric}: `{path}` names no file"));
            }
            continue;
        };
        let text = std::fs::read_to_string(root.join(&path))
            .map_err(|e| format!("{metric}: {path}: {e}"))?;
        let n = text.lines().count();
        if !(1 <= a && a <= b && b <= n) {
            return Err(format!(
                "{metric}: {path}:{a}-{b} does not fit a {n}-line file"
            ));
        }
        for line in text.lines().skip(a - 1).take(b - a + 1) {
            cited.push_str(line);
            cited.push('\n');
        }
        ranges += 1;
    }
    if ranges == 0 {
        return Err(format!("{metric}: `{source}` cites no line range"));
    }
    match number_tokens(value)
        .into_iter()
        .find(|t| !cited.contains(t))
    {
        Some(t) => Err(format!(
            "{metric}: value `{value}` shows {t} but `{source}` does not spell it"
        )),
        None => Ok(()),
    }
}

#[test]
fn every_frozen_point_cites_lines_that_spell_its_numbers() {
    let root = repo_root();
    let text =
        std::fs::read_to_string(root.join("contracts/bench/bench.json")).expect("bench.json");
    let doc: Value = serde_json::from_str(&text).expect("bench.json parses");
    let frozen = doc["frozen"].as_array().expect("frozen is an array");
    assert!(!frozen.is_empty(), "the frozen table is not empty");
    let failures: Vec<String> = frozen
        .iter()
        .filter_map(|row| check_row(&root, row).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The grammar the gate reads: continuations inherit the file, a glob
/// is a bare segment, and a ratio is one token.
#[test]
fn the_source_grammar_reads_continuations_globs_and_ratios() {
    let segs = segments(&format!(
        "{md}:16-36 + :47-94 + contracts/eval/x-*-v1.json",
        md = "docs/A.md"
    ));
    assert_eq!(
        segs,
        vec![
            ("docs/A.md".to_string(), Some((16, 36))),
            ("docs/A.md".to_string(), Some((47, 94))),
            ("contracts/eval/x-*-v1.json".to_string(), None),
        ]
    );
    assert_eq!(
        number_tokens("cobra 106/109 raw -> 106/106 attributed (gate <= 1%)"),
        vec!["106/109", "106/106", "1%"]
    );
    assert_eq!(
        number_tokens("61 answered / 0 wrong (1.000)"),
        vec!["61", "0", "1.000"]
    );
}

/// Negative probes, each refused by name: a number the cited lines do
/// not spell, a range past the file's end, and a glob that matches
/// nothing. A gate that only ever saw the green table is not a gate.
#[test]
fn a_row_whose_citation_does_not_carry_its_number_is_refused_by_name() {
    let root = repo_root();
    let row =
        |source: &str, value: &str| json!({"metric": "probe", "source": source, "value": value});
    // fixtures, not citations — see the note on `segments`
    let eval_set = "docs/EVAL-SET.md";
    let real = format!("{eval_set}:131-140");
    let real = real.as_str();
    assert_eq!(
        check_row(&root, &row(real, "0/600 flagged (gate <= 1%)")),
        Ok(())
    );
    let wrong = check_row(&root, &row(real, "1/600 flagged")).unwrap_err();
    assert!(
        wrong.contains("shows 1/600") && wrong.contains(real),
        "{wrong}"
    );
    let past = check_row(&root, &row(&format!("{eval_set}:131-99999"), "0/600")).unwrap_err();
    assert!(past.contains("does not fit"), "{past}");
    let none = check_row(
        &root,
        &row(&format!("{real} + contracts/eval/nope-*-v1.json"), "0/600"),
    )
    .unwrap_err();
    assert!(none.contains("names no file"), "{none}");
    let bare = check_row(&root, &row("contracts/eval/fpr-fourclass-v1.json", "0/600")).unwrap_err();
    assert!(bare.contains("cites no line range"), "{bare}");
}
