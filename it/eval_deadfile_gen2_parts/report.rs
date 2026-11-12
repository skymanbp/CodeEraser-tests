//! BARS-GEN2 section 3's report, from the replay's doc: per state the
//! events and each fate per corpus and pooled, the false share under A
//! and under B with its Clopper-Pearson 95 % upper bound, the pooled
//! share with each corpus left out, and the outcome's sentence. The
//! report reads the doc only; it judges nothing the bars do not.
//!   CE_DEADFILE_GEN2_OUT=<dir> cargo test --test it -- --ignored eval_deadfile_gen2_parts::report::deadfile_gen2_report --nocapture

use super::model::{Event, Fate, Kind, State};
use super::readings::{
    Reading, Table, fates, leave_one_out, outcome, pooled, sentence, share, tally, upper_ppm,
};
use crate::eval_lang_parts::generate::env;
use serde_json::Value;

/// The replay doc's word for each fate: one table, written by run.rs
/// and read back here.
const FATES: [(&str, Fate); 5] = [
    ("re-referred", Fate::ReReferred),
    ("kept up focused", Fate::KeptUp { focused: true }),
    ("kept up sweep", Fate::KeptUp { focused: false }),
    ("removed untouched", Fate::RemovedUntouched),
    ("undetermined", Fate::Undetermined),
];

pub fn fate_name(f: Fate) -> &'static str {
    FATES
        .iter()
        .find(|(_, x)| *x == f)
        .map(|(n, _)| *n)
        .expect("every fate has a word")
}

/// The fate a replay doc names.
pub fn fate_of(name: &str) -> Fate {
    FATES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, f)| *f)
        .unwrap_or_else(|| panic!("unknown fate {name:?}"))
}

/// The table of a replay doc's events.
pub fn table_of(doc: &Value) -> Table {
    let mut t = Table::new();
    for c in doc["corpora"].as_array().into_iter().flatten() {
        let corpus = c["corpus"].as_str().expect("corpus");
        let events: Vec<(Event, Fate)> = c["events"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|e| {
                let state = if e["state"] == "orphaned" {
                    State::Orphaned
                } else {
                    State::Born
                };
                let kind = if e["kind"] == "document" {
                    Kind::Document
                } else {
                    Kind::Code
                };
                let ev = Event {
                    at: 0,
                    path: e["path"].as_str().unwrap_or("").into(),
                    state,
                    kind,
                };
                (ev, fate_of(e["fate"].as_str().expect("fate")))
            })
            .collect();
        tally(corpus, &events, &mut t);
    }
    t
}

/// `k/n (x.x %, upper y.y %)`.
fn cell(k: usize, n: usize) -> String {
    let pct = if n == 0 {
        0.0
    } else {
        100.0 * k as f64 / n as f64
    };
    format!(
        "{k}/{n} ({pct:.1} %, upper {:.1} %)",
        upper_ppm(k, n) as f64 / 10_000.0
    )
}

/// One line of fates and both shares.
fn line(label: &str, f: &[Fate]) -> String {
    let count = |want: fn(&Fate) -> bool| f.iter().filter(|x| want(x)).count();
    let (a, b) = (share(Reading::A, f), share(Reading::B, f));
    format!(
        "| {label} | {} | {} | {} | {} | {} | {} | {} | {} |",
        f.len(),
        count(|x| *x == Fate::ReReferred),
        count(|x| *x == Fate::KeptUp { focused: true }),
        count(|x| *x == Fate::KeptUp { focused: false }),
        count(|x| *x == Fate::RemovedUntouched),
        count(|x| *x == Fate::Undetermined),
        cell(a.0, a.1),
        cell(b.0, b.1)
    )
}

/// The report of one table.
pub fn render(t: &Table) -> String {
    let mut out = Vec::new();
    for (state, name) in [(State::Orphaned, "orphaned"), (State::Born, "born")] {
        out.push(format!("### {name}\n"));
        out.push("| corpus / kind | events | re-referred | kept up focused | kept up sweep | removed untouched | undetermined | false A | false B |".into());
        out.push("|---|---|---|---|---|---|---|---|---|".into());
        for corpus in t.keys() {
            out.push(line(corpus, &fates(t, corpus, state, None)));
        }
        for (kind, k) in [
            (None, "pooled"),
            (Some(Kind::Code), "pooled code"),
            (Some(Kind::Document), "pooled document"),
        ] {
            out.push(line(k, &pooled(t, state, kind, &|_| true)));
        }
        for r in [Reading::A, Reading::B] {
            let loo: Vec<String> = leave_one_out(t, state, r)
                .into_iter()
                .map(|(c, k, n)| format!("without {c} {}", cell(k, n)))
                .collect();
            out.push(format!(
                "\nleave one out, reading {r:?}: {}",
                loo.join("; ")
            ));
        }
        out.push(format!("\n{}\n", sentence(name, outcome(t, state))));
    }
    out.join("\n")
}

#[test]
#[ignore = "reads the replay doc a generation-2 run wrote"]
fn deadfile_gen2_report() {
    let path =
        std::path::Path::new(&env("CE_DEADFILE_GEN2_OUT")).join("deadfile-gen2-replay-v1.json");
    let doc = crate::eval_support::load(&path.to_string_lossy());
    println!("{}", render(&table_of(&doc)));
}
