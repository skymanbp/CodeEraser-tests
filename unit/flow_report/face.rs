use super::*;
use crate::flow::wire::Finding;
use crate::scan::lang::Lang;

const PY: &str = "def gone():\n    return 1\n    print(\"never\")\n\n\ndef overwritten():\n    x = 1\n    x = 2\n    return x\n\n\ndef ignores(a):\n    return 3\n";

fn at(u: usize, kind: u8, seq: i64, v: i64) -> Finding {
    Finding {
        u,
        kind,
        seq,
        v,
        seq_end: seq,
    }
}

/// Two copies of the fixture under two paths, and a verdict the core
/// could have given, delivered out of order: (file, nth, finding).
fn judged_pair() -> (Vec<String>, Vec<Lowered>, Verdict) {
    let paths = vec!["b.py".to_string(), "a.py".to_string()];
    let files: Vec<Lowered> = paths
        .iter()
        .map(|_| lower_file(PY, Lang::Python).expect("lowers"))
        .collect();
    let verdict = Verdict {
        findings: vec![
            (0, 2, at(2, 3, -1, 0)),
            (1, 1, at(1, 1, 0, 0)),
            (0, 0, at(0, 0, 1, -1)),
            (1, 0, at(0, 0, 1, -1)),
        ],
        refused: vec![(1, 2, "stmt 0: parent after child".into())],
        dynamic_units: 0,
    };
    (paths, files, verdict)
}

fn keys(r: &Report) -> Vec<(String, String, &'static str, u32)> {
    r.findings
        .iter()
        .map(|f| (f.path.clone(), f.unit.clone(), f.kind, f.line))
        .collect()
}

/// The findings are placed and ordered by (path, unit, kind, line),
/// whatever order the verdict came in; the counts hold the tables and
/// the verdict whole; a refusal names its unit.
#[test]
fn findings_are_placed_and_ordered_by_path_unit_kind_line() {
    let (paths, files, v) = judged_pair();
    let r = assemble(&paths, &files, Ok(v), None);
    assert_eq!(
        keys(&r),
        [
            ("a.py".into(), "gone".into(), "unreachable", 3),
            ("a.py".into(), "overwritten".into(), "dead_store", 7),
            ("b.py".into(), "gone".into(), "unreachable", 3),
            ("b.py".into(), "ignores".into(), "unused_param", 12),
        ]
    );
    assert_eq!(
        (r.counts["units"], r.counts["findings"], r.counts["shown"]),
        (6, 4, 4)
    );
    assert_eq!(r.counts["refused"], 1);
    assert_eq!(
        (r.refused[0].path.as_str(), r.refused[0].unit.as_str()),
        ("a.py", "ignores")
    );
    let advisory = r
        .findings
        .iter()
        .find(|f| f.kind == "unused_param")
        .expect("kind 3");
    assert!(!advisory.judged, "an unused parameter is never judged");
    assert!(r.degraded.is_none());
}

/// `--kind` narrows the listing and never the counts; an unknown name
/// is refused by name.
#[test]
fn the_kind_filter_shapes_the_listing_and_not_the_counts() {
    let (paths, files, v) = judged_pair();
    let shown = shown_kinds(&["unreachable".into()]).expect("a kind");
    let r = assemble(&paths, &files, Ok(v), shown.as_ref());
    assert_eq!(r.findings.len(), 2);
    assert!(r.findings.iter().all(|f| f.kind == "unreachable"));
    assert_eq!((r.counts["findings"], r.counts["shown"]), (4, 2));
    let both = shown_kinds(&["dead_store,unused_local".into()]).expect("two kinds");
    assert_eq!(both, Some([1u8, 2].into_iter().collect()));
    assert_eq!(shown_kinds(&[]).expect("none"), None);
    let err = shown_kinds(&["dead".into()])
        .expect_err("unknown")
        .to_string();
    assert!(err.contains("unknown kind \"dead\""), "{err}");
}

/// A degraded judgment carries its reason and no finding; the tables'
/// counts still say what was lowered.
#[test]
fn a_degraded_judgment_names_its_reason_and_finds_nothing() {
    let (paths, files, _) = judged_pair();
    let r = assemble(&paths, &files, Err("core offers no flow/1".into()), None);
    assert_eq!(r.degraded.as_deref(), Some("core offers no flow/1"));
    assert_eq!((r.counts["units"], r.counts["findings"]), (6, 0));
    let doc = report_json(&r);
    assert_eq!(doc["schema"], SCHEMA_ID);
    assert_eq!(doc["findings"], serde_json::json!([]));
}
