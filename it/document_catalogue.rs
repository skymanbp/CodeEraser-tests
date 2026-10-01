//! The report documents' catalogue in the definition package (plan
//! v2.32 step 3; design booklet docs/reference/authority-track.md §5,
//! §13 items 7 and 17–19). The product holds no copy of it: a face
//! prints the `schema` its bound document carries, the hook legs' feeds
//! read the flow kind names off the package, and `--kind` is the core's
//! to refuse. So the catalogue is pinned here against the frozen tables
//! golden, and every family's bound document against the catalogue.

use crate::common::{self, stub_core};
use serde_json::{Value, json};

fn catalogue() -> &'static Value {
    &stub_core::real_tables()["document"]
}

/// The catalogue the frozen `tables/golden.ndjson` reply carries.
fn frozen() -> Value {
    let path = common::repo_root().join("contracts/fixtures/tables/golden.ndjson");
    let text = std::fs::read_to_string(&path).expect("tables golden");
    let reply = text
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).expect("a json line"))
        .find(|v| v["type"] == "tables.result")
        .expect("a tables.result line");
    reply["document"].clone()
}

/// The core's catalogue is the frozen one, twelve families, and flow's
/// kind names and judged languages are the package's own.
#[test]
fn the_catalogue_is_the_frozen_one() {
    assert_eq!(catalogue(), &frozen(), "the package against the golden");
    let named: Vec<&str> = catalogue()
        .as_object()
        .expect("families")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        named,
        [
            "arch",
            "check",
            "deadcode",
            "flow",
            "graphscreen",
            "join",
            "mentions",
            "merge",
            "query",
            "rules",
            "sites",
            "structure"
        ]
    );
    let flow = &catalogue()["flow"];
    let kinds = json!([
        ["unreachable", false],
        ["dead_store", false],
        ["unused_local", false],
        ["unused_param", true]
    ]);
    assert_eq!(
        flow["kinds"], kinds,
        "kinds and their advisory role, as observed"
    );
    let mask = codeeraser::flow::judged_mask();
    let judged: Vec<i64> = (0..64).filter(|l| mask >> l & 1 == 1).collect();
    assert_eq!(flow["judged"], json!(judged), "judged languages");
}

/// Each family's document, bound over a one-file tree, carries the
/// catalogue's schema id; flow's findings name catalogue kinds.
#[test]
fn every_bound_document_carries_its_catalogue_schema() {
    let dir = common::tmp("document-catalogue");
    let seed = "def gone():\n    return 1\n    print(2)\n";
    std::fs::write(dir.join("a.py"), seed).expect("seed");
    let core = common::core_bin();
    let docs = [
        ("arch", codeeraser::faces::arch(&dir, &core, &[])),
        ("flow", codeeraser::faces::flow(&dir, &core, &[])),
        ("merge", codeeraser::faces::merge(&dir, &core)),
        (
            "query",
            codeeraser::faces::query(&dir, &core, "dead(F)", false, None),
        ),
        ("rules", codeeraser::faces::rules(&dir, &core, None, false)),
    ];
    for (family, doc) in docs {
        let doc = doc.unwrap_or_else(|e| panic!("{family}: {e:#}"));
        assert_eq!(doc["schema"], catalogue()[family]["schema"], "{family}");
        if family == "flow" {
            let rows = catalogue()["flow"]["kinds"].as_array().expect("kinds");
            let listed: Vec<&Value> = rows.iter().map(|r| &r[0]).collect();
            let found = doc["findings"].as_array().expect("findings");
            assert!(!found.is_empty(), "the seed has an unreachable run");
            assert!(found.iter().all(|f| listed.contains(&&f["kind"])));
        }
    }
}
