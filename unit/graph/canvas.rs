use super::super::deadcode::{GraphWire, Judged};
use super::super::wire::{GRAN_FILE, GRAN_PACKAGE, GRAN_SECTION};
use super::*;
use serde_json::json;

/// Two files, one section of a.rs, one package; b.rs judged dead.
fn fixture() -> (GraphWire, Judged) {
    // the canvas draws the graph, it never reads the export surface
    // or the advisory — the shared wire leaves both unasked
    let w =
        crate::testutil::four_node_wire(&[[2, 1, 0, 0], [0, 1, 5, 0], [3, 0, 0, 0], [2, 0, 0, 0]]);
    let judged = Judged {
        dead: vec![vec![1, 1, 2]],
        reported: vec![],
        kept: Some(3),
        degraded: None,
        fail: true,
        advisory: None,
    };
    (w, judged)
}

/// This side's half (plan v2.32 step 4): every node's canvas row is
/// the one its path names — a section stands for its file, a package
/// for none — and the reply's cycles go member by member.
#[test]
fn graph_rows_name_the_file_and_cycles_flatten() {
    let (w, _) = fixture();
    assert_eq!(
        graph_rows(&w),
        vec![
            [0, GRAN_FILE, 0],
            [1, GRAN_FILE, 1],
            [2, GRAN_SECTION, 0],
            [3, GRAN_PACKAGE, -1]
        ]
    );
    let reply = json!({ "cycles": [[4, [1, 3]], [5, [2]]] });
    assert_eq!(
        cycle_rows(&reply).expect("cycles"),
        vec![[4, 1], [4, 3], [5, 2]]
    );
    assert!(
        cycle_rows(&json!({})).is_err(),
        "a reply without the key is malformed"
    );
}

/// The core's half, on the real core: sections collapse onto their
/// file, packages drop, a self-arc vanishes; a reported SCC counts iff
/// it holds a FILE (the section-only SCC 5 is the core's to report and
/// this tier's to ignore); the dead row carries its four verdict
/// columns and a live row none (absence is null, never a fabricated 0,
/// which on the trust scale means UNVOUCHED); a cycle member outside
/// the node list is refused by the document's contract.
#[test]
fn sections_collapse_packages_drop_and_cycles_count() {
    let (w, judged) = fixture();
    let core = crate::daemon::judge::core_bin().expect("a core");
    let reply = json!({
        "pos": [[0, 1, 2, 0, 1, 0], [1, 2, 0, 4, 2, 1]],
        "cycles": [[4, [1, 3]], [5, [2]]],
    });
    let screen = document((&core, Err(String::new())), &w, &judged, &reply).expect("laid out");
    assert_eq!(screen["schema"], "ce.graph-screen/0.1.0");
    let doc = &screen["canvas"];
    assert_eq!(doc["edges"], json!([[0, 1]]));
    assert_eq!(
        doc["counts"],
        json!({"files": 2, "edges": 1, "dead": 1, "cycles": 1})
    );
    assert_eq!(
        (&doc["files"][0]["cycle"], &doc["files"][1]["cycle"]),
        (&json!(false), &json!(true))
    );
    assert_eq!(doc["files"][0]["pos"], json!([1, 2, 0, 1, 0]));
    assert_eq!(doc["files"][1]["pos"], json!([2, 0, 4, 2, 1]));
    let verdict_columns = |row: &Value| json!({"verdict": row["verdict"], "why": row["why"], "whyCode": row["whyCode"], "conf": row["conf"]});
    assert_eq!(
        verdict_columns(&doc["files"][1]),
        json!({"verdict": "unref_private", "why": "no kept in-edge and no entry flag", "whyCode": 0, "conf": 2})
    );
    assert_eq!(
        verdict_columns(&doc["files"][0]),
        json!({"verdict": null, "why": null, "whyCode": null, "conf": null})
    );
    assert_eq!(
        (&doc["unresolvedSites"], &doc["degraded"], &doc["schema"]),
        (&json!(7), &Value::Null, &json!("ce.graph-canvas/0.4.0"))
    );
    let outside = json!({ "pos": reply["pos"], "cycles": [[0, [9]]] });
    assert!(document((&core, Err(String::new())), &w, &judged, &outside).is_err());
}
