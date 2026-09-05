//! An ignored instrument, isolated from production scoring and the frozen JSON writer.
mod association;
mod config;
mod data;
mod diagnostics;
mod feedback;
mod metrics;
mod novel;
mod output;
mod ranking;
mod roles;
mod score;
mod state;
mod stats;
mod translation;
mod validate;

use crate::similar_replay::{self, CORPORA, Measured};
use data::{Oracle, Pool};
use serde_json::json;
use state::{Cost, Run};
use std::time::Instant;

pub fn run() {
    let root = crate::common::repo_root();
    let path = root.join("contracts/eval/similar-oracle-v1.json");
    let text = std::fs::read_to_string(&path).expect("read frozen oracle");
    let oracle: Oracle = serde_json::from_str(&text).expect("oracle schema");
    let configs = config::configs();
    assert_eq!(configs[0].name, "baseline");
    let mut run = Run {
        costs: (0..configs.len()).map(|_| Cost::default()).collect(),
        outcomes: (0..configs.len()).map(|_| Vec::new()).collect(),
        configs,
        coverage: Vec::new(),
        measured: Vec::new(),
        pairs: Vec::new(),
        directory: root.join("cli/target/similar-tune"),
    };
    std::fs::create_dir_all(&run.directory).expect("result directory");
    output::write(&run.directory, "configurations.txt", config::TABLE);
    for (name, rel) in CORPORA {
        println!("measuring {name} with similar_replay::measure");
        let start = Instant::now();
        let m = similar_replay::measure(&root.join(rel), name);
        let s = stats::Stats::build(&m);
        output::write(
            &run.directory,
            &format!("sha-{}.json", name),
            &diagnostics::drift(&m, &oracle).to_string(),
        );
        run.measured.push(
            json!({"corpus": name, "measurement_ms": start.elapsed().as_millis(),
            "tally": similar_replay::tally(&m), "channel_tf": s.totals}),
        );
        evaluate_corpus(&mut run, &m, &s, &oracle);
        output::checkpoint(&run, &oracle);
    }
    output::finish(&run);
    assert_eq!(
        std::fs::read_to_string(path).expect("oracle remains readable"),
        text
    );
}

fn evaluate_corpus(run: &mut Run, m: &Measured, s: &stats::Stats, oracle: &Oracle) {
    let (pools, coverage) = data::pools(m, oracle);
    println!(
        "coverage {}",
        serde_json::to_string(&coverage).expect("coverage")
    );
    run.coverage.push(coverage);
    let mut unlabelled = Vec::new();
    for (i, p) in pools.iter().enumerate() {
        for (doc, c) in &p.candidates {
            run.pairs.push(roles::Pair {
                corpus: m.name.into(),
                same: c.truth == "same_role",
                evidence: data::evidence(m, p.query, *doc),
            });
        }
        unlabelled.push(evaluate_query(run, m, s, p));
        if i % 10 == 9 {
            println!("{}: {}/{} queries complete", m.name, i + 1, pools.len());
        }
    }
    output::json_lines(
        &run.directory,
        &format!("unlabelled-{}.jsonl", m.name),
        &unlabelled,
    );
}

fn evaluate_query(
    run: &mut Run,
    m: &Measured,
    s: &stats::Stats,
    p: &Pool<'_>,
) -> serde_json::Value {
    let frame = ranking::Frame::build(m, p.query);
    let seats: Vec<_> = p.candidates.iter().map(|(i, _)| *i).collect();
    let mut novel = novel::Novel::default();
    for (i, c) in run.configs.iter().enumerate() {
        let start = Instant::now();
        let prepared = ranking::Prepared::build(m, s, &frame, c);
        let ranked = ranking::rank(m, s, &frame, &prepared, &seats);
        run.costs[i].pool_us += start.elapsed().as_micros();
        if i == 0 {
            validate::baseline(m, p, &frame, &ranked);
        }
        if c.name == "ppmi_v1" {
            validate::widened(m, p.query, &ranked);
        }
        run.outcomes[i].push(metrics::outcome(m, p, &ranked));
        let start = Instant::now();
        let retrieved = ranking::retrieve(m, s, &frame, &prepared);
        run.costs[i].retrieval_us += start.elapsed().as_micros();
        let count = novel.add(m, p, &c.name, &retrieved);
        *run.costs[i].novel.entry(m.name.into()).or_default() += count[0];
        *run.costs[i].stale.entry(m.name.into()).or_default() += count[1];
    }
    novel.row(p)
}
