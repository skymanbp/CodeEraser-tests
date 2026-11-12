//! BARS-GEN2 section 3's report, from the replay's per-corpus docs: per
//! state the events and each fate per corpus (all kinds and per kind) and
//! pooled, the false share under A and under B with its Clopper-Pearson
//! 95 % upper bound, the pooled share with each corpus left out (all kinds
//! and per kind), then the bars condition by condition with their
//! integers and the outcome's sentence (G2), and the side numbers (G3).
//! A reader checks the outcome from the printed integers alone; the
//! report judges nothing the bars do not.
//!   CE_DEADFILE_GEN2_OUT=<dir> cargo test --test it -- --ignored eval_deadfile_gen2_parts::report::deadfile_gen2_report --nocapture

use super::doc::{checked, read_all};
use super::model::{Event, Fate, Kind, State};
use super::readings::{
    MIN_EVENTS, Reading, Table, conditions, fates, leave_one_out, outcome, own_kinds, pooled,
    sentence, share, tally, upper_ppm,
};
use super::words::{KINDS, STATES, fate_of, value, word};
use super::{dir_name, registry};
use crate::eval_lang_parts::generate::env;
use serde_json::Value;

/// The table of a run's docs' events; every word read strictly (G4).
pub fn table_of(doc: &Value) -> Table {
    let mut t = Table::new();
    for c in doc["corpora"].as_array().into_iter().flatten() {
        let corpus = c["corpus"].as_str().expect("corpus");
        let events: Vec<(Event, Fate)> = c["events"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|e| {
                let text = |k: &str| e[k].as_str().unwrap_or_else(|| panic!("event without {k}"));
                let ev = Event {
                    at: 0,
                    path: text("path").into(),
                    state: value(&STATES, "state", text("state")),
                    kind: value(&KINDS, "kind", text("kind")),
                };
                (ev, fate_of(text("fate")))
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

const KIND_ROWS: [(Option<Kind>, &str); 3] = [
    (None, ""),
    (Some(Kind::Code), " code"),
    (Some(Kind::Document), " document"),
];

/// The fate table of one state: per corpus, per corpus and kind, pooled.
fn fate_table(t: &Table, state: State, out: &mut Vec<String>) {
    out.push("| corpus / kind | events | re-referred | kept up focused | kept up sweep | removed untouched | undetermined | false A | false B |".into());
    out.push("|---|---|---|---|---|---|---|---|---|".into());
    for corpus in t.keys() {
        for (kind, k) in KIND_ROWS {
            out.push(line(
                &format!("{corpus}{k}"),
                &fates(t, corpus, state, kind),
            ));
        }
    }
    for (kind, k) in KIND_ROWS {
        out.push(line(
            &format!("pooled{k}"),
            &pooled(t, state, kind, &|_| true),
        ));
    }
    for r in [Reading::A, Reading::B] {
        for (kind, k) in KIND_ROWS {
            let loo: Vec<String> = leave_one_out(t, state, kind, r)
                .into_iter()
                .map(|(c, k, n)| format!("without {c} {}", cell(k, n)))
                .collect();
            out.push(format!(
                "\nleave one out{k}, reading {r:?}: {}",
                loo.join("; ")
            ));
        }
    }
}

/// Conditions 2 and 3 of one kind (or every kind), each with its integers.
fn condition_lines(t: &Table, state: State, kind: Option<Kind>, r: Reading, out: &mut Vec<String>) {
    for c in conditions(t, state, kind, r) {
        let yes = if c.ok { "holds" } else { "fails" };
        match &c.corpus {
            None => out.push(format!("(2) pooled {}/{} at most 1.0 %: {yes}", c.k, c.n)),
            Some(name) => out.push(format!(
                "(3) {name} ({} events) {}/{} at most 5.0 %: {yes}",
                c.events, c.k, c.n
            )),
        }
    }
}

/// The bars of one state, condition by condition, then the outcome.
fn bar_lines(t: &Table, state: State, name: &str, out: &mut Vec<String>) {
    let events = pooled(t, state, None, &|_| true).len();
    let enough = if events >= MIN_EVENTS {
        "holds"
    } else {
        "fails"
    };
    out.push(format!(
        "\nbars, {name}:\n(1) {events} events, at least {MIN_EVENTS}: {enough}"
    ));
    if events >= MIN_EVENTS {
        for r in [Reading::A, Reading::B] {
            out.push(format!("reading {r:?}:"));
            condition_lines(t, state, None, r, out);
            for kind in [Kind::Code, Kind::Document] {
                let n = pooled(t, state, Some(kind), &|_| true).len();
                let kw = word(&KINDS, kind);
                if own_kinds(t, state).contains(&kind) {
                    out.push(format!(
                        "(4) kind {kw} ({n} events, at least {MIN_EVENTS}):"
                    ));
                    condition_lines(t, state, Some(kind), r, out);
                } else {
                    out.push(format!(
                        "(4) kind {kw} ({n} events, fewer than {MIN_EVENTS}): not read"
                    ));
                }
            }
        }
    }
    out.push(format!("\n{}\n", sentence(name, outcome(t, state))));
}

/// The report of one table.
pub fn render(t: &Table) -> String {
    let mut out = Vec::new();
    for (name, state) in STATES {
        out.push(format!("### {name}\n"));
        fate_table(t, state, &mut out);
        bar_lines(t, state, name, &mut out);
    }
    out.join("\n")
}

const SIDES: [&str; 5] = [
    "unjudged_pairs",
    "unjudged_after_window",
    "met_unjudged",
    "dependents",
    "symlink_rows",
];

/// The side numbers per corpus and pooled, and every unjudged tree (G3).
pub fn render_sides(docs: &[Value]) -> String {
    let mut out = vec![
        format!("| corpus | {} |", SIDES.join(" | ")),
        "|---|---|---|---|---|---|".into(),
    ];
    let mut sums = [0u64; 5];
    for d in docs {
        let v: Vec<u64> = SIDES
            .iter()
            .map(|k| d[*k].as_u64().unwrap_or_else(|| panic!("{k}")))
            .collect();
        sums.iter_mut().zip(&v).for_each(|(s, x)| *s += x);
        let cells: Vec<String> = v.iter().map(u64::to_string).collect();
        out.push(format!(
            "| {} | {} |",
            d["corpus"].as_str().unwrap_or(""),
            cells.join(" | ")
        ));
    }
    let cells: Vec<String> = sums.iter().map(u64::to_string).collect();
    out.push(format!("| pooled | {} |", cells.join(" | ")));
    for d in docs {
        for u in d["unjudged_trees"].as_array().into_iter().flatten() {
            out.push(format!(
                "unjudged {} {}: {}",
                d["corpus"], u["commit"], u["reason"]
            ));
        }
    }
    out.join("\n")
}

#[test]
#[ignore = "reads the per-corpus docs a generation-2 run wrote"]
fn deadfile_gen2_report() {
    let docs = read_all(std::path::Path::new(&env("CE_DEADFILE_GEN2_OUT")));
    let reg = registry();
    let chosen: Vec<&str> = reg["corpora"]
        .as_array()
        .into_iter()
        .flatten()
        .map(dir_name)
        .collect();
    checked(&docs, &chosen).unwrap_or_else(|e| panic!("the run's docs: {e}"));
    let run = serde_json::json!({"corpora": docs});
    println!("{}\n\n{}", render(&table_of(&run)), render_sides(&docs));
}
