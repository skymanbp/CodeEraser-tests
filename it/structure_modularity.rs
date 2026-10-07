//! O54 (plan v2.29 step 10, batch C3, wire 7.1.0): the directed
//! dir-edge table rides `structure/1` and axis 7 judges it.
//!
//! Two things need evidence on this side, and neither is the
//! arithmetic — the digits are hand-checked in
//! `StructureModularityProps` and frozen in the golden pairs. First,
//! that the AXIS RIDES THE TABLES THE CORE BUILDS from the paths and
//! arcs (plan v2.33 W1 item 3), driven through the real core over one
//! link: a tree with no arcs draws the axis at charge zero, and a
//! grab-bag geometry draws a charge and names the grab bag in the
//! drill-down. (The integer form's third state — no table, the axis
//! unjudged — is the core's contract and its golden pair's; the path
//! form always draws the table.) Second, that the MEASUREMENT actually
//! sends the arcs on a real tree — the leg no wire fixture can stand in
//! for, because a producer that never reads them would leave every wire
//! test above still green.

use crate::common::{self, core_bin};
use codeeraser::structure::rows::Arcs;
use codeeraser::structure::{judge, wire};

/// One request shape and what axis 7 must do with it: the judged paths
/// and the file-to-file arcs as slot pairs over the same paths. The
/// full axis and finding arrays belong to the golden pairs; this side
/// owns presence.
struct Case {
    why: &'static str,
    paths: &'static str,
    arcs: &'static [[usize; 2]],
    charge: i64,
    finding: Option<[i64; 2]>,
}

/// The grab-bag geometry the golden pairs and the battery share: root
/// over dir 1 and dir 2 (three files each, an internal 3-cycle) and
/// dir 3 (three files, no internal reference, three edges out to dir 1
/// and three in from dir 2). m = 12; dir 3 earns a negative
/// contribution, dirs 1 and 2 earn 500‰, the root has no mass at all,
/// so axis 7 counts one directory of four: charge 200.
const GRAB_PATHS: &str = "d1/a.py d1/b.py d1/c.py d2/a.py d2/b.py d2/c.py d3/a.py d3/b.py d3/c.py";
const GRAB_ARCS: &[[usize; 2]] = &[
    [0, 1],
    [1, 2],
    [2, 0],
    [3, 4],
    [4, 5],
    [5, 3],
    [6, 0],
    [7, 1],
    [8, 2],
    [3, 6],
    [4, 7],
    [5, 8],
];

const CASES: &[Case] = &[
    Case {
        why: "a tree with no arcs is judged clean — axis 7 at charge zero",
        paths: "x.py",
        arcs: &[],
        charge: 0,
        finding: None,
    },
    Case {
        why: "the grab bag is charged and named, the two modules are not",
        paths: GRAB_PATHS,
        arcs: GRAB_ARCS,
        charge: 200,
        finding: Some([3, 7]),
    },
];

fn request(c: &Case) -> wire::Request {
    let paths: Vec<String> = c.paths.split(' ').map(String::from).collect();
    wire::Request {
        arcs: Arcs {
            paths: paths.clone(),
            files: c.arcs.to_vec(),
            sections: Vec::new(),
            packages: Vec::new(),
        },
        paths,
        layout: Vec::new(),
        stale: None,
        redundancy: None,
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
        assert_eq!(seen.map(|[_, p]| *p), Some(c.charge), "{}", c.why);
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

/// The two tables are one graph by construction now — the core draws
/// both off the one arc list — so the law that once licensed reading
/// the intra mass off `fileRefs` cannot be broken from this side; what
/// can is the arc list itself, and an arc naming no path is refused BY
/// NAME, not judged on the arcs the core happens to place.
#[test]
fn an_arc_naming_no_path_is_refused_by_name() {
    let mut r = request(&CASES[1]);
    r.arcs.files.push([2, 9]);
    // .err(), not .expect_err(): Reply is a plain data face with no
    // Debug, and a test is not a reason to derive one on the wire type
    let err = wire::judge(&core_bin(), &r)
        .err()
        .expect("an arc to slot 9 of nine paths — refused");
    let text = format!("{err:#}");
    assert!(
        text.contains("refPairs 12") && text.contains("both slots of refPaths"),
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
    let answer = judge::run(&dir, None, &core_bin(), (false, None, false)).expect("structure");
    let axes = &answer.document["axes"];
    let codes = axes.as_array().map_or(&[][..], Vec::as_slice);
    assert!(
        codes.iter().any(|a| a[0] == 7),
        "the measurement sent no dirEdges table: {axes}"
    );
}
