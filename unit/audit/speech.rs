use super::*;
use crate::dedup::pairs::Block;
use crate::tombstone::{Judged, Kind, Row};

fn block(a: &str, b: &str) -> Block {
    Block {
        a_file: a.into(),
        a_start: 1,
        a_end: 9,
        b_file: b.into(),
        b_start: 20,
        b_end: 28,
        tokens: 60,
        distinct: 20,
    }
}

fn leg(tier: &str, judged: Result<Judged, String>, unread: usize) -> Leg {
    let row = Row {
        file: "README.md".into(),
        line: 3,
        kind: Kind::Prose,
        marks: 1,
        names: 1,
        name: String::new(),
        excerpt: String::new(),
        ledger: 0,
    };
    Leg {
        feed: serde_json::Value::Null,
        judged,
        tier: tier.into(),
        budget: Some(0),
        shown: vec![row],
        erased: 1,
        unread,
        bounded: 0,
    }
}

fn over() -> Result<Judged, String> {
    Ok(Judged {
        sites: vec![0],
        label: 0,
        prose: 1,
        over: true,
    })
}

#[test]
fn the_request_carries_the_verdicts_as_rows_and_the_paths_as_references() {
    let v = Verdict {
        fail: true,
        dups: 5,
        shown: vec![block("a.rs", "b.rs")],
    };
    let t = leg("deny", over(), 0);
    let said = Said {
        net: -3,
        dups: Some(&v),
        tomb: Some(&t),
        changed: 2,
        mode: "deny",
        git: true,
        ..Said::bare(Face::Precommit)
    };
    assert!(said.blocked());
    let mut body = body(&said);
    body["lang"] = serde_json::json!(0);
    let want = serde_json::json!({
        "family": "audit", "degraded": null, "lang": 0,
        "ranges": {"blocks": 1, "places": 1, "errors": 0},
        "facts": {"face": 1, "git": 1, "unreadable": 0, "mounted": 0, "mode": 3, "changed": 2},
        "rows": {
            "net": [[-3]], "dups": [[5, 1]], "blocks": [[0, 1, 9, 20, 28, 60]],
            "tomb": [[1, 1, 0, 1, 1, 0, 3, 1, 0, 0]], "places": [[0, 3, 2]],
        },
        "strings": {
            "block_a": ["a.rs"], "block_b": ["b.rs"], "place_file": ["README.md"], "error": [],
        },
    });
    assert_eq!(body, want);
}

#[test]
fn a_degraded_leg_is_state_two_and_names_its_error() {
    let t = leg("observe", Err("core unavailable".into()), 2);
    assert_eq!(tomb_row(&t, Some(0)), [2, 0, 0, 0, 1, 0, 0, 0, 2, 0]);
    let said = Said {
        tomb: Some(&t),
        git: true,
        ..Said::bare(Face::Stop)
    };
    let body = body(&said);
    assert_eq!(
        body["strings"]["error"],
        serde_json::json!(["core unavailable"])
    );
    assert_eq!(body["ranges"]["errors"], 1);
}

#[test]
fn an_incomplete_measurement_never_blocks_by_the_local_rule() {
    let whole = leg("deny", over(), 0);
    let partial = leg("deny", over(), 1);
    let at = |t| Said {
        tomb: Some(t),
        git: true,
        ..Said::bare(Face::Stop)
    };
    assert_eq!(
        (at(&whole).blocked(), at(&partial).blocked()),
        (true, false)
    );
}

#[test]
fn the_fallback_says_what_blocked_under_the_mount_and_where_it_goes() {
    let t = leg("deny", over(), 0);
    let stop = Said {
        tomb: Some(&t),
        git: true,
        mount: Some("sub"),
        ..Said::bare(Face::Stop)
    };
    let unreadable = Said {
        unreadable: Some("MSG".into()),
        ..Said::bare(Face::Commitmsg)
    };
    let got = [&stop, &unreadable].map(|s| fallback(s, "no core"));
    let want = [
        Line {
            stream: Stream::Out,
            text: "sub: ce audit: rule tombstone fired; the core could not phrase the verdict: \
                   no core"
                .into(),
        },
        Line {
            stream: Stream::Err,
            text: "ce commitmsg: the core could not phrase the verdict: no core".into(),
        },
    ];
    assert_eq!(got, want);
}
