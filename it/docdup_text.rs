//! Plain text in the docdup family alone (plan v2.30 step 5b-8): a
//! `.txt` enters the index as prose paragraphs and the check universe
//! as a docdup opportunity, and no other gate reads it — not the scan's
//! size gates, not the guard's hard budget, not the graph's file nodes
//! (a page naming one lands an asset node, as before). CMake's list
//! file is the one `.txt` that is a script, and it stays outside every
//! arm.

use crate::common;
use crate::common::core_bin;
use codeeraser::graph::deadcode;
use std::path::{Path, PathBuf};

/// 66 words of prose, verbatim in two `.txt` files and in CMake's list
/// file: past the admission floor and the verbatim hard hit (50 words),
/// carrying no license marker.
const SHARED: &str = "Every ledger of this kind is written once and read many times, so the reader
is the one to please: each row lands whole, in the order it arrived, and no
later row ever moves an earlier one. Rotation copies nothing at all; it closes
the file and opens another, and the oldest rows retire together the day their
file does, never one at a time.
";

/// The tree: a README naming notes.txt, a twin of it in a subdirectory,
/// the same words in CMakeLists.txt, and an 800-line `.txt` no size
/// gate may count.
fn fixture() -> PathBuf {
    let doc = format!(
        "--- ce.toml\n[graph]\nentry_globs = [\"main.rs\"]\n--- README.md\n# T\n[notes](./notes.txt)\n\
         --- main.rs\nfn main() {{}}\n--- notes.txt\n{SHARED}--- copy/notes2.txt\n{SHARED}\
         --- CMakeLists.txt\n{SHARED}--- big.txt\n{}",
        "line\n".repeat(800)
    );
    common::doc_tree("docdup-text", &doc)
}

/// The gates a `.txt` never meets: the scan names no `.txt` (the
/// 800-line file fails no size gate), a 900-line write to one meets no
/// budget while the same write to a `.rs` is denied by default, and
/// the graph mints no file node for a `.txt`, so nothing dies.
fn no_other_gate_reads_it(dir: &Path, core: &str) {
    let scan = common::run_ce(dir, &["scan", ".", "--core", core]);
    let out = String::from_utf8_lossy(&scan.stdout);
    assert!(scan.status.success(), "scan: {out}");
    assert!(
        !out.contains("big.txt"),
        "the scan never names a .txt: {out}"
    );

    let big = "line\n".repeat(900);
    let env = common::pretooluse_envelope_at(dir, "more.txt", "Write", &big);
    let out = common::run_hook(dir, &["probe", "--hook"], &env);
    assert!(out.trim().is_empty(), "a .txt write meets no budget: {out}");
    common::expect_write_denied(dir, "more.rs", &big, "hard budget of 750");

    let dead = deadcode::run(dir, None, core).expect("deadcode");
    assert!(dead.degraded.is_none(), "healthy run");
    let dead_paths: Vec<&str> = dead.dead.iter().map(|d| d.path.as_str()).collect();
    assert!(dead_paths.is_empty(), "nothing dies: {dead_paths:?}");
}

/// The docdup family reads the two `.txt` files and nothing else does:
/// one pair, both sides text_para, CMake's list file on neither side;
/// no other gate reads a `.txt` (the helper above); the check seats the
/// three prose files after the two graph nodes, rides the pair once and
/// charges the docdup axis alone.
#[test]
fn plain_text_pairs_in_docdup_and_stays_out_of_every_other_gate() {
    let dir = fixture();
    let core = core_bin();
    let doc = codeeraser::faces::docdup(&dir, &core).expect("docdup");
    let dups = &doc["dups"];
    assert_eq!(
        (dups.as_array().map(Vec::len), &dups[0]["a"], &dups[0]["b"]),
        (
            Some(1),
            &"copy/notes2.txt:1-5 text_para".into(),
            &"notes.txt:1-5 text_para".into()
        ),
        "one pair, both sides plain-text paragraphs, CMakeLists.txt on neither: {dups}"
    );
    no_other_gate_reads_it(&dir, &core);

    let o = common::judged(&dir);
    assert_eq!(
        (o.files, o.sim_pairs),
        (5, 1),
        "two nodes + three prose files; the pair once"
    );
    let axis = |c: i64| o.reply.axes.iter().find(|a| a[0] == c).map(|a| a[1]);
    assert!(
        axis(3) > Some(0),
        "the docdup axis charges the twins: {:?}",
        o.reply.axes
    );
    assert_eq!(axis(2), Some(0), "no clone pair: {:?}", o.reply.axes);
}
