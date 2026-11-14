//! A duplicated documentation section a link's fragment lands on is
//! referenced (release 1.10.0, erase's `linked_section`): of two
//! byte-identical segments the side with FEWER links to its section is
//! the candidate, equal counts keep the older choice (the later by
//! path and line), and a candidate whose section is linked is refused
//! by name and erased by no `--apply`. And the plan's provenance prints
//! in the console's language, written by the core from the row's facts
//! (J2): the English console line carries the document's `provenance`
//! byte for byte, the Chinese one no English clause.

use crate::common::{self, core_bin, tmp};
use crate::erase_e2e::PARA;
use codeeraser::erase;
use std::path::{Path, PathBuf};

/// A tree whose `README.md` writes `links`, with `docs/a.md` and
/// `docs/b.md` each holding PARA under the heading `a_head` / `b_head`.
fn tree(name: &str, links: &str, a_head: &str, b_head: &str) -> PathBuf {
    let dir = tmp(name);
    let page = |head: &str| format!("# Page\n\n{head}\n\n{PARA}\n");
    common::write_all(
        &dir,
        &[
            ("README.md", format!("# P\n\n{links}\n")),
            ("docs/a.md", page(a_head)),
            ("docs/b.md", page(b_head)),
        ],
    );
    common::init_and_commit(&dir, "fixture");
    dir
}

/// The plan's one verbatim_doc row: (path, eraseable, reason, linked).
fn doc_row(dir: &Path) -> (String, bool, &'static str, i64) {
    let plan = erase::plan(dir, None, &core_bin()).expect("plan");
    let rows: Vec<_> = plan
        .rows
        .iter()
        .filter(|r| r.class == "verbatim_doc")
        .collect();
    assert_eq!(rows.len(), 1, "one doc pair");
    let r = rows[0];
    let linked = match &r.source {
        erase::Source::Doc { linked, .. } => *linked,
        _ => panic!("a doc row carries doc facts"),
    };
    (r.path.clone(), r.eraseable, r.reason, linked)
}

/// Apply the plan; then every file's bytes and the converged re-plan.
fn applied(dir: &Path) -> (usize, String, String) {
    let core = core_bin();
    let plan = erase::plan(dir, None, &core).expect("plan");
    let n = erase::apply_plan(dir, None, &core, &plan).expect("apply");
    let read = |p: &str| std::fs::read_to_string(dir.join(p)).expect(p);
    let after = erase::plan(dir, None, &core).expect("re-plan");
    assert_eq!(after.counts.eraseable, 0, "converged");
    (n, read("docs/a.md"), read("docs/b.md"))
}

/// Both sections linked once: equal counts, so the later side
/// (docs/b.md) is the candidate as before — and its section is linked,
/// so the core refuses it as `linked_section`; `--apply` erases nothing.
#[test]
fn a_linked_candidate_is_refused_and_never_erased() {
    let links = "[a](docs/a.md#install) [b](docs/b.md#install)";
    let dir = tree("erase-linked-both", links, "## Install", "## Install");
    assert_eq!(
        doc_row(&dir),
        ("docs/b.md".into(), false, "linked_section", 1)
    );
    let (n, a, b) = applied(&dir);
    assert_eq!(n, 0, "nothing eraseable");
    assert!(a.contains(PARA) && b.contains(PARA), "both copies stay");
}

/// The tree's candidate `cand` is the eraseable side with no linked
/// section, `--apply` erases its copy and leaves the other one.
fn the_unlinked_copy_goes(name: &str, links: &str, heads: [&str; 2], cand: &str) {
    let dir = tree(name, links, heads[0], heads[1]);
    assert_eq!(doc_row(&dir), (cand.into(), true, "eraseable", 0));
    let (n, a, b) = applied(&dir);
    let stays = (a.contains(PARA), b.contains(PARA));
    let want = (cand != "docs/a.md", cand != "docs/b.md");
    assert_eq!((n, stays), (1, want), "the unlinked copy goes");
}

/// The auditor's er1 shape: only the LATER side's section is linked
/// (docs/b.md#install; docs/a.md linked without a fragment). The side
/// with fewer links, docs/a.md, is the candidate and is erased;
/// docs/b.md, the linked one, is untouched.
#[test]
fn only_the_later_side_linked_erases_the_earlier() {
    let links = "See [install](docs/b.md#install) and [a](docs/a.md).";
    the_unlinked_copy_goes(
        "erase-linked-later",
        links,
        ["# A", "## Install"],
        "docs/a.md",
    );
}

/// The mirror: only the EARLIER side's section is linked, so the later
/// side is the candidate and is erased, as before.
#[test]
fn only_the_earlier_side_linked_erases_the_later_as_before() {
    let heads = ["## Install", "## Install"];
    the_unlinked_copy_goes(
        "erase-linked-earlier",
        "[a](docs/a.md#install)",
        heads,
        "docs/b.md",
    );
}

/// The console's provenance clause (J2) over a tree with an advisory
/// row of each kind — a dead file, a linked doc twin, a clone twin:
/// in English each advisory line ends with the document's `provenance`
/// in parentheses, byte for byte; in Chinese each carries the Chinese
/// clause of its kind and no two English words side by side after the
/// reason.
#[test]
fn the_provenance_clause_prints_in_the_console_language() {
    let dir = tmp("erase-linked-lang");
    crate::erase_e2e::seed(&dir);
    let links = "[a](docs/a.md#install) [b](docs/b.md#install)";
    let page = format!("# Page\n\n## Install\n\n{PARA} extra words here\n");
    let files = [
        ("links.md", links.to_string()),
        ("docs/a.md", page.clone()),
        ("docs/b.md", page),
    ];
    common::write_all(&dir, &files);
    common::commit_all(&dir, "linked docs");
    let json = common::run_ce_env(&dir, &["erase", "--format", "json", "."], &[]);
    let doc: serde_json::Value = serde_json::from_slice(&json.stdout).expect("erase document");
    let rows = doc["rows"].as_array().expect("rows");
    let en = console(&dir, false);
    for r in rows.iter().filter(|r| r["eraseable"] == false) {
        let tail = format!("({})", r["provenance"].as_str().expect("provenance"));
        assert!(
            en.lines().any(|l| l.ends_with(&tail)),
            "no English line ends {tail}:\n{en}"
        );
    }
    let zh = console(&dir, true);
    for kind in ["死码：", "文档重复：", "克隆："] {
        assert!(zh.contains(kind), "no {kind} clause:\n{zh}");
    }
    for l in zh.lines().filter(|l| l.starts_with("仅建议 ")) {
        let clause = l.split_once('（').map_or("", |(_, c)| c);
        assert!(!prose(clause), "English inside a Chinese line: {l}");
    }
}

fn console(dir: &Path, zh: bool) -> String {
    let lang: &[(&str, &str)] = if zh { &[("CE_LANG", "zh")] } else { &[] };
    let out = common::run_ce_env(dir, &["erase", "."], lang);
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Two lowercase English words of three or more letters side by side.
fn prose(s: &str) -> bool {
    let word = |t: &str| t.len() >= 3 && t.chars().all(|c| c.is_ascii_lowercase());
    let words: Vec<&str> = s
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .collect();
    words.windows(2).any(|w| word(w[0]) && word(w[1]))
}
