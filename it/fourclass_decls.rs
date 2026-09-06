//! Declaration-level relocation, end to end over a real core link
//! (proto 7.1.0, O48). One table: a one-line function
//! whose body cannot open a station still relocates by its
//! declaration, and the same removal against TWO identical
//! destinations relocates nowhere — the ambiguous destination is
//! refused, not tie-broken.

use crate::common::core_bin;
use codeeraser::corelink::Link;
use codeeraser::fourclass::batch::{PairInput, classify_batch};
use codeeraser::scan::lang::Lang;

/// The helper before it moves, and after: the declaration line
/// changes (`pub`), so exactly ONE body line survives byte-identical
/// — one line is the coincidence tie, below destFloor.
const GONE: &str = "fn helper(d: u32) -> u32 {\n    d.wrapping_mul(2654435761).rotate_left(7)\n}\n";
const ARRIVED: &str =
    "pub fn helper(d: u32) -> u32 {\n    d.wrapping_mul(2654435761).rotate_left(7)\n}\n";

/// (case, pairs as (before, after), expected declaration edges as
/// (from pair, to pair, unit key)).
type Case = (
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static [(usize, usize, &'static str)],
);

const CASES: [Case; 4] = [
    (
        "one destination: the declaration opens the edge no station could",
        &[(GONE, ""), ("", ARRIVED)],
        &[(0, 1, "helper/1")],
    ),
    (
        "two identical destinations: the key is refused, not tie-broken",
        &[(GONE, ""), ("", ARRIVED), ("", ARRIVED)],
        &[],
    ),
    (
        "many sources may reach one destination",
        &[(GONE, ""), (GONE, ""), ("", ARRIVED)],
        &[(0, 2, "helper/1"), (1, 2, "helper/1")],
    ),
    (
        "a destination without leftover lines still makes the key ambiguous",
        &[
            (GONE, ""),
            ("", ARRIVED),
            (
                "/*\npub fn helper(d: u32) -> u32 {\n    d.wrapping_mul(2654435761).rotate_left(7)\n}\n*/\n",
                ARRIVED,
            ),
        ],
        &[],
    ),
];

#[test]
fn a_short_body_relocates_by_its_declaration_and_a_tie_relocates_nowhere() {
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    for (name, texts, want) in CASES {
        let inputs: Vec<PairInput> = texts
            .iter()
            .map(|(b, a)| PairInput {
                before: b,
                after: a,
                lang: Lang::Rust,
            })
            .collect();
        let batch = classify_batch(&inputs, Some(&mut link));
        assert!(batch.degraded.is_none(), "{name}: {:?}", batch.degraded);
        let got: Vec<_> = batch
            .relocations
            .iter()
            .filter(|r| r.lines == 0)
            .map(|r| {
                (
                    r.from_pair,
                    r.to_pair,
                    r.to_unit
                        .as_deref()
                        .expect("declaration edges name both ends"),
                )
            })
            .collect();
        assert_eq!(got, want, "{name}");
        for (pair, input) in batch.pairs.iter().zip(&inputs) {
            let l1 = codeeraser::fourclass::classify(input.before, input.after, input.lang);
            assert_eq!(pair.counts, l1.counts, "{name}: no line changes class");
            assert_eq!(pair.moved, l1.moved, "{name}: no moved-line delta");
        }
        // the LINE stage said nothing either way: one shared line is
        // the tie that does not open a station
        assert!(
            batch.relocations.iter().all(|r| r.lines == 0),
            "{name}: a station opened where the floor forbids one"
        );
    }
}
