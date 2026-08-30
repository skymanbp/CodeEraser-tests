//! The homepage terminal block IS this repository, measured.
//!
//! It was hand-typed. The caption said "Real output" over
//! `check score 956/1000 | axes 0:45 …` while the repository it names
//! had moved to 953 with five different axes, a different candidate
//! count and a different note line — the page stated a measurement it
//! was no longer making. A version-varying documentation fact belongs
//! on the derived channel (`docs-generated-facts`), the one the bench
//! strip and the demo table already ride.
//!
//! The run is the real binary against this repository into a SCRATCH
//! index (baseline_bridge's shape — the project's own `.ce/index.db`
//! is never touched), so what the block shows is what a reader gets by
//! typing the command. Both languages come from one warmed index: the
//! console answers `--lang`, the measurement is the same. The prompt
//! line shows the invocation a reader would type: `--core` and `--db`
//! are this test's plumbing (a shipped `ce` finds its core beside
//! itself and its index under the project), and the four lines below
//! it are byte-identical to what the run printed.

use crate::common::{self, core_bin, tmp};

/// The four console lines of `ce check --roast` over this repository.
/// `db` is shared by the two language runs so only the first pays for
/// a cold index.
fn roast(db: &str, zh: bool) -> Vec<String> {
    let root = common::repo_root();
    let core = core_bin();
    let mut args = vec!["check", ".", "--core", &core, "--db", db];
    if zh {
        args.extend(["--lang", "zh"]);
    }
    args.push("--roast");
    let (code, out, err) = common::ce_triple(&root, &args, &[]);
    assert_eq!(
        code,
        Some(0),
        "the repository passes its own gate:\n{out}{err}"
    );
    out.replace("\r\n", "\n")
        .lines()
        .map(str::to_string)
        .collect()
}

/// `<`, `>` and `&` are the three that matter inside `<pre>`; the
/// console's own `->` is the only one that occurs.
fn escaped(line: &str) -> String {
    line.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Wrap the part of `line` after `sep` (inclusive of neither) in a
/// span of `class`, leaving the head alone; the whole line when `sep`
/// is empty. One rule for three shapes: the score up to its first
/// column separator, the ratchet verdict after its arrow, the roast.
fn span(line: &str, sep: &str, class: &str) -> String {
    let e = escaped(line);
    if sep.is_empty() {
        return format!("<span class=\"{class}\">{e}</span>");
    }
    match e.split_once(sep) {
        Some((head, tail)) => format!("{head}{sep}<span class=\"{class}\">{tail}</span>"),
        None => e,
    }
}

/// The score's own head — everything before the axes column.
fn scored(line: &str) -> String {
    let e = escaped(line);
    match e.split_once(" |") {
        Some((head, tail)) => format!("<span class=\"s\">{head}</span> |{tail}"),
        None => e,
    }
}

/// The terminal block the page embeds. The verdict class follows the
/// verdict: a repository that stopped passing would say so in red
/// rather than in green, and the assert above would have failed first.
fn render(lines: &[String], zh: bool) -> String {
    assert_eq!(lines.len(), 4, "score, ratchet, note, roast: {lines:?}");
    let verdict = if lines[1].ends_with("pass") { "s" } else { "r" };
    format!(
        "<pre><span class=\"p\">$</span> ce check{} --roast\n{}\n{}\n{}\n{}</pre>\n",
        if zh { " --lang zh" } else { "" },
        scored(&lines[0]),
        span(&lines[1], "-&gt; ", verdict),
        escaped(&lines[2]),
        span(&lines[3], "", "r"),
    )
}

/// Both pages describe ONE state of this repository, so both
/// measurements are taken before either block is written. Blessing the
/// English page changes its line count, and a Chinese run taken after
/// it measures a tree the English block never saw: the first draft did
/// exactly that and the two blocks disagreed about the same repository
/// — one said `0 tolerance drawn`, the other `1` — which is the shape
/// of dishonesty this gate exists to remove. Once the markers are in
/// place a bless moves digits only, so the pair is a fixed point.
#[test]
fn the_homepage_terminal_block_is_this_repository_measured() {
    let db = tmp("site-roast").join("index.db");
    let db = db.display().to_string();
    let pages = [("site/index.html", false), ("site/zh/index.html", true)];
    let blocks: Vec<String> = pages
        .iter()
        .map(|&(_, zh)| render(&roast(&db, zh), zh))
        .collect();
    for ((page, _), block) in pages.iter().zip(&blocks) {
        crate::facts::block::assert_current(page, "roast", block);
    }
}
