//! The same-role advisor's three faces (plan v2.29 step 6, spec §六)
//! agree on ONE document: `ce similar --format json` prints the
//! library face byte for byte, the MCP tool `similar_units` relays it,
//! and the Stop audit's leg writes its `similar` object into the feed
//! for the units a session added. The fixture is built to be judged:
//! `fetch_user_row` and `load_user_row` share two name words at equal
//! shape — the core's second role arm — while eight unrelated units
//! keep every shared term under the idf-zero line.

use crate::common;
use codeeraser::similar::query::Ask;
use serde_json::Value;

const A: &str = "/// Fetch the user row by id.\nfn fetch_user_row(id: u64) -> u64 {\n    id\n}\n";
const B: &str = "/// Load the user row by id.\nfn load_user_row(id: u64) -> u64 {\n    id + 1\n}\n";
const FILLER: &str = "fn z0() {}\nfn z1() {}\nfn z2() {}\nfn z3() {}\nfn z4() {}\nfn z5() {}\nfn z6() {}\nfn z7() {}\n";

/// `a.rs` + the filler committed; `b.rs` written and staged as the
/// session's addition (the Stop leg's "new unit").
fn seeded(name: &str) -> std::path::PathBuf {
    let dir = common::tmp(name);
    std::fs::write(dir.join("a.rs"), A).expect("a.rs");
    std::fs::write(dir.join("z.rs"), FILLER).expect("z.rs");
    common::init_and_commit(&dir, "seed");
    std::fs::write(dir.join("b.rs"), B).expect("b.rs");
    common::git(&dir, &["add", "b.rs"]);
    dir
}

fn face(dir: &std::path::Path, ask: &Ask, widen: bool) -> Value {
    codeeraser::faces::similar(dir, &common::core_bin(), ask, widen).expect("face")
}

fn cli_doc(dir: &std::path::Path, args: &[&str]) -> Value {
    let core = common::core_bin();
    let mut full = vec!["similar", ".", "--core", &core, "--format", "json"];
    full.extend_from_slice(args);
    let out = common::run_ce(dir, &full);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("json")
}

#[test]
fn the_cli_prints_the_library_face_and_the_core_judged_the_pair_same_role() {
    let dir = seeded("similar-cli");
    let doc = cli_doc(&dir, &["--at", "b.rs:2"]);
    assert_eq!(doc, face(&dir, &Ask::at("b.rs:2").unwrap(), false));
    assert_eq!(doc["schema"], "ce.similar-report/0.1.0");
    assert_eq!(doc["query"]["label"], "b.rs:2-4 load_user_row/1");
    assert!(doc["degraded"].is_null(), "{doc}");
    let top = &doc["candidates"][0];
    assert_eq!(top["key"], "fetch_user_row/1");
    assert_eq!(top["at"], "a.rs:2-4");
    assert_eq!(top["role"], true, "two name words at equal shape: {top}");
    assert_eq!(top["hits"][0], 2, "N = user, row");
    assert_eq!(top["shape_equal"], true);
    assert_eq!(doc["counts"]["role"], 1);
    // the same unit by key, and as free text: one document road
    assert_eq!(
        cli_doc(&dir, &["--unit", "load_user_row/1"])["candidates"],
        doc["candidates"]
    );
    let text = cli_doc(&dir, &["--text", "load the user row"]);
    assert_eq!(text["query"]["label"], "text: load the user row");
    assert_eq!(
        text["candidates"][0]["role"], false,
        "a text has no shape and no callee"
    );
    // console face: the header names the count the document carries
    let out = common::run_ce(
        &dir,
        &[
            "similar",
            ".",
            "--core",
            &common::core_bin(),
            "--at",
            "b.rs:2",
        ],
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.starts_with("similar: b.rs:2-4 load_user_row/1 —"),
        "{text}"
    );
    assert!(
        text.contains("1 same-role") && text.contains("a.rs:2-4 fetch_user_row/1  N2"),
        "{text}"
    );
}

/// The associative view adds rows and tags them; every bare row keeps
/// its place and its tag off. The ask group refuses two asks and none.
#[test]
fn widen_tags_what_only_the_widened_query_reaches_and_the_ask_is_exactly_one() {
    let dir = seeded("similar-widen");
    let bare = cli_doc(&dir, &["--at", "b.rs:2"]);
    let wide = cli_doc(&dir, &["--at", "b.rs:2", "--widen"]);
    assert_eq!(wide["query"]["widen"], true);
    let (bare_rows, wide_rows) = (
        bare["candidates"].as_array().expect("rows"),
        wide["candidates"].as_array().expect("rows"),
    );
    assert_eq!(
        &wide_rows[..bare_rows.len()],
        &bare_rows[..],
        "bare rows lead, untouched"
    );
    assert!(bare_rows.iter().all(|r| r["widened"] == false));
    assert!(
        wide_rows[bare_rows.len()..]
            .iter()
            .all(|r| r["widened"] == true)
    );
    assert_eq!(
        wide["counts"]["widened"],
        (wide_rows.len() - bare_rows.len()) as u64
    );
    for bad in [&["--at", "b.rs:2", "--text", "x"][..], &[][..]] {
        let mut args = vec!["similar", "."];
        args.extend_from_slice(bad);
        let out = common::run_ce(&dir, &args);
        assert!(!out.status.success(), "{bad:?}");
    }
    let miss = common::run_ce(
        &dir,
        &[
            "similar",
            ".",
            "--core",
            &common::core_bin(),
            "--at",
            "a.rs:1",
        ],
    );
    assert_eq!(miss.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&miss.stderr).contains("no indexed unit at a.rs:1"));
}

#[test]
fn the_mcp_tool_relays_the_face_and_refuses_an_ask_that_is_not_one() {
    let dir = seeded("similar-mcp");
    // warm once so both faces read the same index state (the
    // mcp_precommit refresh-counter lesson)
    codeeraser::dedup::analyze(&dir, None, None, None).expect("warm");
    let mut s = common::McpSession::over(&dir);
    let got = s.call(
        1,
        "similar_units",
        serde_json::json!({"at": "b.rs:2", "widen": true}),
    );
    assert_eq!(got["isError"], false, "{got}");
    assert_eq!(
        got["content"][0]["text"],
        face(&dir, &Ask::at("b.rs:2").unwrap(), true).to_string()
    );
    let text = s.call(
        2,
        "similar_units",
        serde_json::json!({"text": "fetch user"}),
    );
    let doc: Value =
        serde_json::from_str(text["content"][0]["text"].as_str().expect("text")).expect("json");
    assert_eq!(doc["query"]["label"], "text: fetch user");
    let none = s.call(3, "similar_units", serde_json::json!({}));
    assert_eq!(none["isError"], true);
    assert!(
        none["content"][0]["text"]
            .as_str()
            .expect("why")
            .contains("exactly one")
    );
    s.finish();
}

/// The Stop leg: the session added `load_user_row`; its top-1 is
/// same-role, so the feed line carries the advisor's row — and a
/// session that added nothing carries no `similar` key at all.
#[test]
fn the_stop_audit_writes_the_advisors_row_for_a_new_same_role_unit() {
    // no ce.toml: the audit class is not promoted, unset mode stays
    // observe, and the fixture's pair stays the fixture's (seed_project
    // would overwrite a.rs with its own clone pair)
    let dir = seeded("similar-stop");
    let line = common::stop_observe(&dir);
    assert_eq!(line["schema"], codeeraser::hookio::OBSERVE_SCHEMA);
    let sim = &line["similar"];
    assert_eq!(sim["rev"], codeeraser::similar::SIMILAR_REV);
    assert_eq!(sim["new_units"], 1);
    assert_eq!(sim["queried"], 1);
    assert!(sim.get("degraded").is_none(), "{sim}");
    assert_eq!(sim["rows"][0]["unit"], "b.rs:2");
    assert_eq!(sim["rows"][0]["twin"], "a.rs:2-4");
    assert!(sim["rows"][0]["score"].as_i64().expect("score") > 0);
    // committed: nothing added since HEAD, nothing to say
    common::commit_all(&dir, "b");
    let quiet = common::stop_observe(&dir);
    assert!(quiet.get("similar").is_none(), "{quiet}");
}
