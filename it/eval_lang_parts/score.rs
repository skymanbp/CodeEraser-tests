//! The v2.30 language exams' scoring (booklet language-expansion.md
//! §11; registry docs/EVAL-SET-LANGS.md): one corpus's frozen sample
//! rows, resolved by the shipped ladder against the files of the frozen
//! universe the product's own walk reads (walk.rs), judged against the
//! frozen audit — the M5-2 engine (eval_graph_precision_parts: the
//! five-way verdict and its rescore) re-instantiated per language, plus
//! one line per site kind (`type_ref`, the new kind, is attributable on
//! its own). The generator (generate.rs, `#[ignore]`) and the verifier
//! the CI gate runs (precision.rs) derive through these functions (G1).

use super::walk::Walk;
use crate::common::{Fixture, reason_name};
use crate::eval_graph_precision_parts::{ratio, rescore, verdict_of};
use crate::eval_lang_parts::review::ECHO;
use crate::eval_lang_parts::{Exam, PRECISION_DOCS};
use crate::eval_support::{lang_of, tally_add};
use codeeraser::graph::ladder::{self, Outcome, Scope, Site, java_header, lua_path};
use codeeraser::graph::sites::{RawSite, detect};
use codeeraser::scan::lang::Lang;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::Path;

pub const PRECISION_SCHEMA: &str = "ce.eval-lang-precision/1.1.0";

/// The pre-registered floor (the M5-2 G2 contract, booklet §11):
/// overall, and per corpus where the in-corpus truths reach 5.
pub const GATE: f64 = 0.90;

/// Whether an exam's ladder is scored: its EXAMS row says so, and the
/// docs on disk agree (Exam::filed).
pub fn scored(exam: &Exam) -> bool {
    exam.filed(&PRECISION_DOCS, exam.scored)
}

/// The frozen tree as the walk hands it to the ladder: the frozen
/// files, each Java file's header and each Lua file's `package.path`
/// templates read the walk's way (dedup/walkidx.rs, by the product's
/// own path table), the tree's resolver configs, and no declared root
/// — the product's defaults, for no corpus carries a ce.toml.
pub fn tree(root: &Path, texts: &[(String, String)], configs: Vec<String>) -> Fixture {
    let of = |lang: Lang| {
        texts
            .iter()
            .filter(move |(p, _)| Lang::from_path(Path::new(p)) == Some(lang))
    };
    let java = of(Lang::Java)
        .map(|(p, t)| (p.clone(), java_header::read(t)))
        .collect();
    let lua = of(Lang::Lua).flat_map(|(_, t)| lua_path::read(t)).collect();
    Fixture {
        dir: root.to_path_buf(),
        files: texts.iter().map(|(p, _)| p.clone()).collect(),
        configs,
        memo: Default::default(),
        crate_roots: Default::default(),
        search_roots: Default::default(),
        java,
        lua,
    }
}

/// An outcome as the four fields every judged row and gap site carries:
/// the in-corpus answer (a package at the tree root is "."; a section
/// is `path#slug`, its slugless degrade the file), whether External
/// answered, the rung, and the refusal reason.
pub(super) fn answer(out: &Outcome) -> Value {
    let (answered, external, rung, reason) = match out {
        Outcome::Resolved { path, rung }
        | Outcome::ResolvedVia { path, rung }
        | Outcome::ResolvedInert { path, rung } => (Some(path.clone()), false, Some(*rung), None),
        Outcome::ResolvedPackage { dir, rung } => {
            let dir = if dir.is_empty() { "." } else { dir };
            (Some(dir.to_string()), false, Some(*rung), None)
        }
        Outcome::ResolvedSection { path, slug, rung } => {
            let at = slug
                .as_ref()
                .map_or_else(|| path.clone(), |s| format!("{path}#{s}"));
            (Some(at), false, Some(*rung), None)
        }
        Outcome::External { rung } => (None, true, Some(*rung), None),
        Outcome::Unresolved(r) => (None, false, None, Some(reason_name(*r))),
    };
    json!({"answered": answered, "external": external, "rung": rung, "reason": reason})
}

/// Which of the three answer shapes a row or gap site holds: an
/// in-corpus answer with its rung, External with its rung, a refusal
/// with its reason — anything else is a cooked row.
pub(super) fn shape_of(a: &Value) -> Option<&'static str> {
    match (&a["answered"], &a["external"], &a["rung"], &a["reason"]) {
        (Value::String(_), Value::Bool(false), Value::Number(_), Value::Null) => Some("answered"),
        (Value::Null, Value::Bool(true), Value::Number(_), Value::Null) => Some("external"),
        (Value::Null, Value::Bool(false), Value::Null, Value::String(_)) => Some("refused"),
        _ => None,
    }
}

/// The ladder's answer for one detected site of a frozen file.
fn resolved(path: &str, site: &RawSite, lang: &str, scope: &Scope) -> Value {
    let at = Site {
        kind: site.kind,
        from: path,
        spec: &site.spec,
        line: site.line,
    };
    answer(&ladder::resolve(lang_of(lang), &at, scope))
}

/// One judged sample row: the sampled identity, the ladder's answer and
/// the verdict against the frozen truth as the walk scores it (the M5-2
/// row shape, plus `audit_truth` when the walk rewrote the truth).
pub fn judge(row: &Value, truth: &str, scope: &Scope, walk: &Walk) -> Value {
    let lang = row["lang"].as_str().expect("lang");
    let site = Site {
        kind: row["kind"].as_str().expect("kind"),
        from: row["path"].as_str().expect("path"),
        spec: row["spec"].as_str().expect("spec"),
        line: usize::try_from(row["line"].as_u64().expect("line")).expect("line"),
    };
    judged(
        row,
        truth,
        walk,
        answer(&ladder::resolve(lang_of(lang), &site, scope)),
    )
}

/// A sample row judged on the answer given — the ladder's (judge), or
/// the tamper frame's oracle (tamper.rs).
pub(super) fn judged(row: &Value, truth: &str, walk: &Walk, mut judged: Value) -> Value {
    for field in ECHO.iter().chain(&["lang"]) {
        judged[*field] = row[*field].clone();
    }
    let scored = walk.scored(truth);
    judged["truth"] = json!(scored);
    if scored != truth {
        judged["audit_truth"] = json!(truth);
    }
    judged["verdict"] = json!(verdict_of(
        scored,
        judged["answered"].as_str(),
        judged["external"] == true
    ));
    judged
}

/// The M5-2 summary (rescore) plus one line per site kind — verdicts,
/// precision, recall — so each kind's precision is attributable.
pub fn summary(rows: &[Value]) -> Value {
    let mut kinds: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    for r in rows {
        kinds
            .entry(r["kind"].as_str().expect("kind"))
            .or_default()
            .push(r.clone());
    }
    let keep = ["verdicts", "precision", "recall", "in_corpus_truths"];
    let by_kind: Map<String, Value> = kinds
        .into_iter()
        .map(|(kind, rs)| {
            let s = rescore(&rs);
            let line: Map<String, Value> = keep
                .iter()
                .map(|k| (k.to_string(), s[*k].clone()))
                .collect();
            (kind.to_string(), line.into())
        })
        .collect();
    let mut s = rescore(rows);
    s["by_kind"] = by_kind.into();
    s
}

/// The ledger over EVERY site of the frozen universe, not the sample
/// (the M5-2 three numbers): answers per (lang, kind, rung), refusals
/// per (lang, kind, reason). Its resolution rate is a recall ceiling —
/// what the detector cannot see is in no denominator.
pub fn universe(texts: &[(String, String)], lang: &str, scope: &Scope) -> Value {
    let (mut by, mut refused) = (BTreeMap::new(), BTreeMap::new());
    for (path, text) in texts {
        for site in detect(text, lang_of(lang)) {
            let a = resolved(path, &site, lang, scope);
            let cell = format!("{lang}/{}", site.kind);
            match (a["rung"].as_u64(), a["reason"].as_str()) {
                (Some(g), _) => tally_add(&mut by, &format!("{cell}/r{g}"), 1),
                (None, reason) => tally_add(
                    &mut refused,
                    &format!("{cell}/{}", reason.expect("reason")),
                    1,
                ),
            }
        }
    }
    ledger(by, refused)
}

/// The universe ledger from its two tallies alone — the generator and
/// the gate derive the rates through this one function (G1).
/// `ratio(c, w)` is c / (c + w).
pub(super) fn ledger(by: BTreeMap<String, u64>, refused: BTreeMap<String, u64>) -> Value {
    let answered: u64 = by.values().sum();
    let r1: u64 = by
        .iter()
        .filter(|(k, _)| k.ends_with("/r1"))
        .map(|(_, n)| n)
        .sum();
    json!({
        "total_sites": answered + refused.values().sum::<u64>(),
        "resolution_rate": ratio(answered, refused.values().sum()),
        "r0_share": ratio(r1, answered - r1),
        "resolution_by": by,
        "unresolved_by": refused,
    })
}

/// Each audit site gap, answered: every site the detector reads off
/// that line and what the ladder makes of it — the auditors could not
/// see the detector, so each candidate is settled here, by the data.
pub fn gaps(review: &Value, texts: &[(String, String)], lang: &str, scope: &Scope) -> Value {
    let text: BTreeMap<&str, &str> = texts
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    let answered = |g: &Value| {
        let path = g["path"].as_str().expect("path");
        let line = usize::try_from(g["line"].as_u64().expect("line")).expect("line");
        let Some(read) = text.get(path) else {
            panic!("{path}: a site gap in a file the walk refuses")
        };
        let sites: Vec<Value> = detect(read, lang_of(lang))
            .into_iter()
            .filter(|s| s.line == line)
            .map(|s| {
                let mut row = resolved(path, &s, lang, scope);
                (row["nth"], row["kind"], row["spec"]) =
                    (json!(s.nth), json!(s.kind), json!(s.spec));
                row
            })
            .collect();
        json!({"path": path, "line": line, "sites": sites})
    };
    review["site_gaps"]
        .as_array()
        .expect("site_gaps")
        .iter()
        .map(answered)
        .collect()
}
