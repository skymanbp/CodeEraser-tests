//! The 6.4.0 fence batch through the real binary (plan v2.18 step
//! #14, piece (b)). O40: a committed row whose file an ignore line
//! hides is DROPPED — `ce check` fails by name and only the fence act
//! owns it — while a deleted file's row is simply removed. O33: `ce
//! scan` judges the same config fence `ce check` does, names it on
//! the console and in the 0.2.0 `failed` key, and the PreToolUse hook
//! judges budgets with the SHIPPED lines while the config drifts.
//! O59: `[graph] scc_floor` is refused at 0 by name at load, and at 1
//! rides the self-loop table — a loop-free tree judges exactly as it
//! does under the shipped floor.

use crate::common;
use crate::common::{FENCE, WHOLESALE, core_bin, declare, rust_fn, tmp};
use codeeraser::score::baseline::file_entity;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn project(tag: &str) -> PathBuf {
    let dir = tmp(tag);
    for (name, n) in [("a.rs", 1), ("b.rs", 2)] {
        std::fs::write(dir.join(name), rust_fn(n)).expect(name);
    }
    dir
}

/// One family run over `dir`, asserting the exit code; returns stdout.
fn expect_exit(dir: &Path, args: &[&str], env: &[(&str, &str)], code: i32) -> String {
    let (got, out, err) = common::ce_triple(dir, args, env);
    assert_eq!(got, Some(code), "{args:?} {env:?}:\n{out}\n{err}");
    out
}

/// The family's JSON report, asserting the exit code: the fail bit
/// rides in the code, the names in the document.
fn report(dir: &Path, family: &str, core: &str, code: i32) -> Value {
    let out = expect_exit(
        dir,
        &[family, ".", "--core", core, "--format", "json"],
        &[],
        code,
    );
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{family} json: {e}\n{out}"))
}

/// One run refused by name — exit `code` (1: a verdict the act will
/// not write over; 2: `ce <name>:` fault, the config's) with every
/// `needle` on stderr, which is returned.
fn refused(dir: &Path, args: &[&str], code: i32, needles: &[&str]) -> String {
    let (got, out, err) = common::ce_triple(dir, args, &[]);
    assert_eq!(got, Some(code), "{args:?}:\n{out}\n{err}");
    for needle in needles {
        assert!(
            err.contains(needle),
            "{args:?}: {needle:?} names the refusal: {err}"
        );
    }
    err
}

fn establish(dir: &Path, core: &str) {
    expect_exit(dir, &["baseline", ".", "--core", core], WHOLESALE, 0);
}

fn ratchet_rows(doc: &Value) -> Vec<[u64; 3]> {
    serde_json::from_value(doc["ratchet"]["dropped"].clone()).expect("dropped rows")
}

/// O40. A clean tree answers an empty `dropped`; hiding b.rs behind a
/// `.ceignore` line drops its committed rows (named on the console,
/// in the report, and by `ce baseline`, which only the fence act
/// clears — writing the baseline WITHOUT them); deleting b.rs instead
/// removes them, which no condition holds on.
#[test]
fn an_ignored_file_drops_its_rows_and_a_deleted_file_removes_them() {
    let dir = project("fence-dropped");
    let core = core_bin();
    let check = ["check", ".", "--core", &core];
    establish(&dir, &core);
    let clean = report(&dir, "check", &core, 0);
    assert_eq!(clean["schema"], "ce.check-report/0.5.0");
    assert_eq!(clean["ratchet"]["dropped"], serde_json::json!([]));
    assert_eq!(clean["ratchet"]["failed"], serde_json::json!([]));

    std::fs::write(dir.join(".ceignore"), "b.rs\n").expect(".ceignore");
    let out = expect_exit(&dir, &check, &[], 1);
    assert!(
        out.contains("-> FAIL (failed: rows_dropped)\n"),
        "named on the console: {out}"
    );
    let hidden = report(&dir, "check", &core, 1);
    assert_eq!(
        hidden["ratchet"]["failed"],
        serde_json::json!(["rows_dropped"])
    );
    let dropped = ratchet_rows(&hidden);
    assert!(
        dropped
            .iter()
            .any(|r| r[0] == file_entity("b.rs") && r[1] == 0)
            && dropped.iter().all(|r| r[0] != file_entity("a.rs")),
        "b.rs's line row is dropped, a.rs is measured: {dropped:?}"
    );

    // the routine act refuses and names the narrow one
    refused(
        &dir,
        &["baseline", ".", "--core", &core],
        1,
        &["rows_dropped", "CE_ACCEPT_FENCE=1"],
    );
    let out = expect_exit(&dir, &["baseline", ".", "--core", &core], FENCE, 0);
    assert!(
        out.contains("fence accepted") && out.contains("rows_dropped"),
        "{out}"
    );
    let owned: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("ce-baseline.json")).expect("baseline"),
    )
    .expect("baseline json");
    let rows: Vec<[u64; 3]> =
        serde_json::from_value(owned["continuous"].clone()).expect("continuous");
    assert!(
        rows.iter().all(|r| r[0] != file_entity("b.rs")),
        "owning the exclusion writes the baseline without b.rs: {rows:?}"
    );
    expect_exit(&dir, &check, &[], 0);

    // the counterfactual: the same rows gone with their file
    std::fs::remove_file(dir.join(".ceignore")).expect("unhide");
    establish(&dir, &core);
    std::fs::remove_file(dir.join("b.rs")).expect("delete b.rs");
    expect_exit(&dir, &check, &[], 0);
    let deleted = report(&dir, "check", &core, 0);
    assert!(
        ratchet_rows(&deleted).is_empty(),
        "a deletion is a removal, not a drop"
    );
    assert_eq!(deleted["ratchet"]["failed"], serde_json::json!([]));
}

/// O33. Unfenced, the scan names nothing; under the drift `ce check`
/// fails on, `ce scan` exits 1 with `knobs_digest` on its console
/// line (both languages) and in the report's `failed`; the fence act
/// clears it for both families at once.
#[test]
fn scan_judges_the_config_fence_and_names_it() {
    let dir = project("fence-scan");
    let core = core_bin();
    let scan = ["scan", ".", "--core", &core];
    let out = expect_exit(&dir, &scan, &[], 0);
    assert!(
        out.contains(" fail\n") && !out.contains("FAIL"),
        "unfenced: the summary line keeps its bytes: {out}"
    );
    let unfenced = report(&dir, "scan", &core, 0);
    assert_eq!(unfenced["schema"], "ce.scan-report/0.2.0");
    assert_eq!(unfenced["failed"], serde_json::json!([]));

    common::seed_budget(&dir, 41);
    establish(&dir, &core);
    declare(&dir, "\n");
    // (environment, the exact suffix): the same drift, both languages
    for (env, want) in [
        (&[][..], "fail -> FAIL (failed: knobs_digest)\n"),
        (
            &[("CE_LANG", "zh")][..],
            "fail -> FAIL（失败条件：knobs_digest）\n",
        ),
    ] {
        let out = expect_exit(&dir, &scan, env, 1);
        assert!(out.contains(want), "{env:?}: named on the console: {out}");
    }
    let drifted = report(&dir, "scan", &core, 1);
    assert_eq!(drifted["failed"], serde_json::json!(["knobs_digest"]));
    expect_exit(&dir, &["check", ".", "--core", &core], &[], 1);

    expect_exit(&dir, &["baseline", ".", "--core", &core], FENCE, 0);
    expect_exit(&dir, &scan, &[], 0);
}

/// O33, the hook. A config that declares no hard line lets a 900-line
/// write through — until a baseline fences the tree under a DIFFERENT
/// config, when the hook judges with the shipped 750 and says why.
#[test]
fn the_guard_judges_shipped_budgets_while_the_config_drifts() {
    let dir = project("fence-guard");
    let core = core_bin();
    let big = "// filler\n".repeat(900);
    let no_line = "[thresholds]\nfile_lines_fail = 0\n";
    let env = common::pretooluse_envelope_at(&dir, "c.rs", "Write", &big);
    let silent = |why: &str| {
        let out = common::run_hook(&dir, &["probe", "--hook"], &env);
        assert!(out.trim().is_empty(), "{why}: nothing fires: {out}");
    };
    declare(&dir, no_line);
    silent("unfenced, no hard line declared");

    // the fence: established under the shipped config, drifted after
    declare(&dir, "\n");
    establish(&dir, &core);
    declare(&dir, no_line);
    let reason = common::expect_write_denied(&dir, "c.rs", &big, "hard budget of 750");
    assert!(
        reason.contains("drifted from the fenced baseline"),
        "the reason names the fence: {reason}"
    );
    // the fence act re-pins under the declared config: the declared
    // budget (none) judges again
    expect_exit(&dir, &["baseline", ".", "--core", &core], FENCE, 0);
    silent("re-pinned: the declared budget");
}

/// O59. `scc_floor = 0` is refused at load with the key named, before
/// anything is measured; `scc_floor = 1` rides the self-loop table
/// and judges a loop-free tree axis for axis as the shipped floor.
#[test]
fn scc_floor_is_refused_at_zero_and_rides_at_one() {
    let dir = project("fence-scc");
    let core = core_bin();
    declare(&dir, "[graph]\nscc_floor = 0\n");
    refused(
        &dir,
        &["check", ".", "--core", &core],
        2,
        &["[graph] scc_floor must be >= 1"],
    );

    let mut axes = Vec::new();
    for toml in ["\n", "[graph]\nscc_floor = 1\n"] {
        declare(&dir, toml);
        establish(&dir, &core);
        let doc = report(&dir, "check", &core, 0);
        assert_eq!(doc["ratchet"]["failed"], serde_json::json!([]), "{toml}");
        axes.push((doc["axes"].clone(), doc["score"].clone()));
    }
    assert_eq!(
        axes[0], axes[1],
        "no self-loop: the same axes and score under both floors"
    );
}
