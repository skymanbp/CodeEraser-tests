//! The sample half of the similar replay: a deterministic stratified
//! draw over the ranked units, the frozen-doc envelope, and the
//! arbiter's packet. Split from similar_replay.rs at the file budget.

use crate::common;
use crate::eval_support::content_sha;
use crate::similar_replay::{CORPORA, K, Measured};
use codeeraser::similar::bm25::{self, Doc, Hit};
use codeeraser::similar::{Channel, SIMILAR_REV, docs, ppmi};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Sample quotas per corpus under the BARE arm's top-1 role bit:
/// (role = 1 queries, role = 0 queries). The self corpus carries the
/// bulk; each fixture tree contributes a stratum of its own so no
/// language is judged on the self corpus alone. Every stratum is a
/// prefix of the sha256 rank order, so the draw is reproducible from
/// the doc and nobody chooses their own queries.
pub const QUOTA_SELF: (usize, usize) = (35, 35);
pub const QUOTA_FIXTURE: (usize, usize) = (7, 6);

/// The rank of one query: sha256 over domain | corpus | identity.
fn rank(corpus: &str, path: &str, key: &str, nth: i64) -> String {
    content_sha(&format!("similar|{corpus}|{path}|{key}|{nth}"))
}

fn ext(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or("")
}

/// The identity fields every unit row carries: where it lives, its
/// file's content sha (so a later gate can tell a row whose file still
/// reads the same from one whose file moved on), and its span.
fn identity(m: &Measured, d: &Doc) -> serde_json::Map<String, Value> {
    let v = json!({
        "path": d.path, "key": d.bag.key, "nth": d.bag.nth,
        "sha": content_sha(&m.texts[&d.path]),
        "start_line": d.bag.start_line, "end_line": d.bag.end_line,
    });
    v.as_object().expect("object").clone()
}

/// One arm's placement of a candidate: rank (1-based) and score.
fn placement(arm: &[Hit], doc: usize) -> Value {
    arm.iter().position(|h| h.doc == doc).map_or(
        Value::Null,
        |i| json!({"rank": i + 1, "score": arm[i].score}),
    )
}

/// One candidate's row: identity plus evidence integers, shape
/// equality, the role bit, same-file, and each arm's placement.
fn candidate(m: &Measured, d: &Doc, c: usize, bare: &[Hit], wide: &[Hit]) -> Value {
    let h = bare
        .iter()
        .chain(wide.iter())
        .find(|h| h.doc == c)
        .expect("in one arm");
    let cd = &m.corpus.docs[c];
    let mut row = identity(m, cd);
    row.extend(
        json!({
            "hits": h.hits, "shape_equal": h.shape_equal, "role": h.role,
            "same_file": cd.path == d.path,
            "bare": placement(bare, c), "widened": placement(wide, c),
        })
        .as_object()
        .expect("object")
        .clone(),
    );
    Value::Object(row)
}

/// The row of one sampled query: identity, bag sizes per channel,
/// and the union of both arms' candidates in bare order then
/// widened-only order, each with its evidence integers.
pub fn row(m: &Measured, i: usize) -> Value {
    let d = &m.corpus.docs[i];
    let (bare, wide) = &m.ranked[i];
    let mut order: Vec<usize> = bare.iter().map(|h| h.doc).collect();
    let widened_only: Vec<usize> = wide
        .iter()
        .map(|h| h.doc)
        .filter(|c| !order.contains(c))
        .collect();
    order.extend(widened_only);
    let candidates: Vec<Value> = order
        .iter()
        .map(|&c| candidate(m, d, c, bare, wide))
        .collect();
    let sizes: BTreeMap<&str, usize> = Channel::ALL
        .iter()
        .map(|ch| (ch.label(), d.bag.channel(*ch).len()))
        .collect();
    let mut row = identity(m, d);
    row.extend(
        json!({
            "rank": rank(m.name, &d.path, &d.bag.key, d.bag.nth),
            "corpus": m.name, "lang": ext(&d.path),
            "bag": sizes,
            "role_bare": bare.first().is_some_and(|h| h.role),
            "candidates": candidates,
        })
        .as_object()
        .expect("object")
        .clone(),
    );
    Value::Object(row)
}

/// The stratified draw: per corpus, the rank-order prefix of answered
/// queries whose bare top-1 carries the role bit, and of those whose
/// top-1 does not, up to the quotas. Returns the rows (rank-sorted)
/// and the per-corpus counts actually drawn (a short stratum is
/// reported, never padded from the other).
pub fn sample(measured: &[Measured]) -> (Vec<Value>, Value) {
    let mut rows = Vec::new();
    let mut drawn = serde_json::Map::new();
    for m in measured {
        let quota = if m.name == "self" {
            QUOTA_SELF
        } else {
            QUOTA_FIXTURE
        };
        let mut order: Vec<usize> = (0..m.corpus.docs.len()).collect();
        order.sort_by_cached_key(|&i| {
            let d = &m.corpus.docs[i];
            rank(m.name, &d.path, &d.bag.key, d.bag.nth)
        });
        let mut took = [0usize; 2];
        for i in order {
            let Some(top) = m.ranked[i].0.first() else {
                continue;
            };
            let stratum = usize::from(!top.role);
            let cap = if stratum == 0 { quota.0 } else { quota.1 };
            if took[stratum] < cap {
                took[stratum] += 1;
                rows.push(row(m, i));
            }
        }
        drawn.insert(m.name.into(), json!({"role1": took[0], "role0": took[1]}));
    }
    rows.sort_by_cached_key(|r| r["rank"].as_str().expect("rank").to_string());
    (rows, Value::Object(drawn))
}

/// Every constant the measurement was taken under — a change to any
/// of them is a different instrument and a different doc.
pub fn constants() -> Value {
    json!({
        "similar_rev": SIMILAR_REV, "k": K,
        "k1": [bm25::K1.0, bm25::K1.1], "b": [bm25::B.0, bm25::B.1],
        "idf_frac_bits": bm25::IDF_FRAC_BITS, "score_frac_bits": bm25::SCORE_FRAC_BITS,
        "w_unit": bm25::W_UNIT,
        "top_m": ppmi::TOP_M, "min_cooc": ppmi::MIN_COOC, "min_ppmi": ppmi::MIN_PPMI,
        "ppmi_cap": ppmi::PPMI_CAP, "ppmi_scale": ppmi::PPMI_SCALE, "term_cap": ppmi::TERM_CAP,
        "lead_gap": docs::LEAD_GAP, "head_gap": docs::HEAD_GAP,
        "quota_self": [QUOTA_SELF.0, QUOTA_SELF.1],
        "quota_fixture": [QUOTA_FIXTURE.0, QUOTA_FIXTURE.1],
        "corpora": CORPORA.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
    })
}

/// The frozen doc envelope: constants, the tree it was measured on,
/// the per-corpus tally, the draw, the rows.
pub fn document(measured: &[Measured], summary: Value, rows: Vec<Value>) -> Value {
    let root = common::repo_root();
    let (_, head) = common::git_out(&root, &["rev-parse", "HEAD"]);
    let (_, status) = common::git_out(&root, &["status", "--porcelain"]);
    json!({
        "constants": constants(),
        "corpus": {"tip": head.trim(), "dirty": !status.trim().is_empty()},
        "generated_from": {"ce": env!("CARGO_PKG_VERSION")},
        "method": "every unit of each corpus (the product's unitsig universe, bagged from the text the index saw) queries the rest of its corpus; bare arm = the unit's own bag at channel weights, widened arm = the same plus each spelled word term's top-m PPMI neighbours at a capped fraction of its weight; integer BM25, fixed-point idf; role = (N ≥ 1 ∧ C ≥ 1) ∨ (N ≥ 2 ∧ shape equal). Sample = per corpus the sha256-rank prefix of role = 1 and of role = 0 bare top-1 queries up to the quotas; candidates = the union of both arms' top-k with one evidence row each. Arbitration labels every candidate; precision at 1 and hit at 5 are computed over the arbitrated rows only — no oracle here knows every same-role partner, so recall has no denominator and is not claimed.",
        "summary": summary,
        "units": measured.iter().map(|m| (m.name, m.corpus.docs.len())).collect::<BTreeMap<_, _>>(),
        "rows": rows,
    })
}

/// The arbiter's packet: for every sampled row, the query's source
/// lines and each candidate's, verbatim, under the row's rank — texts
/// only, no scores, no role bits (RM18).
pub fn packet(measured: &[Measured], doc: &Value) -> String {
    let by_name: BTreeMap<&str, &Measured> = measured.iter().map(|m| (m.name, m)).collect();
    let mut out = String::new();
    for r in doc["rows"].as_array().expect("rows") {
        let m = by_name[r["corpus"].as_str().expect("corpus")];
        let _ = writeln!(
            out,
            "=== {} ({})",
            r["rank"].as_str().expect("rank"),
            m.name
        );
        excerpt(&mut out, m, r, "QUERY");
        for (i, c) in r["candidates"]
            .as_array()
            .expect("candidates")
            .iter()
            .enumerate()
        {
            excerpt(&mut out, m, c, &format!("CANDIDATE {}", i + 1));
        }
    }
    out
}

fn excerpt(out: &mut String, m: &Measured, unit: &Value, head: &str) {
    let (path, key) = (
        unit["path"].as_str().expect("path"),
        unit["key"].as_str().expect("key"),
    );
    let (s, e) = (line_of(unit, "start_line"), line_of(unit, "end_line"));
    let _ = writeln!(out, "--- {head} {path}:{s}-{e} {key}#{}", unit["nth"]);
    let lines: Vec<&str> = m.texts[path].lines().collect();
    for line in &lines[s - 1..e.min(lines.len())] {
        let _ = writeln!(out, "{line}");
    }
}

fn line_of(unit: &Value, field: &str) -> usize {
    unit[field].as_u64().expect(field) as usize
}
