//! The registry page's numbers (docs/EVAL-SET-DEADFILE.md, part 3A):
//! every table on the page is rendered here from the committed record
//! and the frozen docs, and the CI gate holds the page to carrying each
//! one verbatim — the page is checked, never hand-typed. The section 4
//! tables and the outcome are page_bars.rs.

use super::review::WORDS;
use serde_json::Value;

/// A Markdown table from a header row and body rows.
pub(super) fn table(head: &[&str], body: Vec<Vec<String>>) -> String {
    let cells = |r: Vec<String>| format!("| {} |", r.join(" | "));
    let mut out = vec![
        cells(head.iter().map(|h| h.to_string()).collect()),
        format!("|{}|", vec!["---"; head.len()].join("|")),
    ];
    out.extend(body.into_iter().map(cells));
    out.join("\n")
}

pub(super) fn s(v: &Value) -> String {
    match v {
        Value::String(t) => t.clone(),
        other => other.to_string(),
    }
}

pub(super) fn share(v: &Value) -> String {
    v.as_f64().map_or("-".into(), |x| format!("{x:.3}"))
}

/// Per language: the universe's rows (the pool), the primary sample and
/// the reserve.
pub fn sample_table(samples: &[Value]) -> String {
    let len = |d: &Value, k: &str| d[k].as_array().map_or(0, Vec::len);
    let mut body: Vec<Vec<String>> = samples
        .iter()
        .map(|d| {
            vec![
                s(&d["lang"]),
                s(&d["pool"]),
                len(d, "rows").to_string(),
                len(d, "reserve").to_string(),
            ]
        })
        .collect();
    let sum = |f: &dyn Fn(&Value) -> usize| samples.iter().map(f).sum::<usize>().to_string();
    body.push(vec![
        "all".into(),
        sum(&|d| d["pool"].as_u64().unwrap_or(0) as usize),
        sum(&|d| len(d, "rows")),
        sum(&|d| len(d, "reserve")),
    ]);
    table(
        &["language", "universe rows", "primary sample", "reserve"],
        body,
    )
}

/// The readings table's rows: `label|key`.
const READINGS: &str = "rows answered (first reading)|n
in_use / unused / unsure|first
in_use tagged named / generic|tags
rows read twice|twice
same side of in_use, all rows read twice|overall
same side, first answer in_use and generic|first_generic
same side, first answer in_use and named|first_named
same side, first answer not in_use|first_not_in_use
final in_use / unused / unsure|final";

/// The two readings: answers of the first, the tags, the rows read twice
/// and their agreement by the first answer's kind.
pub fn readings_table(rows: &[Value], agreement: &Value) -> String {
    let count = |at: &str, word: &str| {
        rows.iter()
            .filter(|r| r.pointer(at).and_then(Value::as_str) == Some(word))
            .count()
            .to_string()
    };
    let words = |at: &str| WORDS.map(|w| count(at, w)).join(" / ");
    let value = |key: &str| match key {
        "n" => rows.len().to_string(),
        "first" => words("/first/answer"),
        "tags" => ["named", "generic"]
            .map(|t| count("/first/tag", t))
            .join(" / "),
        "twice" => rows
            .iter()
            .filter(|r| !r["second"].is_null())
            .count()
            .to_string(),
        "final" => words("/answer"),
        k => format!(
            "{} / {} ({})",
            agreement[k]["same_side"],
            agreement[k]["rows"],
            share(&agreement[k]["share"])
        ),
    };
    let body = READINGS
        .lines()
        .filter_map(|l| l.split_once('|'))
        .map(|(label, key)| vec![label.to_string(), value(key)]);
    table(&["quantity", "value"], body.collect())
}

/// The rows whose two readings gave two different answers and the
/// settlement's word for each.
pub fn settled_table(rows: &[Value]) -> String {
    let differ = rows
        .iter()
        .filter(|r| !r["second"].is_null() && r["first"]["answer"] != r["second"]["answer"]);
    let body = differ
        .map(|r| {
            vec![
                s(&r["corpus"]),
                format!("`{}`", s(&r["path"])),
                s(&r["first"]["answer"]),
                s(&r["second"]["answer"]),
                s(&r["answer"]),
            ]
        })
        .collect();
    table(&["corpus", "path", "first", "second", "settled"], body)
}

/// One §3 judgment's outcome in words.
fn outcome(j: &Value) -> String {
    let rules: Vec<String> = j["refused_by"]
        .as_array()
        .into_iter()
        .flatten()
        .map(Value::to_string)
        .collect();
    match (j["passes"] == true, j["kind"] == "project") {
        (true, _) => "passes".into(),
        (false, true) => "stays out by rule 1 (a project's convention)".into(),
        (false, false) => format!("refused by rule {}", rules.join(" and ")),
    }
}

/// BARS §3: per candidate its carrying rows, answers, the rule facts and
/// the outcome; the `beside` reading on its own line, never deciding.
pub fn section3_table(judged: &[Value]) -> String {
    let yes = |v: &Value| if *v == true { "yes" } else { "no" }.to_string();
    let row = |name: String, j: &Value| {
        vec![
            name,
            s(&j["rows"]),
            format!("{} / {} / {}", j["in_use"], j["unused"], j["unsure"]),
            yes(&j["names_tool"]),
            yes(&j["documented"]),
            yes(&j["position"]),
            outcome(j),
        ]
    };
    let mut body = Vec::new();
    for j in judged {
        body.push(row(format!("`{}`", s(&j["name"])), j));
        if j["beside"].is_object() {
            body.push(row(
                "- beside (printed for comparison, not evaluated)".into(),
                &j["beside"],
            ));
        }
    }
    table(
        &[
            "candidate",
            "rows",
            "in_use / unused / unsure",
            "an answer names the tool",
            "documented",
            "position rule",
            "outcome",
        ],
        body,
    )
}

/// Population B per corpus: dead rows before and after, the matched
/// files and the dead rows they reach beyond themselves.
pub fn population_table(doc: &Value) -> String {
    let len = |r: &Value, k: &str| r[k].as_array().map_or(0, Vec::len).to_string();
    let body = doc["corpora"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|r| {
            vec![
                s(&r["corpus"]),
                format!("{} -> {}", r["before"], r["after"]),
                format!("{} ({} dead)", len(r, "matched"), len(r, "matched_dead")),
                len(r, "reached_beyond"),
            ]
        })
        .collect();
    table(
        &[
            "corpus",
            "dead before -> after",
            "matched files",
            "reached beyond matched",
        ],
        body,
    )
}

/// The committed record: every file and its pinned sha256.
pub fn record_table(pinned: &[(&str, &str)]) -> String {
    let body = pinned
        .iter()
        .map(|(p, h)| vec![format!("`contracts/eval/{p}`"), format!("`{h}`")])
        .collect();
    table(&["record file", "sha256"], body)
}
