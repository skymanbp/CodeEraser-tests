//! Three-face parity for the hooks (plan v2.33 step 10): a hook row
//! claims, per CLI entry, the guard classes its legs judge — the write
//! guard (`[guard] mode`), flow (`[flow] tier`), tombstone
//! (`[tombstone] tier`) — and the gate derives the legs from the code
//! the entry runs: a class's leg is the call that judges it, found on a
//! code line (comments do not count) of the entry's file, and an entry
//! that hands its work to another entry's body inherits that entry's
//! legs. Dropping a class leg from a hook turns this red; so does a leg
//! the row does not claim. The classes' tiers ride every face's doctor
//! (`ce.doctor-report` `classes`), checked here too.

use crate::face_parity_sources::src;
use crate::face_parity_table::rows;
use std::collections::BTreeSet;

/// `<entry> | <file> | <class>=<the call that judges it>, …`: the
/// legs each hook entry runs itself.
const LEGS: &str = "
probe | cli/src/guard.rs | guard=budget::size_classes(, flow=flow::observe(, tombstone=tombstone::observe(
audit | cli/src/audit.rs | guard=verdict::judge(, flow=flow::leg(, tombstone=tombstone::leg(
";

/// `<entry> | <file> | <the call into the other body> | <that entry>`:
/// the entries that run another entry's body (the git-hook faces run
/// the audit's gather; commit-msg is pre-commit with one more surface).
const CHAINS: &str = "
precommit | cli/src/audit/precommit.rs | gather( | audit
commitmsg | cli/src/audit/commitmsg.rs | precommit::run( | precommit
";

/// Whether `call` sits on a code line of `file`.
fn calls(file: &str, call: &str) -> bool {
    src(file)
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .any(|l| l.contains(call))
}

fn cells(table: &str) -> impl Iterator<Item = Vec<&str>> {
    table
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(" | ").map(str::trim).collect())
}

/// The classes an entry's legs judge, as the code runs them.
pub fn legs(entry: &str) -> BTreeSet<String> {
    if let Some(c) = cells(CHAINS).find(|c| c[0] == entry) {
        assert!(
            calls(c[1], c[2]),
            "{entry}: {} no longer calls `{}`",
            c[1],
            c[2]
        );
        return legs(c[3]);
    }
    let row = cells(LEGS)
        .find(|c| c[0] == entry)
        .unwrap_or_else(|| panic!("{entry}: a hook entry LEGS does not know"));
    row[2]
        .split(", ")
        .filter_map(|leg| leg.split_once('='))
        .filter(|(_, call)| calls(row[1], call))
        .map(|(class, _)| class.to_string())
        .collect()
}

/// The legs the table claims and the legs the code runs, per entry
/// that names any: every mismatch named.
pub fn leg_mismatches() -> Vec<String> {
    let mut out = Vec::new();
    for item in rows().iter().flat_map(|r| r.cli.clone()) {
        let Some((claim, classes)) = item.split_once(" [") else {
            continue;
        };
        let entry = claim.split(' ').next().expect("entry");
        let claimed: BTreeSet<String> = classes
            .trim_end_matches(']')
            .split('+')
            .map(String::from)
            .collect();
        let run = legs(entry);
        for c in claimed.symmetric_difference(&run) {
            let side = if run.contains(c) {
                "runs, unclaimed"
            } else {
                "claimed, not run"
            };
            out.push(format!("`ce {claim}`: the {c} leg {side}"));
        }
    }
    out
}

#[test]
fn every_hook_entry_runs_the_class_legs_its_row_claims() {
    crate::face_parity_options::none(leg_mismatches());
    // every entry LEGS and CHAINS know is claimed with its legs
    let claimed: BTreeSet<String> = rows()
        .iter()
        .flat_map(|r| r.cli.clone())
        .filter_map(|i| {
            i.split_once(" [")
                .map(|(c, _)| c.split(' ').next().map(String::from))
        })
        .flatten()
        .collect();
    for c in cells(LEGS).chain(cells(CHAINS)) {
        assert!(
            claimed.contains(c[0]),
            "{}: its legs are claimed by no row",
            c[0]
        );
    }
}

/// The doctor document lists every class a hook leg judges, with the
/// key the hooks read — the one face the classes have.
#[test]
fn the_doctor_lists_every_class_a_hook_judges() {
    let dir = crate::common::tmp("parity-classes");
    let d = codeeraser::faces::doctor(&dir, &crate::common::core_bin());
    let listed: BTreeSet<&str> = crate::common::listed(&d, "classes", "class")
        .into_iter()
        .collect();
    let judged: BTreeSet<String> = ["probe", "audit"].iter().flat_map(|e| legs(e)).collect();
    let judged: BTreeSet<&str> = judged.iter().map(String::as_str).collect();
    assert_eq!(listed, judged);
}
