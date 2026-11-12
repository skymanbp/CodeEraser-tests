//! The registry page's section 4 tables and its outcome (split from
//! page.rs at its 300-line soft line): the eight readings, the
//! leave-one-project-out flips, "same under all readings?", the agreed
//! outcomes and the one-sentence outcome.

use super::bars::{CLASSES, same_under_all};
use super::page::{s, share, table};
use serde_json::Value;

/// BARS §4 under the eight readings: per reading and class the rows, the
/// weighted in-share and the verdict, all answers and named only.
pub fn section4_table(report: &Value) -> String {
    let readings = report["readings"].as_object().cloned().unwrap_or_default();
    let cell = |j: &Value| {
        format!(
            "{} rows, in {}, {}",
            j["rows"],
            share(&j["in"]),
            s(&j["verdict"])
        )
    };
    let body = readings
        .iter()
        .flat_map(|(key, r)| ["all", "named"].map(|m| (key.clone(), m, r.clone())))
        .map(|(key, m, r)| {
            let mut row = vec![key, m.into()];
            row.extend(CLASSES.iter().map(|c| cell(&r["classes"][*c][m])));
            row.push(s(&r["code"]));
            row
        })
        .collect();
    table(
        &[
            "reading",
            "in_use counted",
            "runs-alone",
            "declares",
            "document",
            "code outcome (all answers)",
        ],
        body,
    )
}

/// Every leave-one-project-out line where taking a project's rows out
/// flips a class's verdict: (reading, mode, class, project, in, verdict).
pub fn flips_table(report: &Value) -> String {
    let mut body = Vec::new();
    for (key, r) in report["readings"].as_object().into_iter().flatten() {
        for c in CLASSES {
            for m in ["all", "named"] {
                let j = &r["classes"][c][m];
                for (p, v) in j["leave_one_out"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter(|(_, v)| v["verdict"] != j["verdict"])
                {
                    body.push(vec![
                        key.clone(),
                        m.into(),
                        c.into(),
                        p.clone(),
                        s(&j["verdict"]),
                        format!("{} (in {})", s(&v["verdict"]), share(&v["in"])),
                    ]);
                }
            }
        }
    }
    table(
        &[
            "reading",
            "in_use counted",
            "class",
            "project left out",
            "verdict",
            "verdict without it",
        ],
        body,
    )
}

/// Per class whether its verdict is the same under all eight readings.
pub fn same_table(report: &Value) -> String {
    let body = same_under_all(report).into_iter().map(|l| {
        let (class, rest) = l.split_once(' ').unwrap_or((&l, ""));
        vec![class.to_string(), rest.trim().to_string()]
    });
    table(&["class", "same under all readings?"], body.collect())
}

/// The two agreed outcomes as BARS §4 states them.
pub fn outcomes(report: &Value) -> String {
    let line = |k: &str| {
        format!(
            "- {}: {} (readings agree: {})",
            k,
            s(&report[k]["outcome"]),
            report[k]["agreed"]
        )
    };
    [line("code"), line("document")].join("\n")
}

/// The outcome in one sentence, from the agreed outcomes and the
/// passing set. The opening clause holds only when no code outcome
/// ships: the readings differ, or they agree on "nothing ships"; any
/// other agreed code outcome has no sentence here and is refused.
pub fn outcome_sentence(report: &Value, passing: &[&str]) -> Result<String, String> {
    let names: Vec<String> = passing.iter().map(|n| format!("`{n}`")).collect();
    let agreed = |k: &str| report[k]["agreed"] == true;
    let code_outcome = s(&report["code"]["outcome"]);
    if agreed("code") && !code_outcome.starts_with("nothing ships") {
        return Err(format!(
            "no sentence for the agreed code outcome `{code_outcome}`"
        ));
    }
    let asked = |c: &str| {
        let all = report["readings"].as_object().into_iter().flatten();
        let mut questions =
            all.flat_map(|(_, r)| r["questions"].as_array().cloned().unwrap_or_default());
        questions.any(|q| q.as_str().is_some_and(|q| q.starts_with(&format!("{c}:"))))
    };
    let doc_outcome = s(&report["document"]["outcome"]);
    let document = match (agreed("document"), asked("document")) {
        (true, true) => format!(
            "is {doc_outcome} under every reading and, failing with an in-share of at least 0.50, an owner question"
        ),
        (true, false) => format!("is {doc_outcome} under every reading"),
        (false, _) => "is an owner question (the readings differ)".into(),
    };
    let code = if agreed("code") {
        format!("is \"{code_outcome}\" under every reading")
    } else {
        "is an owner question (the readings differ)".into()
    };
    let count = crate::facts::form::Form::WordCap.render(&passing.len().to_string(), false);
    Ok(format!(
        "Generation 1 ships no split of unreferenced files: the code outcome {code} and the document outcome {document}. {count} tool conventions pass BARS section 3: {}.",
        names.join(", ")
    ))
}
