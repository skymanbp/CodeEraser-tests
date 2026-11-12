//! The extent law's E on a tree where its exact value does not fit in
//! 64 bits (10.0.0, the 2C extent ruling). E is a sum of terms whose
//! denominators come from the data: here nineteen files are each
//! rewritten in one window commit, file i over p_i body lines (the
//! primes 17..97) with a two-line function appended, so each churn
//! excess n * (1 - t (rw + ap) / rw) carries its own prime in the
//! denominator: the reduced E is 2^116 over 2^107 (numerator and
//! denominator bits, read off the pre-ruling core's reply).
//! Until the ruling the reply carried E as `{num, den, lines}` and both
//! roads that read it refused the tree: `ce check --days 14` and
//! `ce join --days 14` exited 2 with "extent.num: not a 64-bit count"
//! (measured on this fixture with the pre-ruling build). The reply now
//! carries `{tenths, lines}`, computed where E is exact.
//!
//! The fixture is the defect's shape and nothing else: every file is
//! live (`__main__.py` imports each module), no clone (statement forms
//! drawn from a seeded sequence), no function over the complexity
//! ceiling, no file over the soft line, so the churn axis alone names
//! lines and its own share is the only one above 0.

use crate::common;
use std::path::{Path, PathBuf};

const PRIMES: [u64; 19] = [
    17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
];

/// One assignment per form; `{b}` the previous name, `{c}` a literal
/// that differs between the two commits on every line.
const FORMS: &str = "{a} = {b} + {c}|{a} = {b} * {c} - v0|{a} = ({b}, {c})|{a} = [{b}, {c}]|\
{a} = {b} if {c} else v0|{a} = {b} - {c}|{a} = {b}[{c}]|{a} = {b}.real + {c}|{a} = -{b} * {c}|\
{a} = {{b}: {c}}|{a} = {b} // {c} % v0|{a} = ({b}, [{c}], v0)";

/// Module i's text: `run` over n body lines drawn from `seed`, each
/// literal offset by `salt`; the rewrite also appends `extra`.
fn module(i: usize, n: u64, rewrite: bool) -> String {
    let forms: Vec<&str> = FORMS.split('|').collect();
    let (mut s, salt) = (
        1_000_003 * (i as u64 + 1) + if rewrite { 7 } else { 0 },
        if rewrite { 5000 } else { 0 },
    );
    let mut text = String::from("def run(v0):\n");
    for k in 0..n {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let line = forms[((s >> 33) % forms.len() as u64) as usize]
            .replace("{a}", &format!("v{k}"))
            .replace("{b}", &format!("v{}", k.saturating_sub(1)))
            .replace("{c}", &(salt + k).to_string());
        text.push_str(&format!("    {line}\n"));
    }
    text.push_str("    return v0\n");
    if rewrite {
        text.push_str(&format!("\n\ndef extra():\n    return {i}\n"));
    }
    text
}

fn write_tree(dir: &Path, rewrite: bool) {
    for (i, &p) in PRIMES.iter().enumerate() {
        std::fs::write(dir.join(format!("m{i}.py")), module(i, p, rewrite)).expect("module");
    }
}

/// The seed commit dated outside the 14-day window, the rewrite inside it.
fn fixture() -> PathBuf {
    let dir = common::tmp("check-extent-wide");
    write_tree(&dir, false);
    let main: String = (0..PRIMES.len())
        .map(|i| format!("import m{i}\n"))
        .collect();
    std::fs::write(dir.join("__main__.py"), main).expect("__main__.py");
    common::git(&dir, &["init", "-q"]);
    common::git(&dir, &["add", "."]);
    let old = "2026-01-01T12:00:00";
    common::git_env(
        &dir,
        &["commit", "-qm", "seed"],
        &[("GIT_AUTHOR_DATE", old), ("GIT_COMMITTER_DATE", old)],
    );
    write_tree(&dir, true);
    common::commit_all(&dir, "rewrite");
    dir
}

/// E times ten rounded half up, derived: file i has n = p + 6 lines
/// (def, p body lines, return, two blank lines, a two-line function),
/// rw = p rewritten and ap = 4 appended (the two blank lines count),
/// t = 1/2, so its excess is (p + 6)(p - 4) / 2p; summed exactly over
/// the common denominator 2 * prod p (about 2^95, inside u128).
fn derived_tenths() -> u128 {
    let den: u128 = 2 * PRIMES.iter().map(|&p| u128::from(p)).product::<u128>();
    let num: u128 = PRIMES
        .iter()
        .map(|&p| u128::from((p + 6) * (p - 4)) * (den / (2 * u128::from(p))))
        .sum();
    (20 * num + den) / (2 * den)
}

/// `ce <args> --format json .` in `dir`: exit 0 asserted, the document read.
fn judged(dir: &Path, face: &str) -> serde_json::Value {
    let out = common::run_ce(dir, &[face, "--days", "14", "--format", "json", "."]);
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(out.status.code(), Some(0), "{face}: {err}");
    serde_json::from_slice(&out.stdout).expect("document")
}

#[test]
fn an_extent_wider_than_64_bits_is_read_as_tenths_on_both_roads() {
    let dir = fixture();
    let doc = judged(&dir, "check");
    let ext = &doc["extent"];
    let [t, l] = ["tenths", "lines"].map(|k| ext[k].as_u64().expect(k));
    assert!(
        ext.get("num").is_none(),
        "no exact E leaves the core: {ext}"
    );
    assert!(t > 0 && t <= 10 * l, "0 < tenths <= 10 lines: {t} / {l}");
    assert_eq!(
        u128::from(t),
        derived_tenths(),
        "tenths = the exact E rounded half up"
    );
    // the defect's shape: churn alone names lines
    let shares: Vec<u64> = doc["axes"]
        .as_array()
        .expect("axes")
        .iter()
        .map(|r| r[1].as_u64().expect("share"))
        .collect();
    assert_eq!(
        (shares[5] > 0, [0, 1, 2, 3, 4, 6].map(|a| shares[a])),
        (true, [0; 6]),
        "{shares:?}"
    );
    let console = common::run_ce(&dir, &["check", "--days", "14", "."]);
    let head = String::from_utf8_lossy(&console.stdout).to_string();
    let printed = format!("findings on {}.{} of {l} lines", t / 10, t % 10);
    assert!(
        head.contains(&printed),
        "the console prints the document's tenths ({printed}): {head}"
    );
    let join = judged(&dir, "join");
    assert!(
        join["degraded"].is_null(),
        "the join read its verdict reply: {}",
        join["degraded"]
    );
}
