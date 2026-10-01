//! The flow/1 leg's own rules (plan v2.31 step 4 A3): the body's four
//! tables led by the request's unit index, the greedy split and the
//! over-cap unit refused by name beside it, every strict check consume
//! holds a reply to (a degraded one is cap-mirror drift), and the
//! refusal text read back to its table and row.

use super::*;
use crate::flow::lower::Legend;
use crate::flow::wire_batch::refusal;
use crate::scan::lang::Lang;
use serde_json::json;

/// A unit with `stmts` plain statements, the variables given as
/// `(declSeq, flags)`, and one read of each variable at statement 0.
fn unit(nth: usize, stmts: usize, vars: &[(i64, i64)], dynamic: bool) -> Unit {
    let flag = if dynamic { 8 } else { 0 };
    Unit {
        nth,
        name: format!("u{nth}"),
        start_line: 1,
        end_line: 2,
        params: vars.iter().filter(|(_, f)| f & 1 == 1).count() as u32,
        dynamic,
        stmts: (0..stmts as i64).map(|s| [s, -1, 1, flag, 0]).collect(),
        vars: (0..).zip(vars).map(|(v, &(d, f))| [v, d, f]).collect(),
        uses: (0..vars.len() as i64).map(|v| [0, v, 0]).collect(),
        legend: Legend {
            stmt_at: Vec::new(),
            stmt_end: Vec::new(),
            stmt_text: Vec::new(),
            var_name: Vec::new(),
            var_at: Vec::new(),
        },
    }
}

fn file(lang: Lang, units: Vec<Unit>) -> Lowered {
    Lowered {
        lang,
        units,
        unlowered: Vec::new(),
    }
}

/// Two files: a Go unit (two statements, a parameter and a local) and
/// a dynamic Python unit (one statement, no variable).
fn two_files() -> Vec<Lowered> {
    vec![
        file(Lang::Go, vec![unit(0, 2, &[(-1, 1), (1, 0)], false)]),
        file(Lang::Python, vec![unit(3, 1, &[], true)]),
    ]
}

#[test]
fn the_body_leads_every_row_with_its_units_request_index() {
    let files = two_files();
    let sent = &plan(&files).0[0];
    assert_eq!(
        body(sent),
        json!({
            "units": [[0, 4, 1], [1, 0, 0]],
            "stmts": [[0, 0, -1, 1, 0, 0], [0, 1, -1, 1, 0, 0], [1, 0, -1, 1, 8, 0]],
            "vars": [[0, 0, -1, 1], [0, 1, 1, 0]],
            "uses": [[0, 0, 0, 0], [0, 0, 1, 0]],
        })
    );
    assert_eq!(sizes(sent), [2, 3, 2, 2]);
}

#[test]
fn the_plan_is_greedy_under_the_cap_and_refuses_a_unit_heavier_than_it() {
    let half = ROW_CAP / 2;
    let files = vec![
        file(
            Lang::Rust,
            vec![
                unit(0, half - 1, &[], false),
                unit(1, half - 1, &[], false),
                unit(2, 5, &[], false),
            ],
        ),
        file(Lang::C, vec![unit(7, ROW_CAP, &[], false)]),
        file(Lang::C, vec![unit(8, 3, &[], false)]),
    ];
    let (sent, refused) = plan(&files);
    let batches: Vec<Vec<usize>> = sent
        .iter()
        .map(|s| s.units.iter().map(|(u, _)| u.nth).collect())
        .collect();
    assert_eq!(batches, [vec![0, 1], vec![2, 8]]);
    let [(file, nth, reason)] = &refused[..] else {
        panic!("one unit over the cap, refused: {refused:?}")
    };
    assert_eq!((*file, *nth), (1, 7));
    assert!(
        reason.starts_with("weighs 524289 rows against the cap of 524288"),
        "{reason}"
    );
    let (none, nothing) = plan(&[]);
    assert!(none.is_empty() && nothing.is_empty());
}

/// A healthy reply to two_files(): a dead store on the local, the
/// unused parameter, one dynamic unit.
fn healthy() -> Value {
    json!({
        "findings": [[0, 1, 1, 1, 1], [0, 3, -1, 0, -1]],
        "counts": {"units": 2, "stmts": 3, "vars": 2, "uses": 2, "findings": 2, "dynamicUnits": 1},
        "degraded": false,
    })
}

#[test]
fn consume_types_a_healthy_reply_and_refuses_a_degraded_one_as_drift() {
    let files = two_files();
    let sent = &plan(&files).0[0];
    let j = consume(&healthy(), sent).unwrap();
    let dead = Finding {
        u: 0,
        kind: 1,
        seq: 1,
        v: 1,
        seq_end: 1,
    };
    let param = Finding {
        u: 0,
        kind: 3,
        seq: -1,
        v: 0,
        seq_end: -1,
    };
    assert_eq!(j.findings, vec![dead, param]);
    assert_eq!(j.counts["dynamicUnits"], 1);
    let mut degraded = healthy();
    degraded["degraded"] = json!(true);
    degraded["reason"] = json!("flow_too_large");
    degraded["findings"] = json!("never read");
    let Err(e) = consume(&degraded, sent) else {
        panic!("a degraded reply to a priced batch consumed")
    };
    assert!(e.contains("cap mirror drift"), "{e}");
}

/// Each strict check, one number moved: `(pointer, value)` against
/// healthy().
const SKEWED: &str = "\
/counts/units 3
/counts/stmts 2
/counts/vars 1
/counts/uses 3
/counts/findings 1
/counts/dynamicUnits 0
/findings/0/0 2
/findings/1/1 4
/findings/0/2 2
/findings/0/3 2
/findings/0/4 0
/findings/1/2 0
/findings/1/3 1
/findings/1 [0,1,1,1,1]
/findings/0 [0,0,0,0,1]
/findings/0 [0,0,1,-1,0]
/findings/0 [0,0,0,-1,2]
/findings/0 [0,2,0,1,0]
/findings/0 [0,1,1,1]";

#[test]
fn consume_refuses_every_skew_by_name() {
    let files = two_files();
    let sent = &plan(&files).0[0];
    for line in SKEWED.lines() {
        let (at, value) = line.split_once(' ').unwrap();
        let mut reply = healthy();
        *reply.pointer_mut(at).expect(at) = serde_json::from_str(value).unwrap();
        match consume(&reply, sent) {
            Err(e) => assert!(e.contains("wire skew"), "{line}: {e}"),
            Ok(_) => panic!("{line}: a skewed reply consumed"),
        }
    }
}

#[test]
fn a_refusal_reads_back_to_its_table_row_and_reason() {
    assert_eq!(
        refusal("stmt 12: case outside a switch"),
        Some(("stmt", 12, "case outside a switch"))
    );
    assert_eq!(
        refusal("unit 3: params disagree with the var table"),
        Some(("unit", 3, "params disagree with the var table"))
    );
    assert_eq!(refusal("flow: Error in $: not an object"), None);
}

/// The ten languages whose precision docs read judged (plan v2.31
/// step 4 commit G: the docs regenerated on the lowering commit E
/// fixed), bit by bit at their `Lang` codes, read through the
/// package's `flow_judged` column (plan v2.32 step 2) and pinned to the
/// number the core's battery pins; eval_flow_precision.rs pins each bit
/// to its doc on disk.
#[test]
fn the_judged_mask_holds_the_ten_judged_languages() {
    use crate::scan::lang::Lang;
    let want = [
        Lang::Python,
        Lang::TypeScript,
        Lang::Tsx,
        Lang::Rust,
        Lang::Go,
        Lang::C,
        Lang::Cpp,
        Lang::Java,
        Lang::Lua,
        Lang::R,
    ]
    .iter()
    .fold(0i64, |m, &l| m | 1 << l as i64);
    assert_eq!((crate::flow::judged_mask(), want), (1540127, 1540127));
}
