// The flow document over a verdict (plan v2.32 step 3: the request this
// side sends, laid out by the real core and bound back): placement,
// order, counts, the `--kind` filter and the degraded road.
use super::*;
use crate::flow::wire::Finding;
use crate::scan::lang::Lang;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// The fields of the bound document these legs read (the console that
/// once read them is the core's since plan v2.32 step 5).
#[derive(Deserialize)]
struct FindingFace {
    path: String,
    unit: String,
    kind: String,
    line: u32,
    judged: bool,
}

#[derive(Deserialize)]
struct RefusedFace {
    path: String,
    unit: String,
}

#[derive(Deserialize)]
struct Report {
    counts: BTreeMap<String, u64>,
    findings: Vec<FindingFace>,
    refused: Vec<RefusedFace>,
    degraded: Option<String>,
}

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

/// The request laid out by the core this test process measures with,
/// bound and read.
fn laid_out(
    paths: &[String],
    files: &[Lowered],
    judgment: Result<Verdict, String>,
    shown: Option<&[(String, i64)]>,
) -> (Value, Report) {
    let req = request((paths, files), judgment, shown);
    let core = crate::daemon::judge::core_bin().expect("a core");
    let doc = document::assemble(&core, req).expect("laid out").document;
    let r = Report::deserialize(&doc).expect("read");
    (doc, r)
}

fn keys(r: &Report) -> Vec<(&str, &str, &str, u32)> {
    r.findings
        .iter()
        .map(|f| (f.path.as_str(), f.unit.as_str(), f.kind.as_str(), f.line))
        .collect()
}

/// The findings are placed and ordered by (path, unit, kind, line),
/// whatever order the verdict came in; the counts hold the tables and
/// the verdict whole; a refusal names its unit.
#[test]
fn findings_are_placed_and_ordered_by_path_unit_kind_line() {
    let (paths, files, v) = judged_pair();
    let (_, r) = laid_out(&paths, &files, Ok(v), None);
    assert_eq!(
        keys(&r),
        [
            ("a.py", "gone", "unreachable", 3),
            ("a.py", "overwritten", "dead_store", 7),
            ("b.py", "gone", "unreachable", 3),
            ("b.py", "ignores", "unused_param", 12),
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

/// `--kind` narrows the listing and never the counts; the names are
/// read against the package's catalogue, each once, in the order given.
#[test]
fn the_kind_filter_shapes_the_listing_and_not_the_counts() {
    let (paths, files, v) = judged_pair();
    let shown = shown_kinds(&["unreachable".into()]);
    assert_eq!(shown, Some(vec![("unreachable".to_string(), 0)]));
    let (_, r) = laid_out(&paths, &files, Ok(v), shown.as_deref());
    assert_eq!(r.findings.len(), 2);
    assert!(r.findings.iter().all(|f| f.kind == "unreachable"));
    assert_eq!((r.counts["findings"], r.counts["shown"]), (4, 2));
    let both = shown_kinds(&["unused_local,dead_store".into(), "unused_local".into()]);
    let codes: Vec<i64> = both.expect("two kinds").iter().map(|(_, c)| *c).collect();
    assert_eq!(codes, [2, 1]);
    assert_eq!(shown_kinds(&[]), None);
}

/// A name the catalogue does not list goes as −1 and the core refuses
/// it, the refusal named by the kind as given and the core's list.
#[test]
fn an_unknown_kind_is_the_cores_refusal_named_by_the_name_given() {
    let (paths, files, v) = judged_pair();
    let shown = shown_kinds(&["unreachable,dead".into()]);
    let codes: Vec<i64> = shown.iter().flatten().map(|(_, c)| *c).collect();
    assert_eq!(codes, [0, -1]);
    let req = request((&paths, &files), Ok(v), shown.as_deref());
    let core = crate::daemon::judge::core_bin().expect("a core");
    let err = document::assemble(&core, req).expect_err("refused");
    assert_eq!(
        named_kind(err, shown.as_deref()).to_string(),
        "flow document: unknown kind \"dead\"; the catalogue lists unreachable, dead_store, unused_local, unused_param"
    );
}

/// A degraded judgment carries its reason and no finding; the tables'
/// counts still say what was lowered.
#[test]
fn a_degraded_judgment_names_its_reason_and_finds_nothing() {
    let (paths, files, _) = judged_pair();
    let (doc, r) = laid_out(&paths, &files, Err("core offers no flow/1".into()), None);
    assert_eq!(r.degraded.as_deref(), Some("core offers no flow/1"));
    assert_eq!((r.counts["units"], r.counts["findings"]), (6, 0));
    assert_eq!(doc["schema"], "ce.flow-report/0.1.0");
    assert_eq!(doc["findings"], serde_json::json!([]));
}
