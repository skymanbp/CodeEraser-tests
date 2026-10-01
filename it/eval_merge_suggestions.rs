//! The clone-merge family's frozen suggestion set (plan v2.31 step 7,
//! design booklet §6.5): `contracts/eval/merge-suggestions-v<n>.json`
//! holds, per corpus, the counts, the groups not sent and one row per
//! suggestion — its members (`at` a unit or a fragment's
//! `path:start-end`, `run` the lines the core priced),
//! family, parameters, kept member, savings, feasibility and reason.
//! The CI leg measures this repository's pinned tree again and holds
//! every row; `regenerate` (`CE_BLESS=1`, `--ignored`) re-measures the
//! five corpora — the external four need their pinned checkouts under
//! `.ce-eval/corpora` — and redraws the blind sample from the new set.
//! Both write the exam generation's docs (merge generation 2, booklet
//! §13 item 50); a record generation's set is never re-measured.

use crate::eval_merge_parts::sample::{SAMPLE, SAMPLE_SCHEMA, audit_row, draw};
use crate::eval_merge_parts::{
    DOC, GENERATIONS, SCHEMA, SELF, corpora, frozen, load, measure, section, self_tree, write,
    write_lined,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The pinned tree answers its frozen rows, one by one.
#[test]
fn the_self_tree_answers_its_frozen_rows() {
    let doc = load(DOC).unwrap_or_else(|| panic!("{DOC}: the frozen set is missing"));
    assert_eq!(doc["schema"], SCHEMA);
    let want = frozen(&doc, SELF.0);
    assert_eq!(
        want["corpus"]["commit"], SELF.1,
        "the self corpus is the pinned commit"
    );
    let got = section(&measure(&self_tree("merge-eval-gate")));
    assert_eq!(got["counts"], want["counts"], "counts");
    assert_eq!(got["unsendable"], want["unsendable"], "groups not sent");
    let (got, want) = (
        got["rows"].as_array().expect("rows"),
        want["rows"].as_array().expect("rows"),
    );
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert_eq!(g, w, "row {i}");
    }
    assert_eq!(got.len(), want.len(), "row count");
}

/// One member set, one suggestion (a T1/T2 group covers a T3 group
/// over the same members): in every generation, within each corpus no
/// two rows share their members' `at` identities, so the sample's `id`
/// is unique by construction.
#[test]
fn every_member_set_is_suggested_once() {
    let sets = GENERATIONS.iter().map(|g| (g.set, load(g.set)));
    let docs: Vec<_> = sets
        .map(|(p, d)| d.unwrap_or_else(|| panic!("{p}: missing")))
        .collect();
    for c in docs
        .iter()
        .flat_map(|d| d["corpora"].as_array().expect("corpora"))
    {
        let mut seen: BTreeMap<Vec<String>, usize> = BTreeMap::new();
        for (i, row) in c["rows"].as_array().expect("rows").iter().enumerate() {
            let members = row["members"].as_array().into_iter().flatten();
            let set = members.map(|m| m["at"].to_string()).collect();
            if let Some(j) = seen.insert(set, i) {
                panic!("{}: rows {j} and {i} suggest one member set", c["name"]);
            }
        }
    }
}

/// Re-measure the five corpora, rewrite the set and redraw the sample.
#[test]
#[ignore]
fn regenerate() {
    assert!(crate::facts::blessing(), "CE_BLESS=1 regenerates");
    let mut sections = Vec::new();
    let mut audit = Vec::new();
    for (name, corpus, root) in corpora() {
        let doc = measure(&root);
        let mut s = section(&doc);
        println!(
            "{name}: counts {} unsendable {}",
            s["counts"], s["unsendable"]
        );
        s["name"] = json!(name);
        s["corpus"] = corpus;
        audit.push((name, doc, root));
        sections.push(s);
    }
    let set = json!({"schema": SCHEMA, "corpora": sections});
    write_lined(DOC, &set);
    let rows: Vec<Value> = draw(&set)
        .into_iter()
        .map(|(name, row)| {
            let (_, doc, root) = audit.iter().find(|(n, ..)| *n == name).expect("measured");
            audit_row(&name, &row, doc, root)
        })
        .collect();
    let feasible = draw(&set)
        .iter()
        .filter(|(_, r)| r["feasible"] == true)
        .count();
    println!(
        "sample: {} rows, {feasible} feasible, {} infeasible",
        rows.len(),
        rows.len() - feasible
    );
    write(
        SAMPLE,
        &json!({"schema": SAMPLE_SCHEMA, "from": DOC, "rows": rows}),
    );
}
