//! `ce deadcode`'s grouping end to end (plan v2.35, the dead-file
//! amendment, phase 5A): real trees through the walk, the ladder and the
//! core, then the document's `root` / `under` / counts and the console's
//! order in both languages — the roots first, each once with the files
//! that fall with it, then the dependents naming their roots. The trees
//! are one literal table: `== name` opens a tree, `path: text` is a
//! file (`;` a line break), `under dep <- roots…` a dependent with its
//! roots in ascending order; every other dead file is a root, and
//! `roots N` / `dead N` are the counts the document must carry.

use crate::common;
use crate::common::gates::core_bin;
use crate::common::mcp::McpSession;
use codeeraser::graph::deadcode;

const TREES: &str = r#"
== deadcode-roots-chain
a.ts: import './b';
b.ts: import './c';
c.ts: export {};
under b.ts <- a.ts
under c.ts <- a.ts
dead 3
roots 1
== deadcode-roots-cycle
x.ts: import './y';
y.ts: import './x';
dead 2
roots 2
== deadcode-roots-two
p.ts: import './d';
q.ts: import './d';
d.ts: export {};
under d.ts <- p.ts q.ts
dead 3
roots 2
"#;

/// One tree of the table: its name, its files, its dependents with
/// their roots, and the two counts.
struct Tree {
    name: String,
    files: Vec<(String, String)>,
    under: Vec<(String, Vec<String>)>,
    dead: usize,
    roots: usize,
}

fn trees() -> Vec<Tree> {
    let mut out: Vec<Tree> = Vec::new();
    for line in TREES.trim().lines() {
        if let Some(name) = line.strip_prefix("== ") {
            out.push(Tree {
                name: name.into(),
                files: vec![],
                under: vec![],
                dead: 0,
                roots: 0,
            });
            continue;
        }
        let t = out.last_mut().expect("a tree opens first");
        let (head, rest) = line.split_once(' ').expect("a row");
        match head {
            "under" => {
                let (dep, roots) = rest.split_once(" <- ").expect("dep <- roots");
                t.under
                    .push((dep.into(), roots.split(' ').map(String::from).collect()));
            }
            "dead" => t.dead = rest.parse().expect("a count"),
            "roots" => t.roots = rest.parse().expect("a count"),
            _ => {
                let path = head.strip_suffix(':').expect("path: text");
                t.files
                    .push((path.into(), format!("{}\n", rest.replace(';', ";\n"))));
            }
        }
    }
    out
}

fn tree_dir(t: &Tree) -> std::path::PathBuf {
    let files: Vec<(&str, &str)> = t
        .files
        .iter()
        .map(|(p, s)| (p.as_str(), s.as_str()))
        .collect();
    common::fixture(&t.name, &files).dir.clone()
}

#[test]
fn the_document_names_the_roots_and_each_dependents_roots() {
    let core = core_bin();
    for t in trees() {
        let r = deadcode::run(&tree_dir(&t), None, &core).expect("run");
        assert_eq!(r.dead.len(), t.dead, "{}: the dead rows", t.name);
        assert_eq!(
            (r.counts.roots, r.counts.dependents),
            (Some(t.roots), Some(t.dead - t.roots)),
            "{}",
            t.name
        );
        assert!(!r.under_dropped, "{}: nothing dropped", t.name);
        for d in &r.dead {
            let want = t
                .under
                .iter()
                .find(|(dep, _)| *dep == d.path)
                .map(|(_, rs)| rs.clone());
            assert_eq!(d.root, Some(want.is_none()), "{}: {} root", t.name, d.path);
            assert_eq!(d.under, want, "{}: {} under", t.name, d.path);
        }
    }
}

/// Each tree's console in both languages: every root line before every
/// dependent line, a root's line counts what falls with it from one up
/// and says nothing at zero (early review G2), a dependent's names its
/// roots, and the summary counts the roots.
#[test]
fn the_console_prints_roots_first_in_both_languages() {
    let core = core_bin();
    let words = [
        // a row starts with the word; these trees have no baseline, so no
        // root carries a state mark (5B, early review F3)
        (
            "en",
            "unreferenced",
            "file(s) fall with it",
            " — under ",
            "unreferenced (",
        ),
        ("zh", "无引用", "个文件随它无引用", "——随根 ", "无引用（"),
    ];
    for t in trees() {
        let dir = tree_dir(&t);
        for (lang, row, falls, under, summary) in words {
            let out = common::run_ce_env(
                &dir,
                &["deadcode", ".", "--core", &core],
                &[("CE_LANG", lang)],
            );
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            let rows: Vec<&str> = text.lines().filter(|l| l.starts_with(row)).collect();
            assert_eq!(rows.len(), t.dead, "{} {lang}: {text}", t.name);
            let roots = t.dead - t.under.len();
            for l in &rows[..roots] {
                let n = t
                    .under
                    .iter()
                    .filter(|(_, rs)| rs.contains(&row_path(l)))
                    .count();
                let said = if n == 0 {
                    !l.contains(falls)
                } else {
                    l.contains(&format!("{n} {falls}"))
                };
                assert!(said, "{} {lang}: root {n} falling: {l}", t.name);
            }
            for (l, (dep, rs)) in rows[roots..].iter().zip(&t.under) {
                assert!(
                    l.contains(dep.as_str()) && l.ends_with(&format!("{under}{}", rs.join(", "))),
                    "{} {lang}: {l}",
                    t.name
                );
            }
            assert!(
                text.contains(&format!("{} {summary}{}", t.dead, t.roots)),
                "{} {lang}: {text}",
                t.name
            );
        }
    }
}

/// The path a console row names: after the row word and its colon, up
/// to the two spaces before the verdict.
fn row_path(line: &str) -> String {
    let rest = line
        .split_once(": ")
        .or_else(|| line.split_once('：'))
        .map_or("", |(_, r)| r);
    rest.split("  ").next().unwrap_or("").to_string()
}

/// The MCP tool returns the same document: the grouping's keys arrive.
#[test]
fn the_mcp_tool_carries_the_grouping() {
    let t = trees().into_iter().next().expect("the chain");
    let mut s = McpSession::over(&tree_dir(&t));
    let text = s.relayed(1, "deadcode", serde_json::json!({}));
    s.finish();
    let doc: serde_json::Value = serde_json::from_str(&text).expect("a document");
    assert_eq!(doc["counts"]["roots"], 1, "{doc}");
    assert_eq!(doc["counts"]["dependents"], 2, "{doc}");
    assert_eq!(doc["dead"][0]["root"], true, "{doc}");
    assert_eq!(
        doc["dead"][1]["under"],
        serde_json::json!(["a.ts"]),
        "{doc}"
    );
}
