//! graph/wire.rs unit leg (lane v235_licence, item L2 — graphtrust's
//! T4): the ledger word an answered site stores (`sites.ledger`). Bit 0
//! says the answer was External — a correct terminal answer that draws
//! no edge, which the trust ledger (graph/load.rs `UNRESOLVED_ROWS`)
//! reads as answered — and only External sets it; bit 1 is the opaque
//! star import, independent of it.

use super::*;
use crate::graph::ladder::Reason;

fn word(outcome: Outcome, opaque: bool) -> (usize, i64) {
    let also = ladder::Also {
        rows: Vec::new(),
        opaque,
    };
    let (rows, ledger) = answered("import", outcome, also);
    (rows.len(), ledger)
}

#[test]
fn only_an_external_answer_sets_ledger_bit_zero() {
    let resolved = || Outcome::Resolved {
        path: "a.py".into(),
        rung: 1,
    };
    let got = [
        word(Outcome::External { rung: 5 }, false),
        word(Outcome::External { rung: 5 }, true),
        word(resolved(), false),
        word(Outcome::Unresolved(Reason::Dynamic), false),
        word(Outcome::Unresolved(Reason::Dynamic), true),
    ];
    // (edge rows, ledger word): External draws no edge and sets bit 0;
    // a resolved answer draws its edge, bit 0 clear; an unresolved one
    // neither; bit 1 rides beside bit 0 for an opaque star import
    assert_eq!(got, [(0, 1), (0, 3), (1, 0), (0, 0), (0, 2)]);
}

/// Part 4C: a confirmed section answer with its holder `also` row stores
/// two edges of the site's own kind — the section-granular answer
/// (`also` 0), then the file-granular holder (`also` 1).
#[test]
fn a_section_answer_and_its_holder_row_store_two_edges() {
    let section = Outcome::ResolvedSection {
        path: "c.md".into(),
        slug: Some("sec".into()),
        rung: 2,
    };
    let holder = ladder::Also {
        rows: vec![("c.md".into(), 2)],
        opaque: false,
    };
    let (rows, ledger) = answered("link", section, holder);
    let seen: Vec<(&str, &str, i64, i64, i64)> = rows
        .iter()
        .map(|e| {
            (
                e.dst_path.as_str(),
                e.dst_unit.as_str(),
                e.granularity,
                e.also,
                e.rung,
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            ("c.md", "sec", GRAN_SECTION, 0, 2),
            ("c.md", "", GRAN_FILE, 1, 2)
        ]
    );
    assert_eq!(
        (rows[0].kind, ledger),
        (rows[1].kind, 0),
        "one kind, no ledger bit"
    );
}
