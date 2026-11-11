//! `ce erase` end-to-end against the contract's own acceptance list
//! (docs/reference/erase.md §acceptance): the three classes plan on a
//! fixture tree and the one that erases — the verbatim doc span —
//! applies with a converging re-run, while the two whole-file classes
//! stay advisory rows with their reason (9.1.0, lane v235_licence): a
//! private dead file and a private dead twin refused as
//! `reachable_by_name`, a public one refused by RG10, a live T2 twin
//! never planned. Refusals (dirty worktree / drifted hash / non-repo /
//! a hand-built whole-file row) fire BY NAME without touching
//! anything, and two plans over one tree are byte-identical. The
//! self-repo zero-row gate is CI's `ce erase .. --check` leg, not this
//! file.

use crate::common::gitio::git;
use crate::common::{commit_all, core_bin, tmp};
use codeeraser::erase;
use codeeraser::faces;
use std::path::{Path, PathBuf};

/// 50+ admitted words, byte-identical in both hosts — the verbatim
/// full-segment class.
pub(crate) const PARA: &str = "alpha beta gamma delta epsilon zeta eta theta iota kappa \
lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega one two \
three four five six seven eight nine ten eleven twelve thirteen fourteen \
fifteen sixteen seventeen eighteen nineteen twenty apple pear plum grape \
melon berry stone river cloud";

/// ~58 normalized tokens, one whole unit — the T1 side.
const FN_T1: &str = "def compute_total(values):
    values = list(values)
    count = len(values) + 3
    total = 0
    for item in values:
        if item > 3:
            total = total + item * 2
        else:
            total = total - 1
    result = total * 7 + count
    print(result)
    return result
";

/// The same shape with renamed identifiers and shifted literals —
/// T2 under normalization, byte-different: the advisory class.
const FN_T2: &str = "def sum_amounts(entries):
    entries = list(entries)
    width = len(entries) + 4
    acc = 0
    for row in entries:
        if row > 5:
            acc = acc + row * 6
        else:
            acc = acc - 2
    outcome = acc * 8 + width
    print(outcome)
    return outcome
";

/// A second whole unit, PRIVATE by Python's underscore rule and of a
/// different shape from FN_T1 (no T2 kinship): its dead copy passes
/// every fact the twin class reads, and is still refused — the file
/// could run by its name (9.1.0).
const FN_PRIV: &str = "def _spare_total(items, base):
    items = list(items)
    index = 0
    total = base
    while index < len(items):
        current = items[index]
        total = total + current * current
        index = index + 1
    if total > 100:
        total = total - 100
    print(total, index, base)
    return total
";

pub(crate) fn seed(dir: &Path) {
    let w = |name: &str, text: String| std::fs::write(dir.join(name), text).expect(name);
    w(
        "README.md",
        format!("# fixture\n\n[doc2](doc2.md)\n\n{PARA}\n"),
    );
    w("doc2.md", format!("# second\n\n{PARA}\n"));
    w("orphan.md", "an unlinked note nobody references\n".into());
    w("__main__.py", format!("import t2\n\n{FN_T1}\n{FN_PRIV}"));
    w("copy.py", FN_T1.into());
    w("spare.py", FN_PRIV.into());
    w("t2.py", FN_T2.into());
    git(dir, &["init", "-q"]);
    commit_all(dir, "seed");
}

fn row<'a>(p: &'a erase::Plan, class: &str, path: &str) -> &'a erase::Row {
    p.rows
        .iter()
        .find(|r| r.class == class && r.path == path)
        .unwrap_or_else(|| panic!("no {class} row for {path}"))
}

/// An eraseable row no planner made, at the file's current hash: a
/// span names a `verbatim_doc` row, none a whole-file `dead_file` row.
pub(crate) fn hand_built(root: &Path, path: &str, span: Option<(i64, i64)>) -> erase::Row {
    let bytes = std::fs::read(root.join(path)).expect("read");
    erase::Row {
        class: if span.is_some() {
            "verbatim_doc"
        } else {
            "dead_file"
        },
        eraseable: true,
        reason: "eraseable",
        path: path.into(),
        span,
        provenance: String::new(),
        sites: 0,
        hash: codeeraser::dedup::tokens::fnv1a(&bytes),
    }
}

/// The seeded fixture under a fresh `tmp(name)`, the core, and its plan.
pub(crate) fn planned(name: &str) -> (PathBuf, String, erase::Plan) {
    let dir = tmp(name);
    seed(&dir);
    let core = core_bin();
    let plan = erase::plan(&dir, None, &core).expect("plan");
    (dir, core, plan)
}

/// One named refusal: apply errs with `needle` and touches nothing
/// (the orphan is every fixture's canary file).
pub(crate) fn refuses(dir: &Path, core: &str, plan: &erase::Plan, needle: &str) {
    let err = erase::apply_plan(dir, None, core, plan).expect_err(needle);
    assert!(err.to_string().contains(needle), "{err:#}");
    assert!(dir.join("orphan.md").exists(), "refusal touched nothing");
}

#[test]
fn plan_names_all_classes_and_is_deterministic() {
    let (_dir, _core, p) = planned("erase-plan");
    assert_eq!(p.counts.eraseable, 1, "the doc2 span alone");
    // orphan.md and spare.py are private, vouched and unreferenced —
    // every fact the dead_file and t1_twin classes read holds — and
    // each is refused by name: nothing in the tree references them,
    // which is not evidence that nothing uses them (9.1.0). spare.py's
    // twin row and its dead_file row name one target; the dead file's
    // refusal stands, the one about the file itself
    for path in ["orphan.md", "spare.py"] {
        let r = row(&p, "dead_file", path);
        assert!(
            !r.eraseable && r.reason == "reachable_by_name",
            "{path}: {}",
            r.reason
        );
    }
    assert!(
        !p.rows
            .iter()
            .any(|r| r.class == "t1_twin" && r.path == "spare.py"),
        "one row per whole-file target"
    );
    // copy.py declares `def compute_total(...)` — an export surface, so
    // its dead verdict is unref_public and RG10 names that categorical
    // bar first (6.1.0); the private files above reach the last bar
    let copy = row(&p, "dead_file", "copy.py");
    assert!(!copy.eraseable);
    assert_eq!(copy.reason, "public_surface");
    let doc = row(&p, "verbatim_doc", "doc2.md");
    assert!(
        doc.eraseable && doc.span.is_some(),
        "span row, not the file"
    );
    // the live T2 twin is named, never planned (acceptance row 3)
    let twin = row(&p, "t1_twin", "t2.py");
    assert!(!twin.eraseable);
    assert_eq!(twin.reason, "bytes_differ");
}

/// Acceptance row 4: two plans over one tree are byte-identical; the
/// diff face renders the eraseable span with its provenance and deletes
/// no file.
#[test]
fn two_plans_agree_and_the_diff_deletes_no_file() {
    let (dir, core, p) = planned("erase-plan-twice");
    let again = erase::plan(&dir, None, &core).expect("plan again");
    let plan_json = |q: &erase::Plan| serde_json::json!([q.rows, q.counts]).to_string();
    assert_eq!(plan_json(&p), plan_json(&again), "determinism pinned");
    let diff = erase::document::Diffs::of(&dir, &p)
        .expect("diff")
        .unified();
    assert!(
        diff.contains("## verbatim_doc") && diff.contains("fnv1a64:"),
        "{diff}"
    );
    assert!(!diff.contains("/dev/null"), "no file deleted: {diff}");
}

#[test]
fn apply_refuses_by_name_then_converges() {
    let (dir, core, plan) = planned("erase-apply");
    // dirty worktree refuses, nothing touched
    std::fs::write(dir.join("junk.txt"), "wip").expect("junk");
    refuses(&dir, &core, &plan, "worktree not clean");
    std::fs::remove_file(dir.join("junk.txt")).expect("rm junk");
    // a drifted target refuses by name (clean tree, changed bytes)
    let mut doc2 = std::fs::read_to_string(dir.join("doc2.md")).expect("doc2");
    doc2.push_str("\ntrailing edit\n");
    std::fs::write(dir.join("doc2.md"), doc2).expect("drift");
    commit_all(&dir, "drift");
    refuses(&dir, &core, &plan, "content changed since planning");
    // a fresh plan applies, converges, and leaves the audit trail
    let plan = erase::plan(&dir, None, &core).expect("re-plan");
    let n = erase::apply_plan(&dir, None, &core, &plan).expect("apply");
    assert_eq!(n, 1);
    // no whole-file row erases: the unreferenced files survive the
    // apply, not just the plan (9.1.0), and so does the export surface
    // (6.1.0)
    for kept in ["orphan.md", "spare.py", "copy.py"] {
        assert!(dir.join(kept).exists(), "{kept} survives the apply");
    }
    let doc2 = std::fs::read_to_string(dir.join("doc2.md")).expect("doc2");
    assert!(!doc2.contains("alpha beta gamma"), "span spliced out");
    assert!(doc2.contains("# second"), "the rest of the file survives");
    assert_trail(&dir, &core, &plan);
    let after = erase::plan(&dir, None, &core).expect("post-plan");
    assert_eq!(after.counts.eraseable, 0, "converged");
}

/// The audit trail after an apply of `plan`: one record per applied
/// row, and its reader (O50) hands back exactly what apply wrote —
/// every applied row by class and target, nothing unreadable — with
/// the face's document carrying the same counts.
fn assert_trail(dir: &Path, core: &str, plan: &erase::Plan) {
    let log = std::fs::read_to_string(dir.join(".ce/erase-log.ndjson")).expect("log");
    assert_eq!(log.lines().count(), 1, "one record per applied row");
    assert!(log.lines().all(|l| l.contains("ce.erase-log/0.1.0")));
    let trail = erase::log::read(dir).expect("read the trail");
    assert!(trail.present && trail.unreadable.is_empty(), "{trail:?}");
    let mut written: Vec<(String, String)> = trail
        .rows
        .iter()
        .map(|r| (r.class.clone(), r.path.clone()))
        .collect();
    let mut applied: Vec<(String, String)> = plan
        .eraseable()
        .map(|r| (r.class.to_string(), r.path.clone()))
        .collect();
    written.sort();
    applied.sort();
    assert_eq!(written, applied, "the log names the applied rows");
    let doc = faces::doc(faces::erase_log(dir, core)).expect("erase_log face");
    let schema = doc["schema"].as_str().unwrap_or("");
    assert!(schema.starts_with("ce.erase-trail-report/"), "{schema}");
    assert_eq!(doc["counts"]["rows"], 1);
    assert_eq!(doc["counts"]["unreadable"], 0);
}
