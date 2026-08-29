//! `ce baseline`'s named acts through the real binary (plan v2.18
//! step #14: O30 root-only, O31 no unnamed establish, O35 the narrow
//! fence act) and `ce check`'s named console failure (O36). Every
//! refusal that must precede measurement runs against a core path
//! that does not exist: the old binary reached the core on each of
//! these roads, so its message — a spawn failure naming that path —
//! is not the one asserted here.

use crate::common;
use crate::common::{FENCE, WHOLESALE, core_bin, declare, rust_fn, tmp};
use serde_json::Value;
use std::path::{Path, PathBuf};

const NO_CORE: &str = "ce-core-that-does-not-exist";

/// A project with one source file and a non-default knob — its
/// digest is Some, so reverting ce.toml to the shipped default is a
/// `knobs_digest` drift and nothing else.
fn project(tag: &str) -> PathBuf {
    let dir = tmp(tag);
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    common::seed_budget(&dir, 41);
    dir
}

/// One `ce baseline <scope>` run: (exit code, stderr, stdout).
fn baseline(
    dir: &Path,
    scope: &str,
    env: &[(&str, &str)],
    core: &str,
) -> (Option<i32>, String, String) {
    let (code, out, err) = common::ce_triple(dir, &["baseline", scope, "--core", core], env);
    (code, err, out)
}

fn committed(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join("ce-baseline.json")).ok()
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).expect("baseline json")
}

/// Every road that must refuse BEFORE the core: no file and no act
/// (or the fence act alone, which has nothing to re-pin), a scope
/// below the root even under the wholesale act, and a present file
/// that is not a baseline document. Nothing is written anywhere.
#[test]
fn baseline_refuses_by_name_before_any_measurement() {
    let dir = project("baseline-refusals");
    std::fs::create_dir_all(dir.join("pkg")).expect("mkdir");
    // (scope, environment, the act the refusal must name)
    type Case<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a str);
    let table: [Case; 3] = [
        (".", &[], "set CE_ACCEPT_BASELINE=1"),
        (".", FENCE, "set CE_ACCEPT_BASELINE=1"),
        ("pkg", WHOLESALE, "inside project"),
    ];
    for (scope, env, names) in table {
        let (code, err, _) = baseline(&dir, scope, env, NO_CORE);
        assert_eq!(code, Some(1), "{scope} {env:?}: {err}");
        assert!(err.contains(names), "{scope} {env:?}: names the act: {err}");
        assert!(
            !err.contains(NO_CORE),
            "{scope} {env:?}: the core was never spawned: {err}"
        );
        assert!(
            committed(&dir).is_none() && !dir.join("pkg/ce-baseline.json").exists(),
            "{scope} {env:?}: nothing written"
        );
    }
    // a present file that is not a baseline document is an ERROR (the
    // usage exit, 2), not a policy refusal: nothing was judged, and
    // the old binary re-established the floor wholesale from it
    std::fs::write(dir.join("ce-baseline.json"), "null\n").expect("seed");
    let (code, err, _) = baseline(&dir, ".", &[], NO_CORE);
    assert_eq!(code, Some(2), "{err}");
    assert!(
        err.contains("not a baseline document") && !err.contains(NO_CORE),
        "{err}"
    );
    assert_eq!(
        committed(&dir).as_deref(),
        Some("null\n"),
        "the broken file is left as evidence"
    );
}

/// The road every act leg starts from: wholesale creates the file;
/// the routine road keeps it byte-identical when nothing moved; a
/// digest drift refuses without an act and names both acts; the
/// fence act re-pins the SAME ceilings under the current digest.
/// Returns the project, the core, the first baseline's document and
/// the re-pinned text.
fn established_then_fenced(tag: &str) -> (PathBuf, String, Value, String) {
    let dir = project(tag);
    let core = core_bin();
    let (code, err, out) = baseline(&dir, ".", WHOLESALE, &core);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.contains("baseline written"), "{out}");
    let established = committed(&dir).expect("written");
    assert!(
        established.contains("knobsDigest"),
        "the digest rode: {established}"
    );

    let (code, err, _) = baseline(&dir, ".", &[], &core);
    assert_eq!(code, Some(0), "routine, nothing moved: {err}");
    assert_eq!(
        committed(&dir).as_deref(),
        Some(established.as_str()),
        "byte-identical"
    );

    declare(&dir, "\n");
    let (code, err, _) = baseline(&dir, ".", &[], &core);
    assert_eq!(code, Some(1), "{err}");
    assert!(
        err.contains("knobs_digest")
            && err.contains("CE_ACCEPT_FENCE=1")
            && err.contains("CE_ACCEPT_BASELINE=1"),
        "names what held and both acts: {err}"
    );
    assert_eq!(
        committed(&dir).as_deref(),
        Some(established.as_str()),
        "refused: untouched"
    );

    let (code, err, out) = baseline(&dir, ".", FENCE, &core);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        out.contains("fence accepted") && out.contains("knobs_digest"),
        "{out}"
    );
    let repinned = committed(&dir).expect("re-pinned");
    let (old, new) = (parse(&established), parse(&repinned));
    assert_eq!(new["continuous"], old["continuous"], "ceilings kept");
    assert_eq!(
        new["discrete"], old["discrete"],
        "members current (unchanged here)"
    );
    assert!(
        new.get("knobsDigest").is_none(),
        "a default config records no digest: {repinned}"
    );
    (dir, core, old, repinned)
}

/// The acts write what they promise, and after the fence act the
/// next check is green.
#[test]
fn the_named_acts_write_what_they_promise() {
    let (dir, core, _, _) = established_then_fenced("baseline-acts");
    let check = common::run_ce(&dir, &["check", ".", "--core", &core]);
    assert!(
        check.status.success(),
        "re-pinned: the next check is green: {check:?}"
    );
}

/// Growth refuses under the fence act by name (the narrow act does
/// not launder it), and only wholesale adopts it.
#[test]
fn growth_needs_the_wholesale_act() {
    let (dir, core, old, repinned) = established_then_fenced("baseline-growth");
    common::append(&dir.join("a.rs"), &"// filler\n".repeat(200));
    for env in [&[][..], FENCE] {
        let (code, err, _) = baseline(&dir, ".", env, &core);
        assert_eq!(code, Some(1), "{env:?}: {err}");
        assert!(
            err.contains("ratchet_over") && err.contains("CE_ACCEPT_BASELINE=1"),
            "{env:?}: growth is named and needs the wholesale act: {err}"
        );
        assert_eq!(
            committed(&dir).as_deref(),
            Some(repinned.as_str()),
            "{env:?}: untouched"
        );
    }
    let (code, err, _) = baseline(&dir, ".", WHOLESALE, &core);
    assert_eq!(code, Some(0), "{err}");
    assert_ne!(
        parse(&committed(&dir).expect("grown"))["continuous"],
        old["continuous"],
        "wholesale adopts the grown ceiling — the road the fence act must differ from"
    );
}

/// `ce check` names what held on its console line, verbatim in the
/// core's order, in both languages — and a passing line keeps its
/// bytes (no suffix, ever). The JSON face carried the names already.
#[test]
fn check_names_the_held_conditions_on_the_console() {
    let dir = project("check-names");
    let core = core_bin();
    let (code, err, _) = baseline(&dir, ".", WHOLESALE, &core);
    assert_eq!(code, Some(0), "{err}");
    let check = |env: &[(&str, &str)], extra: &[&str]| {
        let mut args = vec!["check", ".", "--core", &core];
        args.extend_from_slice(extra);
        let out = common::run_ce_env(&dir, &args, env);
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
        )
    };
    let (code, out) = check(&[], &[]);
    assert_eq!(code, Some(0), "{out}");
    assert!(
        out.contains("0 tolerance drawn -> pass\n"),
        "pass bytes: {out}"
    );

    declare(&dir, "\n");
    common::append(&dir.join("a.rs"), &"// filler\n".repeat(200));
    // (environment, extra args, the exact line): the floor row sits
    // BETWEEN the two others in the core's order, which is what
    // proves the renderer never sorts
    let table = [
        (
            &[][..],
            &[][..],
            "ratchet: 0 added, 0 removed, 1 over, 0 tolerance drawn -> FAIL (failed: ratchet_over, knobs_digest)\n",
        ),
        (
            &[("CE_LANG", "zh")][..],
            &[][..],
            "棘轮：新增 0，移除 0，超限 1，动用容差 0 -> FAIL（失败条件：ratchet_over, knobs_digest）\n",
        ),
        (
            &[][..],
            &["--fail-under", "1000"][..],
            "-> FAIL (failed: ratchet_over, floor, knobs_digest)\n",
        ),
    ];
    for (env, extra, want) in table {
        let (code, out) = check(env, extra);
        assert_eq!(code, Some(1), "{env:?} {extra:?}: {out}");
        assert!(
            out.contains(want),
            "{env:?} {extra:?}: verbatim, in the core's order:\n{out}"
        );
    }
    let out = common::run_ce(&dir, &["check", ".", "--core", &core, "--format", "json"]);
    let doc: Value = serde_json::from_slice(&out.stdout).expect("report json");
    assert_eq!(
        doc["ratchet"]["failed"],
        serde_json::json!(["ratchet_over", "knobs_digest"])
    );
}
