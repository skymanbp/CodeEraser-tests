//! O54 (plan v2.29 step 10, batch C3, wire 7.1.0): the directed
//! dir-edge table rides `structure/1` and axis 7 judges it.
//!
//! Two things need evidence on this side, and neither is the
//! arithmetic — the digits are hand-checked in
//! `StructureModularityProps` and frozen in the golden pairs. First,
//! that the AXIS APPEARS EXACTLY WHEN THE TABLE DOES, driven through
//! the real core over one link: an old-shaped request draws no code-7
//! row at all, an empty table draws one at charge zero, and a grab-bag
//! geometry draws a charge and names the grab bag in the drill-down.
//! Second, that the MEASUREMENT actually sends the table on a real
//! tree — the leg no wire fixture can stand in for, because a producer
//! that never assembles the rows would leave every wire test above
//! still green.

use crate::common::{self, core_bin};
use codeeraser::structure::{judge, wire};

/// One request shape and what axis 7 must do with it. `edges: None` is
/// the old client; `Some(&[])` rode empty; `Some(rows)` rode a
/// geometry. `charge`/`finding` are None when the axis is UNJUDGED —
/// absence and zero are two states, which is the whole point here.
/// The full axis and finding arrays belong to the golden pairs; this
/// side owns presence.
struct Case {
    why: &'static str,
    nodes: &'static [[u64; 5]],
    refs: &'static [[u64; 4]],
    edges: Option<&'static [[u64; 3]]>,
    charge: Option<i64>,
    finding: Option<[i64; 2]>,
}

/// The grab-bag geometry the golden pairs and the battery share: root
/// over dir 1 and dir 2 (three files each, an internal 3-cycle) and
/// dir 3 (three files, no internal reference, three edges out to dir 1
/// and three in from dir 2). m = 12; dir 3 earns a negative
/// contribution, dirs 1 and 2 earn 500‰, the root has no mass at all,
/// so axis 7 counts one directory of four: charge 200.
const GRAB_NODES: &[[u64; 5]] = &[
    [0, 0, 0, 3, 0],
    [1, 0, 1, 0, 3],
    [2, 0, 1, 0, 3],
    [3, 0, 1, 0, 3],
];
const GRAB_REFS: &[[u64; 4]] = &[[1, 2, 1, 3], [2, 2, 1, 3], [3, 0, 2, 3]];
const GRAB_EDGES: &[[u64; 3]] = &[[2, 3, 3], [3, 1, 3]];

/// A one-directory tree with no references at all — the seat for the
/// two absence states (an empty table is clean; no table is unjudged).
const BARE_NODES: &[[u64; 5]] = &[[0, 0, 0, 0, 0]];

const CASES: &[Case] = &[
    Case {
        why: "an old-shaped request draws no code-7 row at all",
        nodes: BARE_NODES,
        refs: &[],
        edges: None,
        charge: None,
        finding: None,
    },
    Case {
        why: "an empty table is judged clean — axis 7 at charge zero",
        nodes: BARE_NODES,
        refs: &[],
        edges: Some(&[]),
        charge: Some(0),
        finding: None,
    },
    Case {
        why: "the grab bag is charged and named, the two modules are not",
        nodes: GRAB_NODES,
        refs: GRAB_REFS,
        edges: Some(GRAB_EDGES),
        charge: Some(200),
        finding: Some([3, 7]),
    },
];

fn request(c: &Case) -> wire::Request {
    wire::Request {
        nodes: c.nodes.to_vec(),
        patterns: Vec::new(),
        conventions: Vec::new(),
        file_refs: c.refs.to_vec(),
        declared: Vec::new(),
        stale_docs: None,
        redundancy: None,
        dir_edges: c.edges.map(<[[u64; 3]]>::to_vec),
        seams: None,
        knobs: Vec::new(),
    }
}

#[test]
fn the_modularity_axis_rides_exactly_when_the_dir_edge_table_does() {
    let core = core_bin();
    for c in CASES {
        let reply = wire::judge(&core, &request(c)).unwrap_or_else(|e| panic!("{}: {e}", c.why));
        let codes: Vec<i64> = reply.axes.iter().map(|[code, _]| *code).collect();
        assert_eq!(codes[..5], [0, 1, 2, 3, 4], "{}: S0..S4 always ride", c.why);
        let seen = reply.axes.iter().find(|[code, _]| *code == 7);
        assert_eq!(seen.map(|[_, p]| *p), c.charge, "{}", c.why);
        let named: Vec<[i64; 2]> = reply
            .findings
            .iter()
            .filter(|[_, a]| *a == 7)
            .copied()
            .collect();
        assert_eq!(
            named,
            c.finding.into_iter().collect::<Vec<_>>(),
            "{}",
            c.why
        );
        assert_eq!(reply.knobs.len(), 21, "{}: the knob echo is whole", c.why);
        assert!(
            reply.knobs.contains(&[19, 1]) && reply.knobs.contains(&[20, 4]),
            "{}: the two modularity floors echo their defaults",
            c.why
        );
    }
}

/// The cross-table law is the licence for reading the intra mass off
/// `fileRefs` instead of shipping it twice; a pair that does not
/// describe one graph is refused BY NAME, not judged on the half the
/// core happens to trust.
#[test]
fn a_dir_edge_table_that_disagrees_with_file_refs_is_refused_by_name() {
    let mut r = request(&CASES[2]);
    r.dir_edges = Some(vec![[2, 3, 3], [3, 1, 2]]);
    // .err(), not .expect_err(): Reply is a plain data face with no
    // Debug, and a test is not a reason to derive one on the wire type
    let err = wire::judge(&core_bin(), &r)
        .err()
        .expect("the pair does not describe one graph — refused");
    let text = format!("{err:#}");
    assert!(
        text.contains("dirEdges: directory 1") && text.contains("crossing endpoints"),
        "{text}"
    );
}

/// The producer leg: a real tree, the real walk, the real index — the
/// table has to be ASSEMBLED, not merely accepted. Two files that
/// reference nothing still make a tree with directories, so the axis
/// must appear with the table riding empty or clean; what is asserted
/// is presence, because the digits belong to the core.
#[test]
fn the_measurement_sends_the_table_on_a_real_tree() {
    let dir = common::tmp("structure-modularity-e2e");
    common::write_all(
        &dir,
        &[
            ("mod_a/one.py", "def one():\n    return 1\n"),
            ("mod_a/two.py", "def two():\n    return 2\n"),
        ],
    );
    common::build_index(&dir);
    let report = judge::run(&dir, None, &core_bin(), (false, None, false)).expect("structure");
    assert!(
        report.axes.iter().any(|[c, _]| *c == 7),
        "the measurement sent no dirEdges table: {:?}",
        report.axes
    );
}
