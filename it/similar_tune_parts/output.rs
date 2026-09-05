//! Durable checkpoints and complete paired tables, including every losing configuration.
use super::config;
use super::data::Oracle;
use super::metrics;
use super::roles;
use super::state::Run;
use crate::similar_replay::CORPORA;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fmt::Write as _, path::Path};

pub fn write(dir: &Path, name: &str, text: &str) {
    std::fs::write(dir.join(name), text).expect("write tuning artifact");
}

pub fn json_lines(dir: &Path, name: &str, rows: &[Value]) {
    let text: String = rows.iter().map(|r| format!("{r}\n")).collect();
    write(dir, name, &text);
}

pub fn checkpoint(run: &Run, oracle: &Oracle) {
    let complete = run.coverage.len() == CORPORA.len();
    write(&run.directory, "metadata.json", &json!({"complete": complete,
        "coverage": run.coverage, "measured": run.measured,
        "frozen_oracle_sha256": run.oracle_sha,
        "comparison": "current full-corpus statistics, SHA-matched labelled pools; fixed frozen query strata",
    }).to_string());
    write(
        &run.directory,
        "roles.json",
        &json!({"frozen": roles::table(&roles::frozen(oracle)),
        "current": roles::table(&run.pairs)})
        .to_string(),
    );
    let records: Vec<_> = run
        .configs
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let slices: BTreeMap<_, _> = metrics::scopes()
                .iter()
                .map(|scope| {
                    (
                        scope.clone(),
                        metrics::metric(&run.outcomes[i], &run.outcomes[0], scope),
                    )
                })
                .collect();
            json!({"config": c.name, "parameters": c.knobs, "metrics": slices, "cost": run.costs[i],
            "significant": complete && metrics::significant(&run.outcomes[i], &run.outcomes[0])})
        })
        .collect();
    json_lines(&run.directory, "metrics.jsonl", &records);
    for scope in metrics::scopes() {
        write(&run.directory, &format!("{scope}.md"), &table(run, &scope));
    }
    outcomes(run);
}

fn outcomes(run: &Run) {
    let rows: Vec<_> = (0..run.outcomes[0].len())
        .map(|j| {
            let configs: BTreeMap<_, _> = run
                .configs
                .iter()
                .enumerate()
                .map(|(i, c)| (c.name.as_str(), &run.outcomes[i][j]))
                .collect();
            json!(configs)
        })
        .collect();
    json_lines(&run.directory, "outcomes.jsonl", &rows);
}

fn fraction(value: [usize; 2]) -> String {
    format!("{}/{}", value[0], value[1])
}

fn table(run: &Run, scope: &str) -> String {
    let mut out = format!("# {scope}: labelled-pool re-ranking\n\n");
    out.push_str("Fixed R1/R0 use frozen query strata; selected R1/R0 and NC use the selected candidate.\n\n");
    out.push_str("| Config | p@1 | Fixed R1 | Fixed R0 | NC | hit@5 | W/L | R1 W/L | Selected R1 | Selected R0 | Fixed NC | hit5 W/L |\n");
    out.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for (i, c) in run.configs.iter().enumerate() {
        let m = metrics::metric(&run.outcomes[i], &run.outcomes[0], scope);
        let cells = [
            m.p1,
            m.fixed_role1,
            m.fixed_role0,
            m.nonclone,
            m.hit5,
            m.paired,
            m.paired_role1,
            m.selected_role1,
            m.selected_role0,
            m.fixed_nonclone,
            m.paired_hit5,
        ];
        let _ = writeln!(
            out,
            "| {} | {} |",
            c.name,
            cells
                .into_iter()
                .map(fraction)
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }
    out
}

pub fn finish(run: &Run) {
    let heldout: Vec<_> = CORPORA
        .iter()
        .map(|(corpus, _)| {
            let scope = format!("without_{corpus}");
            let mut choice = 0;
            let mut best = (0, 0, i128::MIN);
            for (i, rows) in run.outcomes.iter().enumerate() {
                let m = metrics::metric(rows, &run.outcomes[0], &scope);
                let key = (m.p1[0], m.fixed_role1[0], -(m.paired[1] as i128));
                if key > best {
                    choice = i;
                    best = key;
                }
            }
            json!({"heldout": corpus, "selected_on_other_four": run.configs[choice].name,
            "training": metrics::metric(&run.outcomes[choice], &run.outcomes[0], &scope),
            "test": metrics::metric(&run.outcomes[choice], &run.outcomes[0], corpus)})
        })
        .collect();
    json_lines(&run.directory, "heldout-selection.jsonl", &heldout);
    println!("{}", table(run, "all"));
    println!(
        "{} configurations; artifacts: {}",
        config::configs().len(),
        run.directory.display()
    );
}
