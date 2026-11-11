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
