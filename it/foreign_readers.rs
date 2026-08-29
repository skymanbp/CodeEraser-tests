//! A declared submodule is a READER of the tree, never a measured part
//! of it (plan v2.18 step #12, user ruling 2026-08-28: the suite rides
//! at `cli/tests` as a submodule so that its lines leave the main
//! repository's score — a seat that kept measuring them would have
//! made the split pointless). One superproject, every surface asked:
//! the measurement walk yields the submodule's files flagged foreign
//! and no size, score, clone or ratchet row is ever cut from them;
//! the graph still reads their references (an own file linked only
//! from the submodule lives) and the advisory still reads their text
//! (an own declaration spelled only there is vetoed); their own files
//! are never verdict candidates; the guard is inert on a write there;
//! and a nested repository nobody declared is cut whole. The unseated
//! refusal (trend_submodule.rs) stands: a missing reader would judge
//! this tree's files without the references they hold.

use crate::common;
use codeeraser::graph::deadcode;
use std::path::{Path, PathBuf};

/// The superproject: `root.rs` opens on a license header and declares
/// a public name nothing of this tree's own references or spells; the
/// submodule at `suite` (the T2 clone pair `seed_superproject` mounts)
/// carries the same license header in `lic.rs` and gains a README
/// that links `root.rs` and spells the name when `reader` is set; a
/// nested repository nobody declared holds a 751-line file. `[dedup]
/// budget = 0` would fail on the mounted pair if the pair were
/// measured.
/// A whole Apache header: the first comment inside the head window
/// (spec LICENSE_HEAD_LINES) clearing the segment floor (MIN_DOC_TOKENS
/// 50 words) — a one-liner never becomes a docsegs row at all.
const LICENSE: &str = "// Licensed under the Apache License, Version 2.0 (the License);\n\
// you may not use this file except in compliance with the License.\n\
// You may obtain a copy of the License at the Apache Software Foundation.\n\
// Unless required by applicable law or agreed to in writing, software\n\
// distributed under the License is distributed on an AS IS BASIS,\n\
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\n\
// See the License for the specific language governing permissions and\n\
// limitations under the License.\n";

fn superproject(tag: &str, reader: bool) -> PathBuf {
    let sup = common::seed_superproject(tag, "suite");
    std::fs::write(
        sup.join("root.rs"),
        format!("{LICENSE}pub fn spoken_only_abroad() -> i64 {{\n    3\n}}\n"),
    )
    .expect("root.rs");
    std::fs::write(sup.join("suite").join("lic.rs"), LICENSE).expect("lic.rs");
    std::fs::write(sup.join("ce.toml"), "[dedup]\nbudget = 0\n").expect("ce.toml");
    if reader {
        std::fs::write(
            sup.join("suite").join("README.md"),
            "# suite\n\nSee [the root](../root.rs): `spoken_only_abroad` lives there.\n",
        )
        .expect("README.md");
    }
    std::fs::create_dir_all(sup.join("nested").join(".git")).expect("nested/.git");
    std::fs::write(sup.join("nested").join("big.rs"), "// filler\n".repeat(751))
        .expect("nested/big.rs");
    sup
}

fn dead_paths(report: &deadcode::Report) -> Vec<String> {
    report.dead.iter().map(|d| d.path.clone()).collect()
}

fn advised_symbols(report: &deadcode::Report) -> Vec<String> {
    let doc = codeeraser::report::deadcode_json(report);
    doc["unmentioned"]
        .as_array()
        .expect("advisory rows")
        .iter()
        .map(|r| r["symbol"].as_str().expect("symbol").to_string())
        .collect()
}

/// The measurement walk: the submodule's files come back flagged
/// foreign, the undeclared repository's never come back, and every
/// measuring face reads the flag — `ce scan` measures `root.rs`
/// alone, the clone gate passes on a budget the mounted pair would
/// break, and the ratchet baseline holds no foreign row.
#[test]
fn foreign_files_are_walked_but_never_measured() {
    let sup = superproject("foreign-readers-measure", true);
    assert_eq!(
        common::walked(&sup),
        [
            ("ce.toml".to_string(), false),
            ("root.rs".to_string(), false),
            ("suite/README.md".to_string(), true),
            ("suite/a.rs".to_string(), true),
            ("suite/b.rs".to_string(), true),
            ("suite/lic.rs".to_string(), true),
        ],
        "declared = foreign, undeclared = cut"
    );
    let (_cfg, rows) = codeeraser::scan::measure(&sup).expect("measure");
    let measured: Vec<&str> = rows.iter().map(|r| r.path.as_str()).collect();
    assert_eq!(measured, ["root.rs"], "no size row is cut from a reader");
    let out = common::run_ce(
        &sup,
        &["dedup", ".", "--check", "--core", &common::core_bin()],
    );
    assert!(
        out.status.success(),
        "the mounted T2 pair is foreign, so budget 0 holds: {out:?}"
    );
    // the unit universe and the docdup exemption counts read the same
    // owner line (codex review of step #12: both once read every file)
    let (idx, _w) = common::graph_wire(&sup, deadcode::Advisory::No);
    let units: Vec<String> = codeeraser::dedup::unitcache::unit_rows(&idx)
        .expect("units")
        .into_iter()
        .map(|u| u.path)
        .collect();
    assert!(
        !units.is_empty() && units.iter().all(|p| !p.starts_with("suite/")),
        "no foreign unit is a T3 candidate: {units:?}"
    );
    assert_eq!(
        codeeraser::docdup::judge::candidates::exempt_counts(&idx).expect("exempt"),
        (1, 0),
        "root.rs's license header counts, suite/lic.rs's is its own tree's"
    );
    // the ratchet is a table of metric rows, not paths: the
    // superproject's equals that of a tree holding its own files alone
    let own = common::fixtures::tmp("foreign-readers-own");
    for rel in ["root.rs", "ce.toml"] {
        std::fs::copy(sup.join(rel), own.join(rel)).expect(rel);
    }
    let table = |dir: &Path| -> serde_json::Value {
        // O31: an establish is a named act — the fixture has no
        // committed floor, so the wholesale act is the road
        let out = common::run_ce_env(dir, &["baseline", "."], &[("CE_ACCEPT_BASELINE", "1")]);
        assert!(out.status.success(), "ce baseline: {out:?}");
        let text = std::fs::read_to_string(dir.join("ce-baseline.json")).expect("baseline");
        serde_json::from_str::<serde_json::Value>(&text).expect("json")["continuous"].clone()
    };
    assert_eq!(table(&sup), table(&own), "the ratchet holds own rows only");
}

/// The graph and the advisory READ the submodule: `root.rs` is linked
/// from the submodule's README alone and lives; its declaration is
/// spelled there alone and is not advised. The control tree without
/// the README shows the same file dead and the same name advised, so
/// the reading — not some entry convention — is what the assertions
/// see. In both trees the submodule's own files are never dead and
/// never advised: they are nobody's verdict candidates here.
#[test]
fn foreign_files_read_for_the_graph_and_the_advisory() {
    let core = common::core_bin();
    let read = superproject("foreign-readers-graph", true);
    let report = deadcode::run(&read, None, &core).expect("deadcode");
    assert_eq!(
        dead_paths(&report),
        Vec::<String>::new(),
        "linked from a foreign reader, root.rs lives; foreign files are never dead"
    );
    assert_eq!(
        advised_symbols(&report),
        Vec::<String>::new(),
        "spelled by a foreign reader, the name is vetoed; foreign declarations are never advised"
    );
    assert!(
        report
            .reported
            .iter()
            .all(|(p, _)| !p.starts_with("suite/")),
        "no aggregate verdict on a reader: {:?}",
        report.reported
    );
    let control = superproject("foreign-readers-control", false);
    let report = deadcode::run(&control, None, &core).expect("deadcode");
    assert_eq!(
        dead_paths(&report),
        ["root.rs"],
        "unread, root.rs dies alone"
    );
    assert_eq!(
        advised_symbols(&report),
        ["spoken_only_abroad"],
        "unspelled, the name is advised alone"
    );
}

/// The staleness measurement (S5) reads a foreign doc's links as
/// edges and never places the doc: `suite/README.md` links `root.rs`
/// and has no directory in the own tree — before the fix `ce check
/// --days` died on it by name ("md node suite/README.md outside the
/// walked tree"; codex review of step #12). An own doc with the same
/// link is the one doc row.
#[test]
fn a_foreign_doc_is_read_and_never_a_stale_candidate() {
    use codeeraser::structure::{judge, rows, tree};
    let core = common::core_bin();
    let read = superproject("foreign-readers-stale", true);
    std::fs::write(
        read.join("NOTES.md"),
        "# notes

See [the root](root.rs).
",
    )
    .expect("NOTES.md");
    judge::run(&read, None, &core, (false, Some(30), false))
        .expect("a foreign doc's link is an edge, not a doc row");
    let own: Vec<String> = common::walked(&read)
        .into_iter()
        .filter(|(_, foreign)| !foreign)
        .map(|(p, _)| p)
        .collect();
    let (_idx, w) = common::graph_wire(&read, deadcode::Advisory::No);
    let (docs, edges) =
        rows::stale_doc_rows(&read, &w, &tree::build(&own), 30).expect("stale rows");
    assert_eq!(docs.len(), 1, "NOTES.md alone is placed: {docs:?}");
    assert_eq!(
        edges.len(),
        1,
        "root.rs, committed in the window, is NOTES.md's one changed target: {edges:?}"
    );
    assert_eq!(edges[0][0], 0, "the edge hangs on the own doc: {edges:?}");
}

/// The guard is inert on a foreign path while the submodule has no
/// gate of its own, live on an own path, and — once the submodule
/// seats a ce.toml — delegated: the same write is judged under the
/// submodule's root, its own config, from the session's own cwd.
#[test]
fn the_guard_is_inert_on_a_foreign_write() {
    let sup = superproject("foreign-readers-guard", true);
    let big = "// filler\n".repeat(751);
    let inert = common::run_hook(
        &sup,
        &["probe", "--hook"],
        &common::pretooluse_envelope_at(&sup, "suite/new.rs", "Write", &big),
    );
    assert!(
        inert.trim().is_empty(),
        "a foreign write is not ours: {inert}"
    );
    // an own write is ours
    common::expect_write_denied(&sup, "new.rs", &big, "751 lines");
    // gated, the same foreign write is judged by the submodule's own
    // gate — from this session's cwd
    std::fs::write(sup.join("suite/ce.toml"), "[guard]\nmode = \"deny\"\n").expect("gate");
    common::expect_write_denied(&sup, "suite/new.rs", &big, "751 lines");
}

/// The mention universe reads through the same owner rule: the
/// submodule's files are in U (they spell names), the undeclared
/// repository's are cut.
#[test]
fn the_mention_universe_reads_foreign_and_cuts_undeclared() {
    let sup = superproject("foreign-readers-universe", true);
    let cut = |rel: &str| codeeraser::mention::cut(Path::new(&sup), rel);
    assert!(!cut("suite/README.md"), "a declared submodule is in U");
    assert!(cut("nested/big.rs"), "an undeclared repository is not");
}
