//! BARS.md §4 — which unreferenced files may fail `ce deadcode --check`
//! — over the review rows, under every reading the addenda require:
//! classifier K1 and K1-strict (addendum 2), population A (audited rows
//! less those carrying a passing §3 convention) and B (the rows the
//! product still reports dead once every passing convention is an
//! entry; addendum 1 A), `in_use` counted all or `named` only (addendum
//! 1 B). An outcome ships only when the four (classifier, population)
//! readings agree; otherwise it is an owner question with every set of
//! numbers. Shares are weighted by pool / primary of the row's
//! (language, corpus) (BARS §1).

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// (in floor, project floor, rows per project, projects, class floor, split gap, failing question)
pub const BARS: (f64, f64, usize, usize, usize, f64, f64) = (0.80, 0.60, 10, 2, 30, 0.25, 0.50);
pub const CLASSES: [&str; 3] = ["runs-alone", "declares", "document"];
/// (classifier key in a row, label)
pub const CLASSIFIERS: [&str; 2] = ["k1", "k1_strict"];

/// One audited row as the evaluator reads it.
#[derive(Clone, Debug)]
pub struct Row {
    pub corpus: String,
    pub path: String,
    pub lang: String,
    pub k1: String,
    pub k1_strict: String,
    pub weight: f64,
    /// final answer: in_use / unused / unsure
    pub answer: String,
    /// addendum 1 B tag of an in_use answer
    pub named: bool,
}

/// Weighted share of `rows` answered in use (`named` only when asked).
fn share(rows: &[&Row], named_only: bool, word: &str) -> f64 {
    let total: f64 = rows.iter().map(|r| r.weight).sum();
    let hit: f64 = rows
        .iter()
        .filter(|r| r.answer == word && (word != "in_use" || !named_only || r.named))
        .map(|r| r.weight)
        .sum();
    // an empty f64 sum is -0.0; adding +0.0 prints a share of nothing as 0.000
    if total == 0.0 { 0.0 } else { hit / total + 0.0 }
}

/// One class's per-project counts and shares and its BARS §4 verdict.
fn verdict_of(rows: &[&Row], named_only: bool) -> (BTreeMap<String, Value>, &'static str) {
    let (floor, pfloor, prows, projects, cfloor, _, _) = BARS;
    let mut by: BTreeMap<String, Vec<&Row>> = BTreeMap::new();
    for r in rows {
        by.entry(r.corpus.clone()).or_default().push(r);
    }
    let per: BTreeMap<String, Value> = by
        .into_iter()
        .map(|(p, rs)| {
            (
                p,
                json!({"rows": rs.len(), "in": share(&rs, named_only, "in_use")}),
            )
        })
        .collect();
    let big: Vec<&Value> = per
        .values()
        .filter(|v| v["rows"].as_u64() >= Some(prows as u64))
        .collect();
    let advisory = share(rows, named_only, "in_use") >= floor
        && big.len() >= projects
        && big.iter().all(|v| v["in"].as_f64() >= Some(pfloor));
    let verdict = match (rows.len() < cfloor, advisory) {
        (true, _) => "not judged",
        (false, true) => "ADVISORY",
        (false, false) => "FAILING",
    };
    (per, verdict)
}

/// One class under one reading: counts, shares, per project, verdict,
/// and leave-one-project-out — per project the share and the verdict of
/// the class with that project's rows taken out.
fn judge(rows: &[&Row], named_only: bool) -> Value {
    let (per, verdict) = verdict_of(rows, named_only);
    let lopo: BTreeMap<&String, Value> = per
        .keys()
        .map(|p| {
            let rest: Vec<&Row> = rows.iter().copied().filter(|r| r.corpus != *p).collect();
            (p, json!({"in": share(&rest, named_only, "in_use"), "rows": rest.len(), "verdict": verdict_of(&rest, named_only).1}))
        })
        .collect();
    json!({"rows": rows.len(), "in": share(rows, named_only, "in_use"), "un": share(rows, named_only, "unused"), "per_project": per, "verdict": verdict, "leave_one_out": lopo})
}

/// BARS §4's "what ships" for the code classes, from two verdicts.
fn ships(classes: &Value) -> String {
    let (run, dec) = (&classes["runs-alone"], &classes["declares"]);
    let inn = |j: &Value| j["in"].as_f64().unwrap_or(0.0);
    match (
        run["verdict"].as_str().unwrap_or(""),
        dec["verdict"].as_str().unwrap_or(""),
    ) {
        ("ADVISORY", "ADVISORY") => {
            "no split: one advisory word for every unreferenced code file".into()
        }
        ("ADVISORY", _) if inn(dec) <= inn(run) - BARS.5 => {
            "the split ships: runs-alone advisory, declares keeps failing".into()
        }
        ("ADVISORY", _) => "nothing ships: owner question about the gate's default".into(),
        ("FAILING", _) => "nothing ships: runs-alone failing".into(),
        _ => "nothing ships: runs-alone not judged".into(),
    }
}

/// One (classifier, population) reading: every class, the code outcome,
/// the document outcome, the questions, the named-only notes.
fn reading(rows: &[&Row], classifier: &str) -> Value {
    let class_of = |r: &Row| {
        if classifier == "k1" {
            r.k1.clone()
        } else {
            r.k1_strict.clone()
        }
    };
    let mut classes = serde_json::Map::new();
    let mut notes = Vec::new();
    let mut questions = Vec::new();
    for c in CLASSES {
        let mine: Vec<&Row> = rows.iter().copied().filter(|r| class_of(r) == c).collect();
        let (all, named) = (judge(&mine, false), judge(&mine, true));
        if all["verdict"] == "ADVISORY" && named["verdict"] != "ADVISORY" {
            notes.push(format!(
                "{c}: ADVISORY with every in_use answer, not with named ones only"
            ));
        }
        if all["verdict"] == "FAILING" && all["in"].as_f64() >= Some(BARS.6) {
            questions.push(format!(
                "{c}: FAILING with in = {:.3} (the failing verdict is wrong more often than right)",
                all["in"].as_f64().unwrap_or(0.0)
            ));
        }
        classes.insert(c.into(), json!({"all": all, "named": named}));
    }
    let all_of = |c: &str| classes[c]["all"].clone();
    let code = ships(&json!({"runs-alone": all_of("runs-alone"), "declares": all_of("declares")}));
    let document = all_of("document")["verdict"]
        .as_str()
        .unwrap_or("")
        .to_string();
    json!({"classes": classes, "code": code, "document": document, "questions": questions, "named_notes": notes})
}

/// The whole evaluation: every reading, the agreed outcomes, the
/// per-language precision of today's "dead" word (`un(all)`).
pub fn evaluate(
    rows: &[Row],
    passing: &BTreeSet<(String, String)>,
    still_dead_b: Option<&BTreeSet<(String, String)>>,
) -> Value {
    let id = |r: &Row| (r.corpus.clone(), r.path.clone());
    let a: Vec<&Row> = rows.iter().filter(|r| !passing.contains(&id(r))).collect();
    let b: Option<Vec<&Row>> =
        still_dead_b.map(|s| rows.iter().filter(|r| s.contains(&id(r))).collect());
    let mut readings = serde_json::Map::new();
    for classifier in CLASSIFIERS {
        readings.insert(format!("{classifier}/A"), reading(&a, classifier));
        if let Some(b) = &b {
            readings.insert(format!("{classifier}/B"), reading(b, classifier));
        }
    }
    let agreed = |key: &str| {
        let set: BTreeSet<String> = readings
            .values()
            .map(|r| r[key].as_str().unwrap_or("").to_string())
            .collect();
        match (set.len(), b.is_some()) {
            (1, true) => json!({"outcome": set.into_iter().next(), "agreed": true}),
            (_, false) => {
                json!({"outcome": "owner question: population B not computed", "agreed": false})
            }
            _ => {
                json!({"outcome": "owner question: the readings differ", "agreed": false, "by_reading": readings.iter().map(|(k, r)| (k.clone(), r[key].clone())).collect::<BTreeMap<_, _>>()})
            }
        }
    };
    let mut precision: BTreeMap<&str, f64> = BTreeMap::new();
    for lang in rows
        .iter()
        .map(|r| r.lang.as_str())
        .collect::<BTreeSet<_>>()
    {
        let mine: Vec<&Row> = rows.iter().filter(|r| r.lang == lang).collect();
        precision.insert(lang, share(&mine, false, "unused"));
    }
    json!({"readings": readings, "code": agreed("code"), "document": agreed("document"), "un_all_by_language": precision})
}

/// The printed report: one line per class per reading, with the
/// leave-one-project-out shares and the agreed outcomes.
pub fn render(report: &Value) -> String {
    let mut out = Vec::new();
    for (key, r) in report["readings"].as_object().into_iter().flatten() {
        out.push(format!(
            "== {key}: code -> {}; document -> {}",
            r["code"], r["document"]
        ));
        for c in CLASSES {
            for mode in ["all", "named"] {
                let j = &r["classes"][c][mode];
                let f = |v: &Value, k: &str| v[k].as_f64().unwrap_or(0.0);
                let per = |k: &str, show: &dyn Fn(&Value) -> String| -> String {
                    let each = j[k].as_object().into_iter().flatten();
                    each.map(|(p, v)| format!("{p} {}", show(v)))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                out.push(format!(
                    "  {c:<10} {mode:<5} rows {:>3} in {:.3} un {:.3} {} | per project: {} | leave one out: {}",
                    j["rows"], f(j, "in"), f(j, "un"), j["verdict"],
                    per("per_project", &|v| format!("{}/{:.3}", v["rows"], f(v, "in"))),
                    per("leave_one_out", &|v| format!("{:.3} {}", f(v, "in"), v["verdict"].as_str().unwrap_or("")))
                ));
            }
        }
        for q in r["questions"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(r["named_notes"].as_array().into_iter().flatten())
        {
            out.push(format!("  note: {q}"));
        }
    }
    out.push(format!("code outcome: {}", report["code"]));
    out.push(format!("document outcome: {}", report["document"]));
    out.push(format!(
        "un(all) by language: {}",
        report["un_all_by_language"]
    ));
    out.join("\n")
}

/// Per class, whether its verdict is the same under all readings (every
/// classifier x population x in_use mode present): "yes (<verdict>)", or
/// every reading that differs from k1/A/all with its verdict and the
/// difference of its in-share from k1/A/all's.
pub fn same_under_all(report: &Value) -> Vec<String> {
    let readings = report["readings"].as_object().cloned().unwrap_or_default();
    CLASSES
        .iter()
        .map(|c| {
            let cell = |key: &str, mode: &str| &readings[key]["classes"][*c][mode];
            let base = cell("k1/A", "all").clone();
            let inn = |j: &Value| j["in"].as_f64().unwrap_or(0.0);
            let differ: Vec<String> = readings
                .keys()
                .flat_map(|k| ["all", "named"].map(|m| (k.clone(), m)))
                .filter(|(k, m)| cell(k, m)["verdict"] != base["verdict"])
                .map(|(k, m)| {
                    format!(
                        "{k}/{m} {} (in {:+.3})",
                        cell(&k, m)["verdict"],
                        inn(cell(&k, m)) - inn(&base)
                    )
                })
                .collect();
            match differ.is_empty() {
                true => format!(
                    "{c:<10} yes ({}, rows {}, in {:.3})",
                    base["verdict"],
                    base["rows"],
                    inn(&base)
                ),
                false => format!(
                    "{c:<10} no: k1/A/all {} (in {:.3}); differs: {}",
                    base["verdict"],
                    inn(&base),
                    differ.join("; ")
                ),
            }
        })
        .collect()
}
