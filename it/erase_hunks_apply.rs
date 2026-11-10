//! The plan's unified diff is one a tool applies (v2.35 G6, R3-14):
//! rendered for span rows whose context windows touch, one far span,
//! and a short file whose every line a span removes (the new side
//! empty), `git apply` takes it in a copy of the tree, and the copy
//! then reads byte for byte what `ce erase --apply` leaves in the
//! original. The hand-built plan rides the real apply: its
//! preconditions (clean worktree, content hashes) and its convergence
//! re-plan.

use crate::common::{self, core_bin, tmp};
use codeeraser::erase;
use std::path::Path;
use std::process::Command;

/// `git apply <args> <patch>` in `dir`: (succeeded, stderr). No line
/// ending translation: the erase writes the file's own bytes back, and
/// a machine-wide `core.autocrlf` would rewrite the patched lines.
fn git_apply(dir: &Path, args: &[&str], patch: &Path) -> (bool, String) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "core.autocrlf=false", "apply"])
        .args(args)
        .arg(patch)
        .output()
        .expect("git apply");
    let said = String::from_utf8_lossy(&out.stderr).into_owned();
    (out.status.success(), said)
}

fn span_row(path: &str, span: (i64, i64), text: &str) -> erase::Row {
    erase::Row {
        class: "verbatim_doc",
        eraseable: true,
        reason: "",
        path: path.into(),
        span: Some(span),
        provenance: "docdup".into(),
        sites: 0,
        hash: codeeraser::dedup::tokens::fnv1a(text.as_bytes()),
    }
}

#[test]
fn git_applies_the_plan_diff_and_lands_where_apply_does() {
    let long: String = (1..=20).map(|n| format!("l{n}\n")).collect();
    let short = "s1\ns2\ns3\n";
    let files = [("f.md", long.as_str()), ("short.md", short)];
    let (dir, copy) = (tmp("erase-hunks-apply"), tmp("erase-hunks-copy"));
    common::write_all(&dir, &files);
    common::write_all(&copy, &files);
    common::init_and_commit(&dir, "fixture");

    let core = core_bin();
    let mut plan = erase::plan(&dir, None, &core).expect("plan");
    let spans = [(5, 6), (9, 10), (18, 18)];
    plan.rows = spans.iter().map(|&s| span_row("f.md", s, &long)).collect();
    plan.rows.push(span_row("short.md", (1, 3), short));
    let diffs = erase::render::file_diffs(&dir, &plan).expect("diff");
    let patch = copy.join("plan.diff");
    let unified: String = diffs.iter().map(|(_, d)| d.clone() + "\n").collect();
    std::fs::write(&patch, &unified).expect("patch");
    // the short file's hunk empties its new side: written `+1,0` (GNU
    // diff writes `+0,0`), and git 2.52 applies either
    assert!(unified.contains("@@ -1,3 +1,0 @@"), "{unified}");

    let (checked, said) = git_apply(&copy, &["--check"], &patch);
    assert!(checked, "git apply --check refused:\n{said}\n{unified}");
    let (applied, said) = git_apply(&copy, &[], &patch);
    assert!(applied, "git apply refused:\n{said}");

    let n = erase::apply_plan(&dir, None, &core, &plan).expect("apply");
    assert_eq!(n, plan.rows.len(), "every row applied");
    for (name, _) in files {
        let read = |root: &Path| std::fs::read_to_string(root.join(name)).expect(name);
        assert_eq!(
            read(&copy),
            read(&dir),
            "{name}: the patch and the erase agree"
        );
    }
    for root in [&dir, &copy] {
        std::fs::remove_dir_all(root).ok();
    }
}
