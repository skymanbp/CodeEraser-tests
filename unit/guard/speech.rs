use super::*;
use crate::document::Resolve;
use crate::tombstone::{Kind, Row};

fn site(file: &str, line: usize, kind: Kind) -> Row {
    Row {
        file: file.into(),
        line,
        kind,
        marks: 0,
        names: 1,
        name: String::new(),
        excerpt: String::new(),
        ledger: 0,
    }
}

/// One rule of each kind, in the order the hook gathers them.
fn every_rule() -> Vec<Said> {
    let m = |file: &str, start| Match {
        file: file.into(),
        start,
        end: start + 8,
        tokens: 60,
    };
    vec![
        Said::Duplicate {
            file: "b.rs".into(),
            regions: 4,
            top: vec![m("a.rs", 1), m("c.rs", 20)],
        },
        Said::OverBudget {
            file: "b.rs".into(),
            lines: 760,
            cap: 750,
            fence: Fence::Drifted,
        },
        Said::Zone {
            file: "z.rs".into(),
            lines: 600,
            permille: 666,
            soft: 300,
            cap: 750,
        },
        Said::Tombstone {
            sites: 3,
            budget: 0,
            fence: Fence::Unreadable,
            shown: vec![
                site("r.md", 1, Kind::Bracketed),
                site("s.md", 4, Kind::Prose),
            ],
        },
        Said::FlowNovel {
            file: "b.py".into(),
            novel: 2,
            unit: "gone".into(),
            kind: 1,
            line: 3,
        },
        Said::ConfigUnreadable("TOML parse error".into()),
    ]
}

#[test]
fn every_rule_is_one_say_row_and_every_string_a_reference() {
    let s = Strings::of(&every_rule());
    let body = s.body();
    let rows = serde_json::json!({
        "say": [
            [0, 0, 4, 0, 2, 0, 0],
            [1, 1, 760, 750, 1, 0, 0],
            [2, 2, 600, 666, 300, 750, 0],
            [3, 3, 0, 2, 0, 2, 0],
            [4, 3, 2, 0, 1, 3, 0],
            [5, 0, 0, 0, 0, 0, 0],
        ],
        "matches": [[0, 1, 9, 60], [1, 20, 28, 60]],
        "places": [[0, 1, 0], [1, 4, 2]],
    });
    assert_eq!(body["rows"], rows);
    let ranges =
        serde_json::json!({"files": 4, "matches": 2, "places": 2, "units": 1, "errors": 1});
    assert_eq!(body["ranges"], ranges);
    assert_eq!(
        (body["family"].as_str(), &body["facts"]),
        (Some("guard"), &serde_json::json!({}))
    );
    assert_eq!(body["lang"], lines::lang());
    let s = s.lists();
    let names = [
        ("file", 3, "b.py"),
        ("match_file", 1, "c.rs"),
        ("place_file", 1, "s.md"),
        ("unit", 0, "gone"),
        ("error", 0, "TOML parse error"),
    ];
    for (class, i, want) in names {
        assert_eq!(s.resolve(class, &[i]).as_deref(), Some(want), "{class} {i}");
    }
    assert_eq!(s.resolve("file", &[4]), None);
    assert_eq!(s.resolve("nosuch", &[0]), None);
}

#[test]
fn the_fallback_names_every_rule_and_why() {
    assert_eq!(
        fallback(&every_rule(), "core_unavailable"),
        "ce: rule duplicate, over_budget, graded_zone, tombstone_over, flow_novel, \
         config_unreadable fired; the core could not phrase the reason: core_unavailable"
    );
}
