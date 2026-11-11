//! An Edit the hook cannot apply says so (v2.35 guard lane, GD3). The
//! size, flow and tombstone legs need the applied text and stay silent
//! without it — correct, but it used to be invisible: the probe line
//! looked like any clean write. Now it names why — the target is
//! absent, unreadable (a degraded stamp `ce doctor` counts), or
//! `old_string` matches nothing or more than once — and a Write, which
//! is its own text, never carries the key.

use crate::common::{self, last_observe, run_hook, tmp, write_all};

#[test]
fn an_edit_that_cannot_apply_leaves_its_reason_on_the_probe_line() {
    let dir = tmp("guard-unapplied");
    write_all(&dir, &[("b.rs", "// one\n// same\n// same\n")]);
    common::build_index(&dir);
    std::fs::create_dir_all(dir.join("d.rs")).expect("a directory at the path");
    // (target, old_string, the reason, a degraded stamp)
    let cases = "gone.rs|// one|absent|0;d.rs|// one|unreadable|1;b.rs|// zero|no_match|0;b.rs|// same|ambiguous|0";
    for case in cases.split(';') {
        let f: Vec<&str> = case.split('|').collect();
        let env = common::pretooluse_edit_envelope(&dir, f[0], f[1], "// new\n");
        let out = run_hook(&dir, &["probe", "--hook"], &env);
        assert!(out.trim().is_empty(), "{case}: nothing to say: {out}");
        let line = last_observe(&dir);
        assert_eq!(
            (line["event"].as_str(), line["unapplied"].as_str()),
            (Some("probe"), Some(f[2])),
            "{case}: {line}"
        );
        assert_eq!(
            line["apply"]["degraded"].is_string(),
            f[3] == "1",
            "{case}: {line}"
        );
    }
    assert_eq!(
        codeeraser::health::degraded_runs(&dir).0,
        1,
        "the unreadable target is the one degraded run ce doctor counts: {:?}",
        common::observe_lines(&dir)
    );
    let w = common::pretooluse_envelope_at(&dir, "b.rs", "Write", "// whole\n");
    run_hook(&dir, &["probe", "--hook"], &w);
    assert!(
        last_observe(&dir).get("unapplied").is_none(),
        "a Write always applies"
    );
}
