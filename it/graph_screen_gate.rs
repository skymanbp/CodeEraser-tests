//! The graph screen's deadcode half is the deadcode face's document
//! (plan v2.35, the dead-file amendment, phase 5C): the screen's
//! judgment rides the face's own wire (the advisory and the dead-file
//! gate) and its document request the face's own road, so on a tree with
//! a committed baseline and a declared `[deadcode] block` the `deadcode`
//! key of `ce.graph-screen` is, key for key and byte for byte, what
//! `ce deadcode --format json` prints for the same tree. The tree holds a
//! standing root (`s.ts`, unreferenced when the baseline was written), a
//! born root (`n.ts`, new since) and a dependent (`d.ts`, under `n.ts`).

use crate::common::{self, core_bin};
use serde_json::Value;

#[test]
fn the_screen_carries_the_deadcode_document_byte_for_byte() {
    let core = core_bin();
    let dir = common::gates::standing_root_under_block("graph-screen-gate");
    std::fs::write(dir.join("n.ts"), "import './d';\n").expect("n.ts");
    std::fs::write(dir.join("d.ts"), "export const d = 1;\n").expect("d.ts");
    let (_, printed, err) = common::ce_triple(
        &dir,
        &["deadcode", ".", "--format", "json", "--core", &core],
        &[],
    );
    let screen = codeeraser::faces::graph_screen(&dir, &core).expect("the screen");
    let half = &screen["deadcode"];
    let face: Value = serde_json::from_str(&printed).expect(&err);
    assert_eq!(half, &face, "key for key");
    // the deadcode family prints compact (its catalogue's `pretty` is
    // false), so the bytes are the half serialized the same way
    assert_eq!(
        serde_json::to_string(half).expect("serialize"),
        printed.trim_end(),
        "byte for byte"
    );
    let states: Vec<(&str, &str)> = face["dead"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|d| {
            (
                d["name"].as_str().unwrap_or(""),
                d["state"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        states,
        [("d.ts", "-"), ("n.ts", "born"), ("s.ts", "standing")],
        "a dependent, a born root and a standing root: {face}"
    );
    assert_eq!(face["gate"]["blocked"], 1, "{face}");
    assert_eq!(
        screen["canvas"]["schema"], "ce.graph-canvas/0.4.0",
        "the canvas half does not move"
    );
}
