//! The MCP report catalog is a transport, never a second serializer
//! (M7-P2 acceptance): each report tool's text equals the same report
//! produced through the family's public library face, and each
//! declared knob reaches the library. Split out of mcp_precommit.rs
//! (plan v2.33 step 11b), which keeps the catalogue pin and the
//! pre-commit gate.

use crate::common;
use crate::common::{McpSession, git, tmp};
use codeeraser::faces;

/// M7-P2 acceptance: each report tool's MCP text equals the SAME
/// report produced through the family's public library face — the
/// catalog is a transport, never a second serializer. The project is
/// git-seeded so the churn/join windows have history to read.
#[test]
fn mcp_report_faces_match_library() {
    let dir = tmp("mcp-faces");
    common::seed_clone_pair(&dir);
    git(&dir, &["init", "-q"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-qm", "seed"]);
    // Steady-state the index once first: Summary carries refresh
    // counters (refreshed/removed/stale_skipped) and the clone report
    // carries the verdict cache's (cached/judged/requests, plan v2.30
    // step 5b-9), so both faces must observe the same warm index and
    // warm cache — otherwise this compares cache states, not
    // serializations.
    codeeraser::dedup::analyze(&dir, None, None, None).expect("warm");
    codeeraser::dedup::t3::run(&dir, None, &common::core_bin()).expect("warm t3");
    let mut s = McpSession::over(&dir);
    for (id, (name, args, want)) in library_reports(&dir).into_iter().enumerate() {
        let got = s.ask(serde_json::json!({
            "jsonrpc": "2.0", "id": 100 + id, "method": "tools/call",
            "params": {"name": name, "arguments": args}
        }));
        assert_eq!(got["result"]["isError"], false, "{name} errored: {got}");
        let text = got["result"]["content"][0]["text"].as_str().expect("text");
        assert_eq!(text, want, "{name} drifted from its library face");
    }
    s.finish();
}

/// The expected report strings, each produced through the public
/// library face the MCP adapter claims to be a transport for — rows
/// built by one closure so the builder cannot re-grow parallel row
/// stanzas (the census caught two builders doing exactly that). The
/// split below is the repo's own seam, not an arbitrary halving:
/// four families report a measurement (scan, dedup, churn, sites), the
/// rest a judgment. Splitting per ROW would recreate the
/// twin-stanza clone this comment used to justify one long builder
/// with; splitting along the seam does not.
fn library_reports(dir: &std::path::Path) -> Vec<(&'static str, serde_json::Value, String)> {
    let mut rows = measured_reports(dir);
    rows.extend(judged_reports(dir));
    rows
}

type Row = (&'static str, serde_json::Value, String);

fn no_args(name: &'static str, text: String) -> Row {
    (name, serde_json::json!({}), text)
}

/// The four faces whose measurement no judgment reads.
fn measured_reports(dir: &std::path::Path) -> Vec<Row> {
    use codeeraser::{graph, scan};
    let core = common::core_bin();
    let row = no_args;
    let scan = scan::judged(dir, &core).expect("scan").document;
    vec![
        row(
            "scan",
            codeeraser::document::rendered("scan", &scan).expect("scan json"),
        ),
        row(
            "check_duplication",
            faces::gated_doc(faces::dedup(
                dir,
                &core,
                (None, None),
                faces::Plumb::default(),
            ))
            .expect("dedup")
            .to_string(),
        ),
        row(
            "churn",
            faces::doc(faces::churn(dir, &core, 14))
                .expect("churn")
                .to_string(),
        ),
        row(
            "graph_sites",
            graph::sites_document(&core, &graph::analyze(dir).expect("sites"))
                .expect("the sites document")
                .document
                .to_string(),
        ),
    ]
}

/// The faces that judge. The trend row runs library-first on purpose:
/// it seeds the cache, so the MCP call reads the same warm rows (the
/// Summary refresh-counter lesson, applied to the trend cache).
fn judged_reports(dir: &std::path::Path) -> Vec<Row> {
    let core = common::core_bin();
    let mut rows = graph_and_clone_reports(dir, &core);
    rows.extend(gate_reports(dir, &core));
    rows
}

/// The first half of `judged_reports`: deadcode, clone, docdup, join.
fn graph_and_clone_reports(dir: &std::path::Path, core: &str) -> Vec<Row> {
    use codeeraser::{dedup, docdup, graph, join};
    let row = no_args;
    vec![
        row(
            "deadcode",
            graph::deadcode::run(dir, None, core)
                .expect("dead")
                .doc
                .to_string(),
        ),
        row(
            "clone",
            dedup::t3::answer(dir, None, core)
                .expect("the clone document")
                .document
                .to_string(),
        ),
        row(
            "docdup",
            docdup::judge::answer(dir, None, core, false)
                .expect("the docdup document")
                .document
                .to_string(),
        ),
        row(
            "join",
            join::run(dir, None, core, 14)
                .expect("join")
                .document
                .to_string(),
        ),
    ]
}

/// The second half of `judged_reports`: structure, check, erase, trend.
fn gate_reports(dir: &std::path::Path, core: &str) -> Vec<Row> {
    use codeeraser::{score, structure, trend};
    let row = no_args;
    let opts = score::Opts {
        db: None,
        core: core.into(),
        days: None,
        floor: None,
        establish: false,
        pinned_soft: None,
        baseline: None,
    };
    vec![
        row(
            "structure",
            structure::judge::run(dir, None, core, (false, None, false))
                .expect("structure")
                .document
                .to_string(),
        ),
        row(
            "check",
            score::document::document(core, &mut score::run(dir, opts).expect("check"), false)
                .expect("the check document")
                .document
                .to_string(),
        ),
        row(
            "erase",
            faces::gated_doc(faces::erase(dir, core, Default::default(), |_, _| None))
                .expect("erase plan")
                .to_string(),
        ),
        // doctor is deliberately absent from this parity list: it is
        // the one face whose value is the MACHINE's state, so its
        // document moves between two calls (the daemon warms, the
        // observe feed grows) and a byte comparison would pin a clock.
        // doctor_face.rs asserts its shape instead.
        row(
            "trend",
            // commits=10 mirrors the MCP adapter's default exactly
            trend::answer(core, &trend::run(dir, None, core, 10, None).expect("trend"))
                .expect("the trend document")
                .document
                .to_string(),
        ),
    ]
}

/// The knob PARAMETERS, which are the other half of "the catalog is
/// a transport": a tool that accepts a knob and ignores it is a
/// transport that lies. `units` switches `clone` to the other
/// document its own CLI flag produces, and `min_distinct` is the
/// diversity floor the face could not receive at all until now.
#[test]
fn declared_knobs_reach_the_library() {
    let dir = tmp("mcp-knobs");
    common::seed_clone_pair(&dir);
    codeeraser::dedup::analyze(&dir, None, None, None).expect("warm");
    let mut s = McpSession::over(&dir);
    let call = |s: &mut McpSession, id, name: &str, args: serde_json::Value| {
        let got = s.ask(serde_json::json!({
            "jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": name, "arguments": args}
        }));
        assert_eq!(got["result"]["isError"], false, "{name}: {got}");
        got["result"]["content"][0]["text"]
            .as_str()
            .expect("text")
            .to_string()
    };
    let units = call(&mut s, 300, "clone", serde_json::json!({"units": true}));
    assert_eq!(
        units,
        faces::doc(faces::clone(
            &dir,
            &common::core_bin(),
            true,
            Default::default()
        ))
        .expect("units")
        .to_string(),
        "the units switch must reach the OTHER document, not the judgment"
    );
    let judged = call(&mut s, 301, "clone", serde_json::json!({}));
    assert!(
        judged != units,
        "units:true and units:false answered the same document"
    );
    // an absurd diversity floor suppresses every block: if the knob
    // were dropped the two answers would be identical
    let wide = call(&mut s, 302, "check_duplication", serde_json::json!({}));
    let narrow = call(
        &mut s,
        303,
        "check_duplication",
        serde_json::json!({"min_distinct": 100000}),
    );
    assert!(
        wide != narrow,
        "min_distinct was accepted and ignored — the catalog lied"
    );
    s.finish();
}
