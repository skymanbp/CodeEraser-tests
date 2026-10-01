//! The report documents' catalogue in the definition package (plan
//! v2.32 step 3; design booklet docs/reference/authority-track.md §5,
//! §13 items 17–19): the names this side still spells for its own
//! readers — each face's schema id (the facts registry and the tests),
//! the flow kind names (the hook legs' feeds) and the flow judged
//! languages (the package's own `flow_judged` column) — are the ones
//! the core's documents carry.

use crate::common::stub_core;
use crate::facts::report::LINKED;
use serde_json::{Value, json};

fn catalogue() -> &'static Value {
    &stub_core::real_tables()["document"]
}

/// Each family the catalogue names: its schema id is the one the facts
/// registry links for that family name (the face's `pub` constant).
#[test]
fn the_faces_schema_ids_are_the_catalogues() {
    let named: Vec<&str> = catalogue()
        .as_object()
        .expect("families")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(named, ["arch", "flow", "merge", "query", "rules"]);
    for family in named {
        let linked = LINKED.iter().find(|(n, _)| *n == family);
        let id = linked.unwrap_or_else(|| panic!("{family}: not linked")).1;
        assert_eq!(catalogue()[family]["schema"], id, "{family}");
    }
}

#[test]
fn the_flow_names_are_the_catalogues() {
    let flow = &catalogue()["flow"];
    assert_eq!(
        flow["kinds"],
        json!(codeeraser::flow_report::KINDS),
        "kinds"
    );
    let mask = codeeraser::flow::judged_mask();
    let judged: Vec<i64> = (0..64).filter(|l| mask >> l & 1 == 1).collect();
    assert_eq!(flow["judged"], json!(judged), "judged languages");
}
