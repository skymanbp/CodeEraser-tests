//! The scalar↔set bridge (design §7.3 as amended 2026-08-14, the
//! conservation form): the dedup budget counts BLOCKS, the baseline
//! set holds unit-pair MEMBERS, and the two spaces are welded by one
//! exact accounting — members + collapsed == blocks == budget — so
//! neither gate can drift silently while the §7.2 identity keeps its
//! deliberate move-stability (57 of the self repo's 97 blocks share
//! a unit pair with an earlier block; that collapse is REPORTED).
//! Plus the ADR-006 independence pair: the --fail-under floor and
//! the ratchet each fail ALONE.

use crate::baseline_ledgers::{RETIRED, rekeyed_pairs, suite_pairs};
use crate::common;
use crate::common::core_bin;
use codeeraser::score::{self, Opts};
use std::path::{Path, PathBuf};

/// The committed baseline is the CALLER's one read (O31): score::run
/// judges what it is handed and never opens the file itself.
fn opts(dir: &Path, db: Option<PathBuf>, floor: Option<u32>) -> Opts {
    Opts {
        db,
        core: core_bin(),
        days: None,
        floor,
        establish: false,
        pinned_soft: None,
        baseline: score::baseline::read(dir).expect("baseline read"),
    }
}

/// Conservation on the self repo, every leg independently sourced:
/// blocks from the dedup report, budget from ce.toml, members and
/// collapse from the score assembly — and the two gates agree.
#[test]
fn bridge_conserves_blocks_into_members_and_gates_agree() {
    let root = Path::new("..");
    let db = common::tmp("bridge-db").join("index.db");
    let o = score::run(root, opts(root, Some(db.clone()), None)).expect("check");
    let (found, _s) = codeeraser::dedup::analyze(root, Some(db), None, None).expect("dedup");
    let budget = codeeraser::config::Config::load(root)
        .expect("ce.toml")
        .dedup
        .budget
        .expect("budget");
    assert_eq!(
        o.members + o.collapsed,
        found.blocks.len(),
        "every block lands in exactly one member (new or collapsed)"
    );
    assert_eq!(found.blocks.len(), budget, "the scalar gate's own equality");
    // the two gates judge the same repo the same way
    assert!(found.blocks.len() <= budget, "dedup --check passes");
    assert!(!o.reply.fail, "ce check passes: {:?}", o.reply.added);
    assert!(o.reply.degraded.is_none(), "healthy judgment");
}

/// The `discrete` member ids of one committed baseline.
fn discrete_members(path: &str) -> std::collections::HashSet<u64> {
    let baseline: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("committed baseline"))
            .expect("baseline json");
    baseline["discrete"]
        .as_array()
        .expect("discrete")
        .iter()
        .map(|v| v.as_u64().expect("member id"))
        .collect()
}

/// The 3k corpus-generation gate (RM14, pre-registered): the frozen
/// pre-Haskell discrete member set must stay a SUBSET of every later
/// committed baseline — growth is a batch of named additions, never
/// a silent rewrite of history. Deliberate de-duplication exits
/// through the named RETIRED ledger above (its first entries landed
/// when the P3 repayment removed real pre-freeze duplication and
/// this gate's original unconditional subset form went red — the
/// gate could not tell a cleanup from a rewrite until the ledger
/// gave cleanups a spoken exit); a path move exits through REKEYED,
/// which demands the successor key be present, so subset strength
/// is conserved modulo the named rename.
#[test]
fn pre_haskell_members_survive_every_generation() {
    let frozen: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("../contracts/eval/pre-haskell-members-v1.json").expect("frozen"),
    )
    .expect("frozen json");
    let now = discrete_members("../ce-baseline.json");
    // the suite's own baseline (plan v2.18 step #12): the members that
    // moved there are checked under their suite keys, never merged
    // into `now` — a key must be found in the ledger that names it
    let suite = discrete_members("../cli/tests/ce-baseline.json");
    let old: Vec<u64> = frozen["members"]
        .as_array()
        .expect("members")
        .iter()
        .map(|v| v.as_u64().expect("member id"))
        .collect();
    assert_eq!(old.len(), 40, "the frozen set is the 3j-close 40");
    let suite_ledger = suite_pairs();
    for m in &old {
        if let Some((_, why)) = RETIRED.iter().find(|(id, _)| id == m) {
            assert!(
                !now.contains(m),
                "retired member {m} is back in the baseline — stale RETIRED entry ({why})"
            );
            continue;
        }
        if let Some((_, new)) = rekeyed_pairs().iter().find(|(id, _)| id == m) {
            assert!(
                !now.contains(m),
                "re-keyed member {m} is back under its pre-move key — stale REKEYED entry"
            );
            let Some((_, sk)) = suite_ledger.iter().find(|(id, _)| id == new) else {
                assert!(
                    now.contains(new),
                    "re-keyed member {m}'s successor {new} is missing — the rename ledger \
                     promised the duplication survived the move"
                );
                continue;
            };
            assert!(
                !now.contains(new),
                "member {new} moved to the suite yet is back in the superproject's baseline — stale REKEYED_SUITE entry"
            );
            assert!(
                suite.contains(sk),
                "member {new}'s suite key {sk} is missing from the suite's baseline — the second-generation ledger promised the duplication survived the move"
            );
            continue;
        }
        assert!(
            now.contains(m),
            "pre-Haskell member {m} vanished from the committed baseline without a \
             named retirement — corpus growth must never rewrite the pre-generation set"
        );
    }
}

/// ADR-006: floor and ratchet each fail ALONE — the floor trips with
/// an empty added set, the ratchet trips with no floor on the wire.
#[test]
fn floor_and_ratchet_fail_independently() {
    let dir = common::tmp("bridge-both");
    common::seed_clone_pair(&dir);
    let est = score::run(&dir, opts(&dir, None, None)).expect("establish");
    assert!(!est.reply.fail, "no baseline, no floor: nothing fails");
    score::baseline::write(&dir, &est.reply.new_baseline).expect("write");

    // direction 1: floor alone (the ratchet half is clean)
    let floored = score::run(&dir, opts(&dir, None, Some(1000))).expect("floored");
    assert!(floored.reply.fail, "the floor alone must fail");
    // and the report SAYS which floor it judged under — a pass with
    // none armed is a weaker claim than a pass with one, and the two
    // faces of this gate disagreed for exactly that reason (K round
    // step 6: CI armed 950, the GUI could arm nothing)
    assert_eq!(
        codeeraser::score::report_json(&floored)["floor"],
        serde_json::json!(1000)
    );
    assert_eq!(
        codeeraser::score::report_json(&est)["floor"],
        serde_json::Value::Null,
        "absent floor echoes null, never a fabricated 0"
    );
    assert!(
        floored.reply.added.is_empty() && floored.reply.over.is_empty(),
        "ratchet half stayed clean: {:?}",
        floored.reply.added
    );
    assert!(floored.reply.score < 1000, "the clone pair costs score");

    // direction 2: ratchet alone (no floor on the wire)
    std::fs::write(dir.join("c.rs"), common::rust_fn(3)).expect("c.rs");
    let grown = score::run(&dir, opts(&dir, None, None)).expect("grown");
    assert!(grown.reply.fail, "a new discrete member alone must fail");
    assert!(
        !grown.reply.added.is_empty(),
        "the new clone is a new member"
    );
}
