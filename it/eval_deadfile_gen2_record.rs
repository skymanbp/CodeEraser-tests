//! The dead-file exam, generation 2 (lane v235_licence, part 4B): the
//! gate over its committed record. The replay's ten per-corpus docs, the
//! audit of the undetermined and the coordinator's acceptance are the
//! run's and the main session's bytes, pinned here by sha256; the docs
//! answer the registry; the audit's rows are BARS-GEN2 section 3's draw
//! from the docs' undetermined events; and the registry page
//! (docs/EVAL-SET-DEADFILE-GEN2.md) carries every table the record
//! renders, through the instrument's own public functions, none changed.

use crate::common::repo_root;
use crate::eval_deadfile_gen2_parts::audit::{Row, drawn, key, reading_c_twice};
use crate::eval_deadfile_gen2_parts::doc::{SCHEMA, checked, read_all};
use crate::eval_deadfile_gen2_parts::model::Fate;
use crate::eval_deadfile_gen2_parts::readings::{Reading, pooled, share};
use crate::eval_deadfile_gen2_parts::report::{
    render, render_head, render_sides, render_unjudged, table_of,
};
use crate::eval_deadfile_gen2_parts::words::{STATES, fate_of, value, word};
use crate::eval_deadfile_gen2_parts::{dir_name, pinned, product_commit, registry};
use crate::eval_deadfile_parts::audit::{page::record_table, record::pins};
use crate::eval_deadfile_review::{page_carries, pins_hold};
use serde_json::{Value, json};

/// `sha256 path` of the record this file holds, paths under
/// `contracts/eval/`: the ten docs, the audit, the acceptance.
const RECORD: &str = "
35d909b1abe57741ab7ff3cd61ca4a22f29437145fe435a2c578435fef18907b deadfile-gen2-replay-v1.Penlight.json
ffb6622af2ec682792702db5674f1a972caa0402c072788551305280f6c1fe78 deadfile-gen2-replay-v1.fd.json
a6e342220735278e0a371e345dddf711f92d321a35d8b5bb111b0c486b473286 deadfile-gen2-replay-v1.flask.json
aa9b3993a45fd98640a90afdac0c304a635dcf85bb071dc6579eae47bcc40d2c deadfile-gen2-replay-v1.hiredis.json
145d4b048ba52ab5d3846e56d4f8e273bc45f3919210d2e13228a74307e5d0ce deadfile-gen2-replay-v1.hlint.json
a237151a8e6a183f63ae6da571392ba0ded4d4f3983f6ee1859c5848cf592d7d deadfile-gen2-replay-v1.junit4.json
091c2bae35dce6e8984339ae60f3d1f3933c9a48103c6562b01c669ffae7f37d deadfile-gen2-replay-v1.purrr.json
9640b89bc2f76a31b14ea05f79a068d3d370dc238972a66001d0b2308fe65fc0 deadfile-gen2-replay-v1.spdlog.json
bb30c78c84ec4e1d10c272094e86a0dbb3b0dde5fbb236fe25b6bbd040fe84f9 deadfile-gen2-replay-v1.viper.json
8076e1832d26d0fc83a168bef1e2f5c30a5e0c5508a41a164664dfe01d6bf029 deadfile-gen2-replay-v1.zustand.json
53a972cb1982981727c905ce5a347b3ed6a078d4d4a1a830ee27a8d101985acc deadfile-gen2-audit-v1.json
25412e628c9409512592299870585b8d9957b6658a10e4e9ead445bd4b805948 deadfile-gen2-acceptance-v1.md
";

const AUDIT: &str = "contracts/eval/deadfile-gen2-audit-v1.json";
const PAGE: &str = "docs/EVAL-SET-DEADFILE-GEN2.md";
const BARS: &str = "deadfile-gen2-bars-v1.md";
/// An auditor's three words.
const ANSWERS: [&str; 3] = ["in_use", "unused", "unsure"];

fn docs() -> Vec<Value> {
    read_all(&repo_root().join("contracts/eval"))
}

fn audit_rows() -> Vec<Value> {
    let doc = crate::eval_support::load(&repo_root().join(AUDIT).to_string_lossy());
    doc["rows"].as_array().cloned().expect("audit rows")
}

#[test]
fn the_record_is_the_runs_bytes() {
    pins_hold(&pins(RECORD));
}

#[test]
fn the_docs_answer_the_registry() {
    let (reg, docs) = (registry(), docs());
    let rows = reg["corpora"].as_array().expect("corpora");
    let chosen: Vec<&str> = rows.iter().map(dir_name).collect();
    assert_eq!(docs.len(), chosen.len(), "one committed doc per corpus");
    checked(&docs, &chosen).unwrap_or_else(|e| panic!("the committed docs: {e}"));
    let product = product_commit(&reg).expect("the registry's product commit");
    for row in rows {
        let name = dir_name(row);
        let d = docs.iter().find(|d| d["corpus"] == name).expect("its doc");
        assert_eq!(d["schema"], SCHEMA, "{name}: schema");
        assert_eq!(d["tip"], row["tip"], "{name}: not the registry's tip");
        assert_eq!(d["product"]["commit"], product, "{name}: product commit");
        assert_eq!(row["tip_judged_by_product"], "judged", "{name}: tip");
    }
}

/// The docs' undetermined events, by their identity.
fn undetermined(docs: &[Value]) -> Vec<Row> {
    let mut out = Vec::new();
    for d in docs {
        for e in d["events"].as_array().into_iter().flatten() {
            let s = |k: &str| e[k].as_str().unwrap_or_else(|| panic!("event without {k}"));
            if fate_of(s("fate")) == Fate::Undetermined {
                out.push(Row {
                    corpus: d["corpus"].as_str().expect("corpus").into(),
                    commit: s("commit").into(),
                    path: s("path").into(),
                    state: value(&STATES, "state", s("state")),
                });
            }
        }
    }
    out
}

/// Section 3's draw: per state in key order, `n` counting from 1.
fn expected(docs: &[Value]) -> Vec<(usize, Row)> {
    let rows = undetermined(docs);
    STATES
        .iter()
        .flat_map(|(_, s)| {
            drawn(&rows, *s)
                .into_iter()
                .enumerate()
                .map(|(i, r)| (i + 1, r))
        })
        .collect()
}

fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap_or("")
}

/// One reading: one of the three words; `in_use` cites evidence,
/// `unused` says where it searched.
fn reading_ok(r: &Value) -> Result<(), String> {
    let answer = text(r, "answer");
    match answer {
        "in_use" if text(r, "evidence").trim().is_empty() => Err("in_use without evidence".into()),
        "unused" if text(r, "searched").trim().is_empty() => Err("unused without a search".into()),
        a if ANSWERS.contains(&a) => Ok(()),
        a => Err(format!("answer {a:?}")),
    }
}

/// The settled answer follows from the readings: one reading or two
/// equal ones stand; two that differ are settled by the evidence, named.
fn settled_ok(row: &Value) -> Result<(), String> {
    let (first, settled) = (text(&row["first"], "answer"), text(row, "settled"));
    let second = row["second"]
        .is_object()
        .then(|| text(&row["second"], "answer"));
    match second {
        Some(s) if s != first => match ANSWERS.contains(&settled) {
            true if !text(row, "settled_by").trim().is_empty() => Ok(()),
            _ => Err(format!(
                "{first} / {s} settled {settled:?} without settled_by"
            )),
        },
        _ if settled == first => Ok(()),
        _ => Err(format!("settled {settled:?}, the reading says {first}")),
    }
}

/// One audit row against its draw.
fn row_ok(row: &Value, (n, r): &(usize, Row), docs: &[Value]) -> Result<(), String> {
    let tip = docs
        .iter()
        .find(|d| d["corpus"] == r.corpus.as_str())
        .map(|d| &d["tip"]);
    let mine = json!({"n": n, "state": word(&STATES, r.state), "corpus": r.corpus,
                      "commit": r.commit, "path": r.path, "key": key(r)});
    for k in ["n", "state", "corpus", "commit", "path", "key"] {
        if row[k] != mine[k] {
            return Err(format!("`{k}` is {}, the draw says {}", row[k], mine[k]));
        }
    }
    if tip != Some(&row["tip"]) {
        return Err("not the corpus doc's tip".into());
    }
    let twice = n % 5 == 0 || text(&row["first"], "answer") == "unsure";
    if row["second"].is_object() != twice {
        return Err(format!("second reading present: {}, want {twice}", !twice));
    }
    reading_ok(&row["first"])?;
    if twice {
        reading_ok(&row["second"])?;
    }
    settled_ok(row)
}

#[test]
fn the_audit_follows_section_3() {
    let (docs, rows) = (docs(), audit_rows());
    let want = expected(&docs);
    assert_eq!(rows.len(), want.len(), "audit rows against the draw");
    let bad: Vec<String> = rows
        .iter()
        .zip(&want)
        .filter_map(|(row, w)| {
            row_ok(row, w, &docs)
                .err()
                .map(|e| format!("row {}: {e}", w.0))
        })
        .collect();
    assert!(bad.is_empty(), "the audit:\n{}", bad.join("\n"));
}

/// `k/n (x.x %)`, the report's percentage; `-` when there is no share.
fn frac(c: Option<(usize, usize)>) -> String {
    match c {
        Some((k, n)) if n > 0 => format!("{k}/{n} ({:.1} %)", 100.0 * k as f64 / n as f64),
        Some((k, n)) => format!("{k}/{n} (0.0 %)"),
        None => "-".into(),
    }
}

/// One state's row of reading C.
fn state_row(fates: &[Fate], name: &str, mine: &[&Value]) -> String {
    let undetermined = fates.iter().filter(|f| **f == Fate::Undetermined).count();
    let twice = mine.iter().filter(|r| r["second"].is_object()).count();
    let n = |w: &str| mine.iter().filter(|r| r["settled"] == w).count();
    let (in_use, unused, unsure) = (n("in_use"), n("unused"), n("unsure"));
    let c = reading_c_twice(fates, in_use, unsure, mine.len());
    format!(
        "| {name} | {undetermined} | {} | {twice} | {in_use} | {unused} | {unsure} | {} | {} | {} |",
        mine.len(),
        frac(Some(share(Reading::B, fates))),
        frac(c[0]),
        frac(c[1])
    )
}

/// Reading C as a pure function of the docs and the audit.
fn reading_c_block(docs: &[Value], audit: &[Value]) -> String {
    let t = table_of(&json!({"corpora": docs}));
    let mut out = vec![
        "| state | undetermined | drawn | read twice | in_use | unused | unsure | reading B | reading C, unsure as in_use | reading C, unsure as not |".to_string(),
        "|---|---|---|---|---|---|---|---|---|---|".into(),
    ];
    for (name, state) in STATES {
        let mine: Vec<&Value> = audit.iter().filter(|r| r["state"] == name).collect();
        out.push(state_row(&pooled(&t, state, None, &|_| true), name, &mine));
    }
    out.push(String::new());
    out.push("| n | state | corpus | commit | path | first | second | settled |".into());
    out.push("|---|---|---|---|---|---|---|---|".into());
    for r in audit {
        let second = r["second"]
            .is_object()
            .then(|| text(&r["second"], "answer"));
        out.push(format!(
            "| {} | {} | {} | `{}` | `{}` | {} | {} | {} |",
            r["n"],
            text(r, "state"),
            text(r, "corpus"),
            &text(r, "commit")[..10],
            text(r, "path"),
            text(&r["first"], "answer"),
            second.unwrap_or("-"),
            text(r, "settled")
        ));
    }
    out.join("\n")
}

/// The registry's replaced rows, in its own words.
fn replaced_rows(reg: &Value) -> String {
    reg["corpora"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| !r["replaced_because"].is_null())
        .map(|r| {
            let (l, f, c) = (text(r, "language"), text(r, "first"), text(r, "chosen"));
            format!("{l}: {f} -> {c}: {}", text(r, "replaced_because"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every block the page carries, named, in page order.
fn rendered() -> Vec<(&'static str, String)> {
    let docs = docs();
    let record: Vec<(&str, &str)> = pinned().into_iter().chain(pins(RECORD)).collect();
    let run = json!({"corpora": docs});
    vec![
        ("record_table", record_table(&record)),
        ("render_head", render_head(&docs)),
        ("replaced_rows", replaced_rows(&registry())),
        ("render_unjudged", render_unjudged(&docs)),
        ("render", render(&table_of(&run))),
        ("render_sides", render_sides(&docs)),
        ("reading_c", reading_c_block(&docs, &audit_rows())),
    ]
}

#[test]
fn the_page_states_the_committed_numbers() {
    let bars = crate::eval_deadfile_parts::audit::record::text(BARS);
    page_carries(
        PAGE,
        rendered().into_iter().map(|(_, b)| b).collect(),
        ("## The bars, as frozen", "## Before any count"),
        &[bars],
    );
}

/// The blocks for the page writer, each after a `=== block` line.
#[test]
#[ignore = "prints the generation-2 registry page's tables for the page writer"]
fn gen2_registry_tables() {
    for (name, block) in rendered() {
        println!("=== block {name}\n{block}");
    }
}
