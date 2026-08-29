//! O32 (6.4.0): the reply self-consistency table. Every invariant is
//! computable from a request and a reply this side constructs, so
//! each leg mutates one fact of a passing pair and names the refusal
//! it expects — no core, no disk, and no fake core: a spy core would
//! have to re-implement the echo policy it is meant to catch drifting.
use super::*;
use crate::score::wire::{Reply, Request};
use serde_json::json;

/// The dedup-only request, then one mutation.
fn req(f: impl FnOnce(&mut Request)) -> Request {
    let mut r = Request::dedup_only(0, 0, Vec::new(), None);
    f(&mut r);
    r
}

/// The reply a shipped-default core gives that request — nothing
/// held, no digest, the default weights echoed whole — then one
/// mutation.
fn rep(f: impl FnOnce(&mut Reply)) -> Reply {
    let mut r = Reply {
        candidates: Vec::new(),
        join_severity: Vec::new(),
        score: 1000,
        axes: Vec::new(),
        added: Vec::new(),
        removed: Vec::new(),
        over: Vec::new(),
        tolerance_drawn: Vec::new(),
        fail: false,
        failed: Vec::new(),
        dropped: None,
        new_baseline: json!({"continuous": [], "discrete": []}),
        knobs: [("defaultWeight".to_string(), 1)].into_iter().collect(),
        weights: (0..7).map(|a| [a, 1]).collect(),
        class_knobs: Vec::new(),
        dedup_blocks: None,
        degraded: None,
    };
    f(&mut r);
    r
}

/// A reply that holds `names` (fail is their disjunction).
fn held(names: &[&str]) -> Reply {
    rep(|r| {
        r.fail = !names.is_empty();
        r.failed = names.iter().map(|n| n.to_string()).collect();
    })
}

fn committed(digest: Option<u64>, soft: Option<u64>) -> serde_json::Value {
    let mut b = json!({"continuous": [], "discrete": []});
    if let Some(d) = digest {
        b["knobsDigest"] = json!(d);
    }
    if let Some(s) = soft {
        b["softLine"] = json!(s);
    }
    b
}

/// (request, reply, the phrase the refusal must carry; "" = holds).
fn table(cases: &[(Request, Reply, &str)]) {
    for (i, (r, reply, phrase)) in cases.iter().enumerate() {
        match (check_reply(r, reply), phrase.is_empty()) {
            (Ok(()), true) => {}
            (Err(e), false) => assert!(
                e.to_string().contains(phrase),
                "case {i}: want {phrase:?} in {e:?}"
            ),
            (Ok(()), false) => panic!("case {i}: passed, wanted {phrase:?}"),
            (Err(e), true) => panic!("case {i}: refused, wanted a pass: {e:?}"),
        }
    }
}

#[test]
fn every_reply_law_is_named() {
    table(&[
        (req(|_| {}), rep(|_| {}), ""),
        // (1) fail is the disjunction of the names
        (
            req(|_| {}),
            rep(|r| r.fail = true),
            "disjunction of the names",
        ),
        (
            req(|_| {}),
            rep(|r| r.failed = vec!["floor".into()]),
            "disjunction of the names",
        ),
        // (2) a degraded reply names itself
        (
            req(|_| {}),
            rep(|r| {
                r.degraded = Some("too_large".into());
                r.fail = true;
                r.failed = vec!["floor".into()];
            }),
            "without naming degraded",
        ),
        // (4) the digest echoes, absent exactly when none was sent
        (
            req(|r| r.knobs_digest = Some(5)),
            rep(|_| {}),
            "absent must mean none sent",
        ),
        (
            req(|_| {}),
            rep(|r| r.new_baseline = committed(Some(5), None)),
            "absent must mean none sent",
        ),
        (
            req(|r| r.knobs_digest = Some(5)),
            rep(|r| r.new_baseline = committed(Some(5), None)),
            "",
        ),
        // (7) dropped answers present, both ways
        (
            req(|r| r.present = Some(Vec::new())),
            rep(|_| {}),
            "pre-6.4.0 core",
        ),
        (
            req(|_| {}),
            rep(|r| r.dropped = Some(Vec::new())),
            "sent no present table",
        ),
        (
            req(|r| r.present = Some(Vec::new())),
            rep(|r| r.dropped = Some(Vec::new())),
            "",
        ),
    ]);
}

#[test]
fn the_fence_holds_exactly_on_a_committed_digest_that_differs() {
    let drift = |r: &mut Request| {
        r.baseline = committed(Some(1), None);
        r.knobs_digest = Some(2);
    };
    let echo2 = |r: &mut Reply| r.new_baseline = committed(Some(2), None);
    let fenced = || {
        rep(|r| {
            echo2(r);
            r.fail = true;
            r.failed = vec!["knobs_digest".into()];
        })
    };
    table(&[
        // a committed 1 against a declared 2: the core must have held
        (req(drift), rep(echo2), "did not hold knobs_digest"),
        (req(drift), fenced(), ""),
        // the same digest on both sides: holding it is the lie
        (
            req(|r| {
                r.baseline = committed(Some(2), None);
                r.knobs_digest = Some(2);
            }),
            fenced(),
            "held knobs_digest",
        ),
        // establish (null baseline) never drifts, whatever was declared
        (
            req(|r| r.knobs_digest = Some(2)),
            fenced(),
            "held knobs_digest",
        ),
        // an object baseline without the key recorded None: None == None
        (req(|r| r.baseline = committed(None, None)), rep(|_| {}), ""),
    ]);
}

#[test]
fn new_baseline_is_the_document_the_writer_persists() {
    let derived = |r: &mut Reply| r.new_baseline = committed(None, Some(1));
    table(&[
        (
            req(|_| {}),
            rep(|r| r.new_baseline = json!([])),
            "not a baseline document",
        ),
        // establish: a derivable LOC set must yield a line, an empty
        // or all-zero one must not
        (
            req(|r| r.judged_loc = vec![0, 120]),
            rep(|_| {}),
            "derived softLine",
        ),
        (req(|r| r.judged_loc = vec![0, 120]), rep(derived), ""),
        (req(|_| {}), rep(derived), "derived softLine"),
        (req(|r| r.judged_loc = vec![0, 0]), rep(|_| {}), ""),
        // a committed line is carried verbatim
        (
            req(|r| r.baseline = committed(None, Some(300))),
            rep(derived),
            "committed baseline carries",
        ),
        (
            req(|r| r.baseline = committed(None, Some(300))),
            rep(|r| r.new_baseline = committed(None, Some(300))),
            "",
        ),
    ]);
}

#[test]
fn knob_echoes_are_asked_of_judged_replies_only() {
    let floor = |r: &mut Request| r.thresholds = vec![[7, 1]];
    let echo = |v: i64| {
        move |r: &mut Reply| {
            r.knobs.insert("cycleFloor".into(), v);
        }
    };
    // a degraded reply never judged: the echo is not asked, but the
    // laws every reply must hold still are
    let degraded = || {
        rep(|r| {
            r.degraded = Some("too_large".into());
            r.fail = true;
            r.failed = vec!["degraded".into()];
            r.knobs.clear();
        })
    };
    table(&[
        (req(floor), rep(|_| {}), "reply echo missing cycleFloor"),
        (req(floor), rep(echo(1)), ""),
        (req(floor), rep(echo(2)), "core judged with cycleFloor=2"),
        (req(floor), degraded(), ""),
        (
            req(|r| r.present = Some(vec![3])),
            degraded(),
            "pre-6.4.0 core",
        ),
        (req(|_| {}), held(&["knobs_digest"]), "held knobs_digest"),
    ]);
}
